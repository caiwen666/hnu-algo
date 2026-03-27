use std::cmp::min;
use std::collections::BTreeSet;

use super::path_dist::PathDist;

const NONE: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullResult {
    pub boundary: PathDist,
    pub keys: Vec<u32>,
}

#[derive(Debug, Clone)]
struct Node {
    key: u32,
    value: PathDist,
    prev: u32,
    next: u32,
    block_id: u32,
    alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockSeq {
    D0,
    D1,
}

#[derive(Debug, Clone)]
struct Block {
    /// block 属于哪个链表
    seq: BlockSeq,
    /// 前一个 block_id
    prev: u32,
    /// 后一个 block_id
    next: u32,
    /// 头部 node_id
    head: u32,
    /// 尾部 node_id
    tail: u32,
    /// block 内的节点数量
    len: u32,
    /// block 的上界
    upper: PathDist,
    /// block 是否存活
    alive: bool,
}

impl Block {
    #[inline]
    fn new(seq: BlockSeq, upper: PathDist) -> Self {
        Self {
            seq,
            prev: NONE,
            next: NONE,
            head: NONE,
            tail: NONE,
            len: 0,
            upper,
            alive: true,
        }
    }
}

#[derive(Debug)]
pub struct BlockDs<'a> {
    m: u32,
    upper_bound_b: PathDist,
    /// key -> node_id，NONE 表示不存在（外部传入，由调用方预分配）
    key_to_node: &'a mut [u32],
    /// 记录哪些 key 被修改过，用于 drop 时清理
    dirty_keys: Vec<u32>,
    alive_count: u32,
    nodes: Vec<Node>,
    blocks: Vec<Block>,
    d0_head: u32,
    d0_tail: u32,
    d1_head: u32,
    d1_tail: u32,
    d1_upper_index: BTreeSet<(PathDist, u32)>,
    /// `batch_prepend` 复用，避免每层 `pull` 循环里反复分配
    prep_pairs: Vec<(u32, PathDist)>,
    prep_block_ids: Vec<u32>,
}

impl<'a> BlockDs<'a> {
    /// 创建一个新的 BlockDs
    ///
    /// # Parameters
    ///
    /// - m: 每个 block 的最大节点数量，也是 pull 操作返回的节点数量
    /// - upper_bound_b: 数据结构中 value 的上界
    /// - key_to_node_buf: 复用内存池
    ///
    /// # Preconditions
    ///
    /// - `key_to_node_buf` 必须全部为 NONE，调用方负责保证。Drop 时会自动清理脏位。
    ///
    /// - `key_to_node_buf` 长度必须与要维护的 key 的值域一致
    pub fn new(m: usize, upper_bound_b: PathDist, key_to_node_buf: &'a mut [u32]) -> Self {
        let m = (m.max(1)) as u32;
        let mut ds = Self {
            m,
            upper_bound_b,
            key_to_node: key_to_node_buf,
            dirty_keys: Vec::new(),
            alive_count: 0,
            nodes: Vec::new(),
            blocks: Vec::new(),
            d0_head: NONE,
            d0_tail: NONE,
            d1_head: NONE,
            d1_tail: NONE,
            d1_upper_index: BTreeSet::new(),
            prep_pairs: Vec::new(),
            prep_block_ids: Vec::new(),
        };
        let first = ds.new_block(BlockSeq::D1, upper_bound_b);
        ds.d1_head = first;
        ds.d1_tail = first;
        ds.d1_upper_index.insert((upper_bound_b, first));
        ds
    }

