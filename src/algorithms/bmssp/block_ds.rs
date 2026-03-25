use std::cmp::min;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullResult {
    pub boundary: u64,
    pub keys: Vec<usize>,
}

#[derive(Debug, Clone)]
struct Node {
    key: usize,
    value: u64,
    prev: Option<usize>,
    next: Option<usize>,
    block_id: usize,
    alive: bool,
}

#[derive(Debug, Clone)]
enum BlockSeq {
    D0,
    D1,
}

#[derive(Debug, Clone)]
struct Block {
    /// block 属于哪个链表
    seq: BlockSeq,
    /// 前一个 block_id
    prev: Option<usize>,
    /// 后一个 block_id
    next: Option<usize>,
    /// 头部 node_id
    head: Option<usize>,
    /// 尾部 node_id
    tail: Option<usize>,
    /// block 内的节点数量
    len: usize,
    /// block 的上界
    upper: u64,
    /// block 是否存活
    alive: bool,
}

impl Block {
    fn new(seq: BlockSeq, upper: u64) -> Self {
        Self {
            seq,
            prev: None,
            next: None,
            head: None,
            tail: None,
            len: 0,
            upper,
            alive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockDs {
    m: usize,
    upper_bound_b: u64,
    key_to_node: HashMap<usize, usize>,
    // block 和 node 被删除时，会从 key_to_node 和链表里真实删除
    // 但仍留在 nodes 和 blocks 中
    nodes: Vec<Node>,
    blocks: Vec<Block>,
    d0_head: Option<usize>,
    d0_tail: Option<usize>,
    d1_head: Option<usize>,
    d1_tail: Option<usize>,
    d1_upper_index: BTreeSet<(u64, usize)>,
}

impl BlockDs {
    /// 创建一个新的 BlockDs
    ///
    /// # Parameters
    ///
    /// - m: 每个 block 的最大节点数量，也是 pull 操作返回的节点数量
    /// - upper_bound_b: 数据结构中 value 的上界
    pub fn new(m: usize, upper_bound_b: u64) -> Self {
        let mut ds = Self {
            m: m.max(1),
            upper_bound_b,
            // TODO: 使用内存池优化
            key_to_node: HashMap::new(),
            nodes: Vec::new(),
            blocks: Vec::new(),
            d0_head: None,
            d0_tail: None,
            d1_head: None,
            d1_tail: None,
            d1_upper_index: BTreeSet::new(),
        };
        let first = ds.new_block(BlockSeq::D1, upper_bound_b);
        ds.d1_head = Some(first);
        ds.d1_tail = Some(first);
        ds.d1_upper_index.insert((upper_bound_b, first));
        ds
    }

    /// 判断当前数据结构是否为空
    pub fn is_empty(&self) -> bool {
        self.key_to_node.is_empty()
    }

    /// 获取当前数据结构中节点的数量
    pub fn len(&self) -> usize {
        self.key_to_node.len()
    }

    /// 插入一个节点，如果节点已经存在，则对其值取 min
    ///
    /// # Parameters
    ///
    /// - key: 要插入的节点的 key
    /// - value: 要插入的节点的 value
    ///
    /// # Panics
    ///
    /// 如果 value 大于当前 block 的上界，则 panic
    pub fn insert(&mut self, key: usize, value: u64) {
        if value > self.upper_bound_b {
            panic!("value is greater than upper bound");
        }
        if let Some(&old_id) = self.key_to_node.get(&key) {
            if self.nodes[old_id].value > value {
                self.delete_node(old_id);
            } else {
                return;
            }
        }

        let block_id = self.locate_d1_block(value);
        let node_id = self.new_node(key, value, block_id);
        self.key_to_node.insert(key, node_id);
        self.push_node_to_block_tail(block_id, node_id);

        if self.blocks[block_id].len > self.m {
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
    /// - records: 要插入的节点列表。
    ///
    /// # Preconditions
    ///
    /// - 所有的 value 必须严格小于当前数据结构中已有的 value 的最小值。该函数不会对这个条件进行任何的检查（包括 debug assertions）。
    pub fn batch_prepend(&mut self, records: &[(usize, u64)]) {
        if records.is_empty() {
            return;
        }

        // TODO：复用内存池来砍掉这块的内存分配
        // 对 records 去重，并把已经存在于数据结构中的点删掉
        let mut insert_records: HashMap<usize, u64> = HashMap::new();
        for (key, value) in records.iter().copied() {
            if let Some(&old_node_id) = self.key_to_node.get(&key) {
                self.delete_node(old_node_id);
            }
            insert_records
                .entry(key)
                .and_modify(|v| *v = (*v).min(value))
                .or_insert(value);
        }

        let mut insert_records = insert_records.into_iter().collect::<Vec<_>>();

        // 论文中使用 BFPRT 来进行递归划分
        // 考虑到 BFPRT 的常数可能较大，这里直接使用排序
        insert_records.sort_unstable_by_key(|&(k, v)| (v, k));

        // 如果 records 中存在相同的 key，则删除之前的
        for (key, _) in insert_records.iter() {
            if let Some(&old_id) = self.key_to_node.get(key) {
                self.delete_node(old_id);
            }
        }

        let mut cursor = 0usize;
        let mut block_ids = Vec::<usize>::new();
        while cursor < insert_records.len() {
            let end = min(cursor + self.m, insert_records.len());
            // [cursor, end) 这个区间为一块
            let slice = &insert_records[cursor..end];
            let upper = slice
                .last()
                .map(|&(_, v)| v)
                .expect("slice cannot be empty");
            let block_id = self.new_block(BlockSeq::D0, upper);
            block_ids.push(block_id);
            for &(k, v) in slice {
                let node_id = self.new_node(k, v, block_id);
                self.key_to_node.insert(k, node_id);
                self.push_node_to_block_tail(block_id, node_id);
            }
            cursor = end;
        }

        for block_id in block_ids.into_iter().rev() {
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
        let mut cand_ids = Vec::<usize>::new();

        let mut collect = |head_block_id: Option<usize>| {
            let mut cur = head_block_id;
            let mut collected_count = 0;
            while let Some(bid) = cur {
                if collected_count >= self.m {
                    break;
                }
                let mut p = self.blocks[bid].head;
                while let Some(id) = p {
                    cand_ids.push(id);
                    collected_count += 1;
                    p = self.nodes[id].next;
                }
                cur = self.blocks[bid].next;
            }
        };

        collect(self.d0_head);
        collect(self.d1_head);

        // pending_boundary 是 None 的话说明还不能确定，等把节点删完之后再求数据结构中剩余的 value 最小值
        let (result_ids, pending_boundary) = if cand_ids.len() < self.m {
            // 如果是小于 m 的话，说明两个链表都遍历完了，上界就是整个数据结构的上界
            (cand_ids, Some(self.upper_bound_b))
        } else if cand_ids.len() == self.m {
            (cand_ids, None)
        } else {
            cand_ids.sort_unstable_by_key(|&id| (self.nodes[id].value, self.nodes[id].key));
            (cand_ids.into_iter().take(self.m).collect(), None)
        };

        for &id in &result_ids {
            debug_assert!(self.nodes[id].alive);
            self.delete_node(id);
        }

        let boundary =
            pending_boundary.unwrap_or(self.min_remaining_value().unwrap_or(self.upper_bound_b));

        let keys = result_ids
            .iter()
            .map(|&id| self.nodes[id].key)
            .collect::<Vec<_>>();
        PullResult { boundary, keys }
    }

    fn new_block(&mut self, seq: BlockSeq, upper: u64) -> usize {
        let id = self.blocks.len();
        self.blocks.push(Block::new(seq, upper));
        id
    }

    fn new_node(&mut self, key: usize, value: u64, block_id: usize) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            key,
            value,
            prev: None,
            next: None,
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
    fn prepend_d0_block(&mut self, block_id: usize) {
        let old = self.d0_head;
        self.blocks[block_id].prev = None;
        self.blocks[block_id].next = old;
        if let Some(h) = old {
            self.blocks[h].prev = Some(block_id);
        } else {
            self.d0_tail = Some(block_id);
        }
        self.d0_head = Some(block_id);
    }

    /// 把某个节点插入到某个 block 的尾部
    ///
    /// # Parameters
    ///
    /// - block_id: 要插入的 block_id
    /// - node_id: 要插入的节点 id
    fn push_node_to_block_tail(&mut self, block_id: usize, node_id: usize) {
        let tail = self.blocks[block_id].tail;
        self.nodes[node_id].prev = tail;
        self.nodes[node_id].next = None;
        if let Some(t) = tail {
            self.nodes[t].next = Some(node_id);
        } else {
            self.blocks[block_id].head = Some(node_id);
        }
        self.blocks[block_id].tail = Some(node_id);
        self.blocks[block_id].len += 1;
    }

    /// 查找第一个上界大于等于 value 的 block
    ///
    /// # Returns
    ///
    /// 返回 block_id
    fn locate_d1_block(&self, value: u64) -> usize {
        let (_, id) = self
            .d1_upper_index
            .range((value, 0usize)..)
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
    fn split_d1_block(&mut self, block_id: usize) {
        debug_assert!(self.blocks[block_id].alive);
        debug_assert!(self.blocks[block_id].len > self.m);

        let mut ids = self.collect_block_nodes(block_id);
        // 原论文中是需要用 BFPRT 找中位数的。考虑到 BFPRT 的常数可能较大，这里改用排序
        ids.sort_unstable_by_key(|&id| (self.nodes[id].value, self.nodes[id].key));
        let mid = ids.len() / 2;
        let left_upper = self.nodes[ids[mid - 1]].value;
        let right_upper = self.blocks[block_id].upper;

        // 分裂出来的两个 block，左边的复用原来的 block，但重新初始化
        self.d1_upper_index
            .remove(&(self.blocks[block_id].upper, block_id));
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
    fn collect_block_nodes(&self, block_id: usize) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.blocks[block_id].len);
        let mut p = self.blocks[block_id].head;
        while let Some(id) = p {
            out.push(id);
            p = self.nodes[id].next;
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
    fn reset_block_nodes(&mut self, block_id: usize, ids: &[usize], upper: u64) {
        debug_assert!(self.blocks[block_id].alive);
        self.blocks[block_id].head = None;
        self.blocks[block_id].tail = None;
        self.blocks[block_id].len = 0;
        self.blocks[block_id].upper = upper;
        for &id in ids {
            self.nodes[id].block_id = block_id;
            self.push_node_to_block_tail(block_id, id);
        }
    }

    /// 插入一个 block 到 d1 中。插入在 left_id 之后
    ///
    /// # Parameters
    ///
    /// - left_id: 要插入在哪个 block 的后面
    /// - new_id: 新插入的 block 的 id
    fn insert_d1_block_after(&mut self, left_id: usize, new_id: usize) {
        let right = self.blocks[left_id].next;
        self.blocks[new_id].prev = Some(left_id);
        self.blocks[new_id].next = right;
        self.blocks[left_id].next = Some(new_id);
        if let Some(r) = right {
            self.blocks[r].prev = Some(new_id);
        } else {
            self.d1_tail = Some(new_id);
        }
    }

    /// 删除一个节点
    ///
    /// # Parameters
    ///
    /// - node_id: 要删除的节点 id
    fn delete_node(&mut self, node_id: usize) {
        debug_assert!(self.nodes[node_id].alive);
        let key = self.nodes[node_id].key;
        let block_id = self.nodes[node_id].block_id;
        let prev = self.nodes[node_id].prev;
        let next = self.nodes[node_id].next;

        if let Some(p) = prev {
            self.nodes[p].next = next;
        } else {
            self.blocks[block_id].head = next;
        }
        if let Some(n) = next {
            self.nodes[n].prev = prev;
        } else {
            self.blocks[block_id].tail = prev;
        }

        self.nodes[node_id].prev = None;
        self.nodes[node_id].next = None;
        self.nodes[node_id].alive = false;
        debug_assert!(self.blocks[block_id].len > 0);
        self.blocks[block_id].len -= 1;

        debug_assert!(self.key_to_node.contains_key(&key));
        self.key_to_node.remove(&key);

        if self.blocks[block_id].len == 0 {
            self.remove_empty_block(block_id);
        }
    }

    /// 删除一个空的 block
    ///
    /// # Parameters
    ///
    /// - block_id: 要删除的 block id
    fn remove_empty_block(&mut self, block_id: usize) {
        debug_assert!(self.blocks[block_id].alive);
        let prev = self.blocks[block_id].prev;
        let next = self.blocks[block_id].next;
        match self.blocks[block_id].seq {
            BlockSeq::D0 => {
                if let Some(p) = prev {
                    self.blocks[p].next = next;
                } else {
                    self.d0_head = next;
                }
                if let Some(n) = next {
                    self.blocks[n].prev = prev;
                } else {
                    self.d0_tail = prev;
                }
            }
            BlockSeq::D1 => {
                self.d1_upper_index
                    .remove(&(self.blocks[block_id].upper, block_id));
                if let Some(p) = prev {
                    self.blocks[p].next = next;
                } else {
                    self.d1_head = next;
                }
                if let Some(n) = next {
                    self.blocks[n].prev = prev;
                } else {
                    self.d1_tail = prev;
                }
                if self.d1_head.is_none() {
                    // 如果 d1 为空的话，则类似初始化时的那样，新建一个初始 block
                    let fresh = self.new_block(BlockSeq::D1, self.upper_bound_b);
                    self.d1_head = Some(fresh);
                    self.d1_tail = Some(fresh);
                    self.d1_upper_index.insert((self.upper_bound_b, fresh));
                }
            }
        }
        self.blocks[block_id].alive = false;
        self.blocks[block_id].prev = None;
        self.blocks[block_id].next = None;
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
    fn block_min_value(&self, block_id: usize) -> Option<u64> {
        let mut p = self.blocks[block_id].head;
        let mut ans: Option<u64> = None;
        while let Some(id) = p {
            ans = Some(match ans {
                Some(x) => min(x, self.nodes[id].value),
                None => self.nodes[id].value,
            });
            p = self.nodes[id].next;
        }
        ans
    }

    /// 获取剩余数据结构中的最小值
    ///
    /// # Returns
    ///
    /// 如果剩余数据结构为空，则返回 None
    fn min_remaining_value(&self) -> Option<u64> {
        let d0_min = self.d0_head.and_then(|id| self.block_min_value(id));
        let d1_min = self.d1_head.and_then(|id| self.block_min_value(id));
        match (d0_min, d1_min) {
            (Some(a), Some(b)) => Some(min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    #[cfg(test)]
    /// 测试辅助：检查 `d0`/`d1` 两条 block 链表的结构不变式是否成立。
    ///
    /// 具体检查：
    /// - 从 `d0_head` 沿着 `next` 遍历到末尾，每个经过的 block：
    ///   - 必须 `alive == true`
    ///   - `seq` 必须为 `BlockSeq::D0`
    ///   - `prev` 必须等于遍历到的前一个 block id（链表的前向/后向指针一致）
    /// - 遍历结束后，最后一个访问到的 block id 必须等于 `d0_tail`
    /// - 对 `d1_head`/`d1_tail` 做同样的检查，并额外断言 `d1_upper_index` 非空
    fn sanity_check_links(&self) {
        let mut cur = self.d0_head;
        let mut prev = None;
        while let Some(id) = cur {
            assert!(self.blocks[id].alive);
            assert!(matches!(self.blocks[id].seq, BlockSeq::D0));
            assert_eq!(self.blocks[id].prev, prev);
            prev = cur;
            cur = self.blocks[id].next;
        }
        assert_eq!(prev, self.d0_tail);

        let mut cur = self.d1_head;
        let mut prev = None;
        while let Some(id) = cur {
            assert!(self.blocks[id].alive);
            assert!(matches!(self.blocks[id].seq, BlockSeq::D1));
            assert_eq!(self.blocks[id].prev, prev);
            prev = cur;
            cur = self.blocks[id].next;
        }
        assert_eq!(prev, self.d1_tail);
        assert!(!self.d1_upper_index.is_empty());
    }

    #[cfg(test)]
    /// 测试辅助：检查 `key_to_node` 与 `nodes` 的一致性。
    ///
    /// 它验证：
    /// - `key_to_node` 中每个条目都指向 `alive == true` 的 node，并且 node 的 `key`
    ///   与 map 的 key 完全一致；
    /// - `nodes` 数组中 `alive == true` 的节点数量，必须与 `key_to_node` 的条目数一致
    ///   （确保“删除节点时从 map 移除 + alive 置为 false”这两步不会漏掉）。
    fn sanity_check_keys(&self) {
        for (&k, &nid) in &self.key_to_node {
            assert!(self.nodes[nid].alive);
            assert_eq!(self.nodes[nid].key, k);
        }
        let alive_nodes = self.nodes.iter().filter(|n| n.alive).count();
        assert_eq!(alive_nodes, self.key_to_node.len());
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::bmssp::block_ds::PullResult;

    use std::collections::HashMap;

    use super::BlockDs;

    struct NaiveModel {
        m: usize,
        upper: u64,
        map: HashMap<usize, u64>,
    }

    impl NaiveModel {
        fn new(m: usize, upper: u64) -> Self {
            Self {
                m: m.max(1),
                upper,
                map: HashMap::new(),
            }
        }

        fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        fn len(&self) -> usize {
            self.map.len()
        }

        fn min_value(&self) -> Option<u64> {
            self.map.values().copied().min()
        }

        fn insert(&mut self, key: usize, value: u64) {
            assert!(value <= self.upper, "test assumes value <= upper bound");
            match self.map.get(&key).copied() {
                None => {
                    self.map.insert(key, value);
                }
                Some(old) => {
                    // 和 BlockDs.insert 的实现一致：只会把值更新为更小的那个
                    if old > value {
                        self.map.insert(key, value);
                    }
                }
            }
        }

        fn batch_prepend(&mut self, records: &[(usize, u64)]) {
            if records.is_empty() {
                return;
            }

            // 和 BlockDs.batch_prepend 一致：对同一个 key，在 records 内取最小值，
            // 然后覆盖掉旧值（测试中会保证 records 的值严格小于当前全局最小值）。
            let mut per_key: HashMap<usize, u64> = HashMap::new();
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

        fn pull(&mut self) -> (u64, Vec<usize>) {
            let len_before = self.map.len();
            if len_before == 0 {
                return (self.upper, vec![]);
            }

            let mut items: Vec<(u64, usize)> = self.map.iter().map(|(&k, &v)| (v, k)).collect();
            items.sort_unstable_by_key(|&(v, k)| (v, k));

            let take = items.len().min(self.m);
            let picked = &items[..take];
            for &(_, k) in picked {
                self.map.remove(&k);
            }

            let boundary = if len_before <= self.m {
                self.upper
            } else {
                // 取走 m 个后，剩余的最小值就是 items[take] 的 value
                items[take].0
            };

            let keys = picked.iter().map(|&(_, k)| k).collect();
            (boundary, keys)
        }
    }

    #[test]
    fn insert_update_then_pull_returns_m_smallest() {
        let mut ds = BlockDs::new(2, 999);
        ds.insert(1, 30);
        ds.insert(2, 10);
        ds.insert(3, 20);
        ds.insert(2, 8);
        ds.insert(1, 40);

        // 1->30, 2->8, 3->20

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![2, 3]);
        assert_eq!(boundary, 30);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_keeps_smallest_per_key() {
        let mut ds = BlockDs::new(4, 1000);
        ds.insert(10, 100);
        ds.insert(11, 110);
        let records = [(2, 3), (1, 4), (10, 90)];
        ds.batch_prepend(&records);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2, 10, 11]);
        assert_eq!(boundary, 1000);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_b_when_structure_becomes_empty() {
        let mut ds = BlockDs::new(2, 777);
        ds.insert(1, 10);
        ds.insert(2, 20);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 777);
        assert!(ds.is_empty());
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_smallest_remaining_value() {
        let mut ds = BlockDs::new(2, 500);
        ds.insert(1, 10);
        ds.insert(2, 20);
        ds.insert(3, 30);
        ds.insert(4, 40);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 30);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_does_not_mistake_prefix_for_all_elements() {
        let mut ds = BlockDs::new(2, 1000);
        ds.insert(1, 10);
        ds.insert(2, 20);
        ds.insert(3, 30);
        ds.insert(4, 40);
        ds.insert(5, 50);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 30);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![3, 4]);
        assert_eq!(boundary, 50);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_on_empty_returns_upper_and_no_keys() {
        let mut ds = BlockDs::new(2, 999);
        assert!(ds.is_empty());

        let PullResult { keys, boundary } = ds.pull();
        assert!(keys.is_empty());
        assert_eq!(boundary, 999);

        ds.sanity_check_links();
        ds.sanity_check_keys();
        assert!(ds.is_empty());
    }

    #[test]
    #[should_panic(expected = "value is greater than upper bound")]
    fn insert_panics_when_value_greater_than_upper() {
        let mut ds = BlockDs::new(2, 10);
        ds.insert(1, 11);
    }

    #[test]
    fn insert_updates_existing_key_to_smaller_value_only() {
        let mut ds = BlockDs::new(2, 1000);
        // 初始：key1=30, key2=20, key3=25 => 最小两者应是 key2, key3
        ds.insert(1, 30);
        ds.insert(2, 20);
        ds.insert(3, 25);

        // 把 key1 更新到更小，应该进入最小两者
        ds.insert(1, 10);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 25);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn insert_does_not_overwrite_with_larger_value() {
        let mut ds = BlockDs::new(2, 1000);
        // 初始：key1=10, key2=20, key3=25 => 最小两者 key1, key2；边界=25
        ds.insert(1, 10);
        ds.insert(2, 20);
        ds.insert(3, 25);

        // 用更大值“更新”key1：实现语义是取 min，因此应保持 key1=10
        ds.insert(1, 30);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 25);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_handles_duplicate_keys_and_overwrites() {
        let mut ds = BlockDs::new(2, 1000);
        ds.insert(1, 50);
        ds.insert(2, 60);
        ds.insert(3, 70);
        // 当前全局最小值是 50；records 内所有值必须严格小于 50

        let records = [(2, 40), (1, 10), (2, 35), (4, 30)];
        ds.batch_prepend(&records);

        // 先确认 batch_prepend 语义本身是否正确（value 取 records 内最小值，key 对应值覆盖）。
        // 如果这一步就不符合预期，则是 batch_prepend 插入逻辑有问题；
        // 如果这一步符合，但 pull 不符合，则问题更可能出在 pull 的候选集/边界计算上。
        let mut actual: HashMap<usize, u64> = HashMap::new();
        for (&k, &nid) in &ds.key_to_node {
            actual.insert(k, ds.nodes[nid].value);
        }
        let expected: HashMap<usize, u64> = [(1, 10u64), (2, 35u64), (3, 70u64), (4, 30u64)]
            .into_iter()
            .collect();
        assert_eq!(actual, expected);

        // 更新后：key1=10, key4=30, key2=35, key3=70
        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 4]);
        assert_eq!(boundary, 35);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_can_overwrite_keys_in_d0_and_d1() {
        let mut ds = BlockDs::new(2, 1000);
        // 先放入 D1
        ds.insert(1, 100);
        ds.insert(2, 200);

        // batch1：覆盖 key1，并生成 D0
        ds.batch_prepend(&[(3, 50), (1, 80)]);
        // 此时全局最小值应是 50

        // batch2：值必须严格小于当前最小值 50，同时覆盖 key2（在 D1）与 key3（在 D0）
        ds.batch_prepend(&[(2, 40), (3, 45)]);

        // 更新后：key2=40, key3=45, key1=80
        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![2, 3]);
        assert_eq!(boundary, 80);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_sorts_candidates_when_d0_plus_d1_exceeds_m() {
        let mut ds = BlockDs::new(3, 1000);
        // D1：3 个节点（m=3，且当前 D0 为空）
        ds.insert(1, 30);
        ds.insert(2, 40);
        ds.insert(3, 50);

        // batch_prepend：插入 2 个节点到 D0（严格小于当前最小值 30）
        ds.batch_prepend(&[(4, 10), (5, 20)]);

        // 全局最小三者是 10(key4), 20(key5), 30(key1)
        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 4, 5]);
        // 删除 3 个后剩余最小值是 40
        assert_eq!(boundary, 40);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_tie_breaks_by_smaller_key_on_equal_values() {
        let mut ds = BlockDs::new(2, 1000);
        // 让 D1 至少分裂出多个 block：插入 4 个 value 都相同的元素
        ds.insert(1, 10);
        ds.insert(2, 10);
        ds.insert(4, 10);
        ds.insert(5, 10);

        // D0：插入一个严格更小的 value，保证 pull 会同时看到 D0 与 D1，
        // 并触发对候选的显式排序（从而确定 tie 的选择）。
        ds.batch_prepend(&[(3, 1)]);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        // 全局排序：(1,key3), (10,key1), (10,key2), (10,key4), (10,key5)
        // m=2 => key3 和 key1
        assert_eq!(keys, vec![1, 3]);
        // 删除后剩余最小值仍为 10
        assert_eq!(boundary, 10);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn d1_split_then_pull_matches_naive_order() {
        let mut ds = BlockDs::new(3, 1000);
        // 插入 4 个节点会触发一次 D1 分裂
        ds.insert(1, 50);
        ds.insert(2, 10);
        ds.insert(3, 40);
        ds.insert(4, 20);

        // 当前值：{key2:10, key4:20, key3:40, key1:50}
        // m=3 => 拉取 10,20,40，边界=50
        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![2, 3, 4]);
        assert_eq!(boundary, 50);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn d1_becomes_empty_and_reinsert_still_works() {
        let mut ds = BlockDs::new(2, 777);
        ds.insert(1, 10);
        ds.insert(2, 20);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![1, 2]);
        assert_eq!(boundary, 777);
        assert!(ds.is_empty());

        // 验证删除最后一个 D1 节点后，后续还能正常工作
        ds.insert(3, 5);
        let PullResult { keys, boundary } = ds.pull();
        assert_eq!(keys, vec![3]);
        assert_eq!(boundary, 777);

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_with_m_eq_1() {
        let mut ds = BlockDs::new(1, 999);
        ds.insert(1, 5);
        ds.insert(2, 3);
        ds.insert(3, 7);

        let PullResult { mut keys, boundary } = ds.pull();
        keys.sort_unstable();
        assert_eq!(keys, vec![2]);
        assert_eq!(boundary, 5);
        ds.sanity_check_links();
        ds.sanity_check_keys();

        let PullResult { keys, boundary } = ds.pull();
        assert_eq!(keys, vec![1]);
        assert_eq!(boundary, 7);
        ds.sanity_check_links();
        ds.sanity_check_keys();

        let PullResult { keys, boundary } = ds.pull();
        assert_eq!(keys, vec![3]);
        assert_eq!(boundary, 999);
        assert!(ds.is_empty());
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn m_zero_is_treated_as_one() {
        let mut ds = BlockDs::new(0, 500);
        ds.insert(1, 10);
        let PullResult { keys, boundary } = ds.pull();
        assert_eq!(keys, vec![1]);
        assert_eq!(boundary, 500);
        assert!(ds.is_empty());

        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_empty_records_is_noop() {
        let mut ds = BlockDs::new(2, 1000);
        ds.insert(1, 10);
        ds.batch_prepend(&[]);
        assert_eq!(ds.len(), 1);

        let PullResult { keys, boundary } = ds.pull();
        assert_eq!(keys, vec![1]);
        assert_eq!(boundary, 1000);
        assert!(ds.is_empty());
    }

    #[test]
    fn randomized_operations_match_naive_model_under_preconditions() {
        // 说明：batch_prepend 的测试严格满足“records 内所有 value 严格小于当前全局最小值”的前置条件，
        // 因为 BlockDs 的实现没有在 batch_prepend 内检查该条件。
        let m = 4usize;
        let upper = 1000u64;
        let mut ds = BlockDs::new(m, upper);
        let mut model = NaiveModel::new(m, upper);

        let mut seed: u64 = 0x1234_abcd_5678_ef01;
        let mut next_u64 = || {
            // LCG：足够用来生成确定性的测试数据。
            seed = seed
                .wrapping_mul(6364136223846793005u64)
                .wrapping_add(1442695040888963407u64);
            seed
        };

        let key_space = 0usize..16usize;
        let mut history: Vec<String> = Vec::new();
        for _step in 0..160 {
            let op = next_u64() % 100;
            if op < 55 {
                // insert
                let key = key_space.start + (next_u64() as usize % key_space.len());
                let value = 1 + (next_u64() % upper.max(1));
                history.push(format!("insert({}, {})", key, value));
                ds.insert(key, value);
                model.insert(key, value);
                ds.sanity_check_links();
                ds.sanity_check_keys();
            } else if op < 82 {
                // batch_prepend
                let cur_min = model.min_value().unwrap_or(upper + 1);
                if cur_min <= 1 {
                    history.push("skip(batch_prepend)".to_string());
                    continue;
                }

                let cnt = 1usize + (next_u64() as usize % 5);
                let mut records: Vec<(usize, u64)> = Vec::with_capacity(cnt);
                for _ in 0..cnt {
                    let key = key_space.start + (next_u64() as usize % key_space.len());
                    let value = 1 + (next_u64() % (cur_min - 1).max(1));
                    // 保证严格小于 cur_min
                    let value = value.min(cur_min - 1).max(1);
                    records.push((key, value));
                }

                history.push(format!("batch_prepend({:?})", records));
                ds.batch_prepend(&records);
                model.batch_prepend(&records);
                ds.sanity_check_links();
                ds.sanity_check_keys();
            } else {
                // pull
                let snapshot: Vec<(usize, u64)> = model.map.iter().map(|(&k, &v)| (k, v)).collect();
                let mut snapshot_sorted = snapshot.clone();
                snapshot_sorted.sort_unstable_by_key(|&(k, v)| (v, k));
                history.push("pull()".to_string());
                let PullResult { boundary, mut keys } = ds.pull();
                let (b2, mut keys2) = model.pull();
                keys.sort_unstable();
                keys2.sort_unstable();

                if boundary != b2 || keys != keys2 {
                    // 失败时打印足够定位信息：step 历史 + pull 前模型快照。
                    panic!(
                        "random mismatch: boundary ds={} model={} keys ds={:?} model={:?}\nmodel_len_before={} snapshot_sorted={:?}\nlast_ops={:?}",
                        boundary,
                        b2,
                        keys,
                        keys2,
                        snapshot_sorted.len(),
                        snapshot_sorted,
                        history.iter().rev().take(12).cloned().collect::<Vec<_>>()
                    );
                }

                ds.sanity_check_links();
                ds.sanity_check_keys();
                assert_eq!(ds.is_empty(), model.is_empty());
                assert_eq!(ds.len(), model.len());
            }
        }
    }
}