    /// 判断当前数据结构是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
    }

    /// 获取当前数据结构中节点的数量
    #[inline]
    pub fn len(&self) -> usize {
        self.alive_count as usize
    }

    /// 插入一个节点，如果节点已经存在，则对其值取 min
    ///
    /// # Parameters
    ///
    /// - key: 要插入的节点的 key
    /// - value: 要插入的节点的 value，需要满足 `value.end == key`
    ///
    /// # Panics
    ///
    /// 如果 value 大于数据结构上界，或 `value.end != key`，则 panic
    pub fn insert(&mut self, key: u32, value: PathDist) {
        assert_eq!(
            value.end(),
            key,
            "PathDist.end must match insert key"
        );
        if value > self.upper_bound_b {
            panic!("value is greater than upper bound");
        }
        let old_id = self.key_to_node[key as usize];
        if old_id != NONE {
            if self.nodes[old_id as usize].value > value {
                self.delete_node(old_id);
            } else {
                return;
            }
        }

        let block_id = self.locate_d1_block(value);
        let node_id = self.new_node(key, value, block_id);
        self.key_to_node[key as usize] = node_id;
        self.dirty_keys.push(key);
        self.alive_count += 1;
        self.push_node_to_block_tail(block_id, node_id);

        if self.blocks[block_id as usize].len > self.m {
            self.split_d1_block(block_id);
        }
    }

    /// 插入一批 key-value
    ///
    /// 类似 insert 的约定，如果 key 在先前已经存在，会把之前的删掉（直接删，因为 batch_prepend 的 value 必然更小）
    ///
    /// 如果要插入的节点链表中存在相同的 key，则只取 value 最小的
    ///
    /// # Parameters
    ///
    /// - records: 要插入的节点列表。节点列表中的元素必须满足 `PathDist.end = key`
    ///
    /// # Preconditions
    ///
    /// - 所有的 value 必须严格小于当前数据结构中已有的 value 的最小值。该函数不会对这个条件进行任何的检查（包括 debug assertions）。
    ///
    /// # Panics
    ///
    /// - 如果 `records` 中存在 `PathDist.end != key`，则 panic
    pub fn batch_prepend(&mut self, records: &[(u32, PathDist)], pool: &mut [PathDist]) {
        if records.is_empty() {
            return;
        }

        // 对 records 去重，并把已经存在于数据结构中的点删掉
        self.prep_pairs.clear();
        self.prep_pairs.reserve(records.len());
        for &(key, value) in records.iter() {
            assert_eq!(
                value.end(),
                key,
                "PathDist.end must match record key"
            );
            let old_id = self.key_to_node[key as usize];
            if old_id != NONE {
                self.delete_node(old_id);
            }
            if pool[key as usize] != PathDist::MAX {
                pool[key as usize] = pool[key as usize].min(value);
            } else {
                // 先把第一个值插进去，主要是为了知道有这个值，后面再扫一遍拿最新值
                self.prep_pairs.push((key, value));
                pool[key as usize] = value;
            }
        }

        self.prep_pairs.iter_mut().for_each(|(key, value)| {
            *value = pool[*key as usize];
            // 需要保证 pool 调用前后，里面的值都是 PathDist::MAX
            pool[*key as usize] = PathDist::MAX;
        });

        // 论文中使用 BFPRT 来进行递归划分
        // 考虑到 BFPRT 的常数可能较大，这里直接使用排序
        self.prep_pairs.sort_unstable_by_key(|&(k, v)| (v, k));

        self.prep_block_ids.clear();
        let m = self.m as usize;
        let mut cursor = 0usize;
        while cursor < self.prep_pairs.len() {
            // [cursor, end) 这个区间为一块（按索引迭代，避免与 new_node 等对 &mut self 的调用冲突）
            let end = min(cursor + m, self.prep_pairs.len());
            let upper = self.prep_pairs[end - 1].1;
            let block_id = self.new_block(BlockSeq::D0, upper);
            self.prep_block_ids.push(block_id);
            for idx in cursor..end {
                let (k, v) = self.prep_pairs[idx];
                let node_id = self.new_node(k, v, block_id);
                self.key_to_node[k as usize] = node_id;
                self.dirty_keys.push(k);
                self.alive_count += 1;
                self.push_node_to_block_tail(block_id, node_id);
            }
            cursor = end;
        }

        while let Some(block_id) = self.prep_block_ids.pop() {
            self.prepend_d0_block(block_id);
        }
    }

    /// 拉取最小 M 个节点
    ///
    /// 如果数据结构内的节点数量小于 M，则返回所有节点
    ///
    /// # Returns
    ///
    /// 返回拉取到的节点和上界
    ///
    /// 其中上界为剩余数据结构中的最小值，如果剩余数据结构为空，则上界为数据结构的上界
    pub fn pull(&mut self) -> PullResult {
        let m = self.m as usize;
        // 多数情况下只需从前缀若干 block 取满 m 个节点；预留略大于 m 避免频繁 realloc
        let mut cand_ids: Vec<u32> = Vec::with_capacity(m * 2);

        let mut collect = |head_block_id: u32, blocks: &[Block], nodes: &[Node]| {
            let mut cur = head_block_id;
            let mut collected_count = 0usize;
            while cur != NONE {
                if collected_count >= m {
                    break;
                }
                let mut p = blocks[cur as usize].head;
                while p != NONE {
                    cand_ids.push(p);
                    collected_count += 1;
                    p = nodes[p as usize].next;
                }
                cur = blocks[cur as usize].next;
            }
        };

        collect(self.d0_head, &self.blocks, &self.nodes);
        collect(self.d1_head, &self.blocks, &self.nodes);

        // pending_boundary 是 None 的话说明还不能确定，等把节点删完之后再求数据结构中剩余的 value 最小值
        let (result_ids, pending_boundary) = if cand_ids.len() < m {
            // 如果是小于 m 的话，说明两个链表都遍历完了，上界就是整个数据结构的上界
            (cand_ids, Some(self.upper_bound_b))
        } else if cand_ids.len() == m {
            (cand_ids, None)
        } else {
            let nodes = &self.nodes;
            cand_ids.select_nth_unstable_by_key(m - 1, |&id| {
                (nodes[id as usize].value, nodes[id as usize].key)
            });
            cand_ids.truncate(m);
            (cand_ids, None)
        };

        for &id in &result_ids {
            debug_assert!(self.nodes[id as usize].alive);
            self.delete_node(id);
        }

        let boundary = pending_boundary
            .unwrap_or_else(|| self.min_remaining_value().unwrap_or(self.upper_bound_b));

        // 根据论文中的要求，拉取到的节点必须严格小于上界
        for &id in &result_ids {
            debug_assert!(self.nodes[id as usize].value < boundary);
        }

        let keys: Vec<u32> = result_ids
            .iter()
            .map(|&id| self.nodes[id as usize].key)
            .collect();

        PullResult { boundary, keys }
    }

    #[inline]
    fn new_block(&mut self, seq: BlockSeq, upper: PathDist) -> u32 {
        let id = self.blocks.len() as u32;
        self.blocks.push(Block::new(seq, upper));
        id
    }

    #[inline]
    fn new_node(&mut self, key: u32, value: PathDist, block_id: u32) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node {
            key,
            value,
            prev: NONE,
            next: NONE,
            block_id,
            alive: true,
        });
        id
    }

    /// 把某个 block 插入到 d0 的头部
    ///
    /// # Parameters
    ///
    /// - block_id: 要插入的 block_id
    #[inline]
    fn prepend_d0_block(&mut self, block_id: u32) {
        let old = self.d0_head;
        self.blocks[block_id as usize].prev = NONE;
        self.blocks[block_id as usize].next = old;
        if old != NONE {
            self.blocks[old as usize].prev = block_id;
        } else {
            self.d0_tail = block_id;
        }
        self.d0_head = block_id;
    }

    /// 把某个节点插入到某个 block 的尾部
    ///
    /// # Parameters
    ///
    /// - block_id: 要插入的 block_id
    /// - node_id: 要插入的节点 id
    #[inline]
    fn push_node_to_block_tail(&mut self, block_id: u32, node_id: u32) {
        let tail = self.blocks[block_id as usize].tail;
        self.nodes[node_id as usize].prev = tail;
        self.nodes[node_id as usize].next = NONE;
        if tail != NONE {
            self.nodes[tail as usize].next = node_id;
        } else {
            self.blocks[block_id as usize].head = node_id;
        }
        self.blocks[block_id as usize].tail = node_id;
        self.blocks[block_id as usize].len += 1;
    }

    /// 查找第一个上界大于等于 value 的 block
    ///
    /// # Returns
    ///
    /// 返回 block_id
    #[inline]
    fn locate_d1_block(&self, value: PathDist) -> u32 {
        let (_, id) = self
            .d1_upper_index
            .range((value, 0u32)..)
            .next()
            .copied()
            .expect("should find a block");
        id
    }

    /// 当 block 内的节点数量大于 m 时，将 block 分裂为两个 block
    ///
    /// # Parameters
    ///
    /// - block_id: 要分裂的 block_id
    ///
    /// # Preconditions
    ///
    /// - block 必须存活
    /// - block 内的节点数量必须大于 m
    fn split_d1_block(&mut self, block_id: u32) {
        debug_assert!(self.blocks[block_id as usize].alive);
        debug_assert!(self.blocks[block_id as usize].len > self.m);

        let mut ids = self.collect_block_nodes(block_id);
        // 原论文用 BFPRT 找中位数划分；这里用 select_nth 将前 mid 个最小（按 value,key）放到 ids[0..mid]，平均 O(n)
        let mid = ids.len() / 2;
        debug_assert!(mid > 0, "split requires more than m nodes");
        let nodes = &self.nodes;
        ids.select_nth_unstable_by_key(mid - 1, |&id| {
            (nodes[id as usize].value, nodes[id as usize].key)
        });
        let left_upper = self.nodes[ids[mid - 1] as usize].value;
        let right_upper = self.blocks[block_id as usize].upper;

        // 分裂出来的两个 block，左边的复用原来的 block，但重新初始化
        self.d1_upper_index
            .remove(&(self.blocks[block_id as usize].upper, block_id));
        self.reset_block_nodes(block_id, &ids[..mid], left_upper);
        self.d1_upper_index.insert((left_upper, block_id));

        // 右边的新建一个
        let new_id = self.new_block(BlockSeq::D1, right_upper);
        self.reset_block_nodes(new_id, &ids[mid..], right_upper);
        self.insert_d1_block_after(block_id, new_id);
        self.d1_upper_index.insert((right_upper, new_id));
    }

    /// 收集 block 内的所有节点的 id
    ///
    /// # Parameters
    ///
    /// - block_id: 要收集的 block_id
    ///
    /// # Returns
    ///
    /// 返回 block 内的所有节点的 id
    fn collect_block_nodes(&self, block_id: u32) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.blocks[block_id as usize].len as usize);
        let mut p = self.blocks[block_id as usize].head;
        while p != NONE {
            out.push(p);
            p = self.nodes[p as usize].next;
        }
        out
    }

    /// 用某个新的节点列表和新的上界来重新初始化 block
    ///
    /// # Parameters
    ///
    /// - block_id: 要重新初始化的 block_id
    /// - ids: 新的节点列表
    /// - upper: 新的上界
    ///
    /// # Preconditions
    ///
    /// - block 必须存活
    fn reset_block_nodes(&mut self, block_id: u32, ids: &[u32], upper: PathDist) {
        debug_assert!(self.blocks[block_id as usize].alive);
        self.blocks[block_id as usize].head = NONE;
        self.blocks[block_id as usize].tail = NONE;
        self.blocks[block_id as usize].len = 0;
        self.blocks[block_id as usize].upper = upper;
        for &id in ids {
            self.nodes[id as usize].block_id = block_id;
            self.push_node_to_block_tail(block_id, id);
        }
    }

    /// 插入一个 block 到 d1 中。插入在 left_id 之后
    ///
    /// # Parameters
    ///
    /// - left_id: 要插入在哪个 block 的后面
    /// - new_id: 新插入的 block 的 id
    #[inline]
    fn insert_d1_block_after(&mut self, left_id: u32, new_id: u32) {
        let right = self.blocks[left_id as usize].next;
        self.blocks[new_id as usize].prev = left_id;
        self.blocks[new_id as usize].next = right;
        self.blocks[left_id as usize].next = new_id;
        if right != NONE {
            self.blocks[right as usize].prev = new_id;
        } else {
            self.d1_tail = new_id;
        }
    }

    /// 删除一个节点
    ///
    /// # Parameters
    ///
    /// - node_id: 要删除的节点 id
    fn delete_node(&mut self, node_id: u32) {
        debug_assert!(self.nodes[node_id as usize].alive);
        let key = self.nodes[node_id as usize].key;
        let block_id = self.nodes[node_id as usize].block_id;
        let prev = self.nodes[node_id as usize].prev;
        let next = self.nodes[node_id as usize].next;

        if prev != NONE {
            self.nodes[prev as usize].next = next;
        } else {
            self.blocks[block_id as usize].head = next;
        }
        if next != NONE {
            self.nodes[next as usize].prev = prev;
        } else {
            self.blocks[block_id as usize].tail = prev;
        }

        self.nodes[node_id as usize].prev = NONE;
        self.nodes[node_id as usize].next = NONE;
        self.nodes[node_id as usize].alive = false;
        debug_assert!(self.blocks[block_id as usize].len > 0);
        self.blocks[block_id as usize].len -= 1;

        debug_assert!(self.key_to_node[key as usize] != NONE);
        self.key_to_node[key as usize] = NONE;
        self.alive_count -= 1;

        if self.blocks[block_id as usize].len == 0
            && self.blocks[block_id as usize].upper != self.upper_bound_b
        {
            self.remove_empty_block(block_id);
        }
    }

    /// 删除一个空的 block
    ///
    /// # Parameters
    ///
    /// - block_id: 要删除的 block id
    fn remove_empty_block(&mut self, block_id: u32) {
        debug_assert!(self.blocks[block_id as usize].alive);
        let prev = self.blocks[block_id as usize].prev;
        let next = self.blocks[block_id as usize].next;
        match self.blocks[block_id as usize].seq {
            BlockSeq::D0 => {
                if prev != NONE {
                    self.blocks[prev as usize].next = next;
                } else {
                    self.d0_head = next;
                }
                if next != NONE {
                    self.blocks[next as usize].prev = prev;
                } else {
                    self.d0_tail = prev;
                }
            }
            BlockSeq::D1 => {
                self.d1_upper_index
                    .remove(&(self.blocks[block_id as usize].upper, block_id));
                if prev != NONE {
                    self.blocks[prev as usize].next = next;
                } else {
                    self.d1_head = next;
                }
                if next != NONE {
                    self.blocks[next as usize].prev = prev;
                } else {
                    self.d1_tail = prev;
                }
                if self.d1_head == NONE {
                    // 如果 d1 为空的话，则类似初始化时的那样，新建一个初始 block
                    let fresh = self.new_block(BlockSeq::D1, self.upper_bound_b);
                    self.d1_head = fresh;
                    self.d1_tail = fresh;
                    self.d1_upper_index.insert((self.upper_bound_b, fresh));
                }
            }
        }
        self.blocks[block_id as usize].alive = false;
        self.blocks[block_id as usize].prev = NONE;
        self.blocks[block_id as usize].next = NONE;
    }

    /// 清理 key_to_node 中的脏位，使其恢复为全 NONE 状态
    pub fn cleanup(&mut self) {
        for &k in self.dirty_keys.iter() {
            self.key_to_node[k as usize] = NONE;
        }
        self.dirty_keys.clear();
    }

    /// 获取 block 中的 value 最小值
    ///
    /// # Parameters
    ///
    /// - block_id: 要获取最小值的 block id
    ///
    /// # Returns
    ///
    /// 如果 block 为空，则返回 None
    #[inline]
    fn block_min_value(&self, block_id: u32) -> Option<PathDist> {
        let mut p = self.blocks[block_id as usize].head;
        let mut ans: Option<PathDist> = None;
        while p != NONE {
            ans = Some(match ans {
                Some(x) => min(x, self.nodes[p as usize].value),
                None => self.nodes[p as usize].value,
            });
            p = self.nodes[p as usize].next;
        }
        ans
    }

    /// 获取剩余数据结构中的最小值
    ///
    /// # Returns
    ///
    /// 如果剩余数据结构为空，则返回 None
    #[inline]
    fn min_remaining_value(&self) -> Option<PathDist> {
        let d0_min = if self.d0_head != NONE {
            self.block_min_value(self.d0_head)
        } else {
            None
        };
        let d1_min = if self.d1_head != NONE {
            self.block_min_value(self.d1_head)
        } else {
            None
        };
        match (d0_min, d1_min) {
            (Some(a), Some(b)) => Some(min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    impl BlockDs<'_> {
        fn sanity_check_links(&self) {
            let mut cur = self.d0_head;
            let mut prev = NONE;
            while cur != NONE {
                assert!(self.blocks[cur as usize].alive);
                assert_eq!(self.blocks[cur as usize].seq, BlockSeq::D0);
                assert_eq!(self.blocks[cur as usize].prev, prev);
                prev = cur;
                cur = self.blocks[cur as usize].next;
            }
            assert_eq!(prev, self.d0_tail);

            let mut cur = self.d1_head;
            let mut prev = NONE;
            while cur != NONE {
                assert!(self.blocks[cur as usize].alive);
                assert_eq!(self.blocks[cur as usize].seq, BlockSeq::D1);
                assert_eq!(self.blocks[cur as usize].prev, prev);
                prev = cur;
                cur = self.blocks[cur as usize].next;
            }
            assert_eq!(prev, self.d1_tail);
            assert!(!self.d1_upper_index.is_empty());
        }

        fn sanity_check_keys(&self) {
            for (k, &nid) in self.key_to_node.iter().enumerate() {
                if nid != NONE {
                    assert!(self.nodes[nid as usize].alive);
                    assert_eq!(self.nodes[nid as usize].key, k as u32);
                }
            }
            let alive_nodes = self.nodes.iter().filter(|n| n.alive).count();
            let map_count = self.key_to_node.iter().filter(|&&v| v != NONE).count();
            assert_eq!(alive_nodes, map_count);
        }

        fn is_empty_check(&self) -> bool {
            self.key_to_node.iter().all(|&v| v == NONE)
        }

        fn len_check(&self) -> usize {
            self.key_to_node.iter().filter(|&&v| v != NONE).count()
        }
    }

    struct NaiveModel {
        m: usize,
        upper: PathDist,
        map: FxHashMap<u32, PathDist>,
    }

    impl NaiveModel {
        fn new(m: usize, upper: PathDist) -> Self {
            Self {
                m: m.max(1),
                upper,
                map: FxHashMap::default(),
            }
        }

        fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        fn len(&self) -> usize {
            self.map.len()
        }

        fn min_value(&self) -> Option<PathDist> {
            self.map.values().copied().min()
        }

        fn insert(&mut self, key: u32, value: PathDist) {
            match self.map.get(&key).copied() {
                None => {
                    self.map.insert(key, value);
                }
                Some(old) => {
                    if old > value {
                        self.map.insert(key, value);
                    }
                }
            }
        }

        fn batch_prepend(&mut self, records: &[(u32, PathDist)]) {
            if records.is_empty() {
                return;
            }
            let mut per_key: FxHashMap<u32, PathDist> = FxHashMap::default();
            for &(k, v) in records {
                per_key
                    .entry(k)
                    .and_modify(|old| {
                        if v < *old {
                            *old = v;
                        }
                    })
                    .or_insert(v);
            }
            for (k, v) in per_key {
                self.map.insert(k, v);
            }
        }

        fn pull(&mut self) -> (PathDist, Vec<u32>) {
            let len_before = self.map.len();
            if len_before == 0 {
                return (self.upper, vec![]);
            }

            let mut items: Vec<(PathDist, u32)> = self.map.iter().map(|(&k, &v)| (v, k)).collect();
            items.sort_unstable_by_key(|&(v, k)| (v, k));

            let take = items.len().min(self.m);
            let picked = &items[..take];
            for &(_, k) in picked {
                self.map.remove(&k);
            }

            let boundary = if len_before <= self.m {
                self.upper
            } else {
                items[take].0
            };

            let keys = picked.iter().map(|&(_, k)| k).collect();
            (boundary, keys)
        }
    }

    const N: usize = 100;

    fn make_ds(m: usize, upper: PathDist, buf: &mut [u32]) -> BlockDs<'_> {
        BlockDs::new(m, upper, buf)
    }

    #[test]
    fn insert_update_then_pull_returns_m_smallest() {
        let mut buf = vec![NONE; N];
        let mut ds = make_ds(2, PathDist::scalar_upper(999), &mut buf);
        ds.insert(1, PathDist::from_dis(30, 1));
        ds.insert(2, PathDist::from_dis(10, 2));
        ds.insert(3, PathDist::from_dis(20, 3));
        ds.insert(2, PathDist::from_dis(8, 2));
        ds.insert(1, PathDist::from_dis(40, 1));

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![2, 3]);
        assert_eq!(boundary, PathDist::from_dis(30, 1));
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_keeps_smallest_per_key() {
        let mut buf = vec![NONE; N];
        let mut ds = make_ds(4, PathDist::scalar_upper(1000), &mut buf);
        ds.insert(10, PathDist::from_dis(100, 10));
        ds.insert(11, PathDist::from_dis(110, 11));
        let records = [
            (2u32, PathDist::from_dis(3, 2)),
            (1, PathDist::from_dis(4, 1)),
            (10, PathDist::from_dis(90, 10)),
        ];
        let mut pool = vec![PathDist::MAX; N];
        ds.batch_prepend(&records, &mut pool);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2, 10, 11]);
        assert_eq!(boundary, PathDist::scalar_upper(1000));
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_b_when_structure_becomes_empty() {
        let mut buf = vec![NONE; N];
        let mut ds = make_ds(2, PathDist::scalar_upper(777), &mut buf);
        ds.insert(1, PathDist::from_dis(10, 1));
        ds.insert(2, PathDist::from_dis(20, 2));

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, PathDist::scalar_upper(777));
        assert!(ds.is_empty_check());
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_smallest_remaining_value() {
        let mut buf = vec![NONE; N];
        let mut ds = make_ds(2, PathDist::scalar_upper(500), &mut buf);
        ds.insert(1, PathDist::from_dis(10, 1));
        ds.insert(2, PathDist::from_dis(20, 2));
        ds.insert(3, PathDist::from_dis(30, 3));
        ds.insert(4, PathDist::from_dis(40, 4));

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, PathDist::from_dis(30, 3));
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn randomized_operations_match_naive_model_under_preconditions() {
        let m = 4usize;
        let upper = 1000u64;
        let upper_pd = PathDist::scalar_upper(upper);
        let mut buf = vec![NONE; 20];
        let mut ds = make_ds(m, upper_pd, &mut buf);
        let mut model = NaiveModel::new(m, upper_pd);

        let mut seed: u64 = 0x1234_abcd_5678_ef01;
        let mut next_u64 = || {
            seed = seed
                .wrapping_mul(6364136223846793005u64)
                .wrapping_add(1442695040888963407u64);
            seed
        };

        let key_space = 0u32..16u32;
        let mut history: Vec<String> = Vec::new();
        for _step in 0..160 {
            let op = next_u64() % 100;
            if op < 55 {
                let key = key_space.start + (next_u64() as u32 % key_space.len() as u32);
                let value = 1 + (next_u64() % upper.max(1));
                let pd = PathDist::from_dis(value, key as usize);
                history.push(format!("insert({}, {})", key, value));
                ds.insert(key, pd);
                model.insert(key, pd);
                ds.sanity_check_links();
                ds.sanity_check_keys();
            } else if op < 82 {
                let cur_min_dis = model.min_value().map(|p| p.dis()).unwrap_or(upper + 1);
                if cur_min_dis <= 1 {
                    history.push("skip(batch_prepend)".to_string());
                    continue;
                }

                let cnt = 1usize + (next_u64() as usize % 5);
                let mut records: Vec<(u32, PathDist)> = Vec::with_capacity(cnt);
                for _ in 0..cnt {
                    let key = key_space.start + (next_u64() as u32 % key_space.len() as u32);
                    let value = 1 + (next_u64() % (cur_min_dis - 1).max(1));
                    let value = value.min(cur_min_dis - 1).max(1);
                    records.push((key, PathDist::from_dis(value, key as usize)));
                }

                history.push(format!("batch_prepend({:?})", records));
                let mut pool = vec![PathDist::MAX; 20];
                ds.batch_prepend(&records, &mut pool);
                model.batch_prepend(&records);
                ds.sanity_check_links();
                ds.sanity_check_keys();
            } else {
                history.push("pull()".to_string());
                let PullResult { boundary, mut keys } = ds.pull();
                let (b2, mut keys2) = model.pull();
                keys.sort_unstable();
                keys2.sort_unstable();

                if boundary != b2 || keys != keys2 {
                    panic!(
                        "random mismatch: boundary ds={:?} model={:?} keys ds={:?} model={:?}\nlast_ops={:?}",
                        boundary,
                        b2,
                        keys,
                        keys2,
                        history.iter().rev().take(12).cloned().collect::<Vec<_>>()
                    );
                }

                ds.sanity_check_links();
                ds.sanity_check_keys();
                assert_eq!(ds.is_empty_check(), model.is_empty());
                assert_eq!(ds.len_check(), model.len());
            }
        }
    }
}
