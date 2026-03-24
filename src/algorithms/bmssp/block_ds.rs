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
    _max_insertions_n: usize,
    key_to_node: HashMap<usize, usize>,
    nodes: Vec<Node>,
    blocks: Vec<Block>,
    d0_head: Option<usize>,
    d0_tail: Option<usize>,
    d1_head: Option<usize>,
    d1_tail: Option<usize>,
    d1_upper_index: BTreeSet<(u64, usize)>,
}

impl BlockDs {
    pub fn new(max_insertions_n: usize, m: usize, upper_bound_b: u64) -> Self {
        let mut ds = Self {
            m: m.max(1),
            upper_bound_b,
            _max_insertions_n: max_insertions_n,
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

    // checked
    /// 判断当前数据结构是否为空
    pub fn is_empty(&self) -> bool {
        self.key_to_node.is_empty()
    }

    // checked
    /// 获取当前数据结构中节点的数量
    pub fn len(&self) -> usize {
        self.key_to_node.len()
    }

    /// 插入一个节点，如果节点已经存在，则更新其值
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
            self.delete_node(old_id);
        }

        let block_id = self.locate_d1_block(value);
        let node_id = self.new_node(key, value, block_id);
        self.key_to_node.insert(key, node_id);
        self.push_node_to_block_tail(block_id, node_id);

        if self.blocks[block_id].len > self.m {
            self.split_d1_block(block_id);
        }
    }

    pub fn batch_prepend(&mut self, records: &[(usize, u64)]) {
        if records.is_empty() {
            return;
        }
        let mut best_in_batch = HashMap::<usize, u64>::new();
        for &(k, v) in records {
            best_in_batch
                .entry(k)
                .and_modify(|old| *old = min(*old, v))
                .or_insert(v);
        }

        let mut effective = Vec::<(usize, u64)>::new();
        for (k, v) in best_in_batch {
            if let Some(&old_id) = self.key_to_node.get(&k) {
                if v >= self.nodes[old_id].value {
                    continue;
                }
                self.delete_node(old_id);
            }
            effective.push((k, v));
        }
        if effective.is_empty() {
            return;
        }

        effective.sort_unstable_by_key(|&(k, v)| (v, k));
        let block_cap = if effective.len() <= self.m {
            self.m
        } else {
            self.m.div_ceil(2)
        };

        let mut cursor = 0usize;
        let mut block_ids = Vec::<usize>::new();
        while cursor < effective.len() {
            let end = min(cursor + block_cap, effective.len());
            let slice = &effective[cursor..end];
            let upper = slice.last().map(|&(_, v)| v).unwrap_or(0);
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

    pub fn pull(&mut self) -> PullResult {
        let mut cand_ids = Vec::<usize>::new();
        let mut c0 = 0usize;
        let mut c1 = 0usize;
        let mut d0_exhausted = true;
        let mut d1_exhausted = true;

        let mut cur0 = self.d0_head;
        while let Some(bid) = cur0 {
            if c0 >= self.m {
                d0_exhausted = false;
                break;
            }
            let mut p = self.blocks[bid].head;
            while let Some(id) = p {
                cand_ids.push(id);
                c0 += 1;
                p = self.nodes[id].next;
            }
            cur0 = self.blocks[bid].next;
        }

        let d1_blocks: Vec<usize> = self.d1_upper_index.iter().map(|&(_, id)| id).collect();
        for bid in d1_blocks {
            if c1 >= self.m {
                d1_exhausted = false;
                break;
            }
            let mut p = self.blocks[bid].head;
            while let Some(id) = p {
                cand_ids.push(id);
                c1 += 1;
                p = self.nodes[id].next;
            }
        }

        let result_ids = if d0_exhausted && d1_exhausted && cand_ids.len() <= self.m {
            cand_ids
        } else {
            cand_ids.sort_unstable_by_key(|&id| (self.nodes[id].value, self.nodes[id].key));
            cand_ids.into_iter().take(self.m).collect()
        };

        for &id in &result_ids {
            if self.nodes[id].alive {
                self.delete_node(id);
            }
        }

        let boundary = if self.is_empty() {
            self.upper_bound_b
        } else {
            self.min_remaining_value().unwrap_or(self.upper_bound_b)
        };

        let mut keys = result_ids
            .iter()
            .map(|&id| self.nodes[id].key)
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        PullResult { boundary, keys }
    }

    // checked
    fn new_block(&mut self, seq: BlockSeq, upper: u64) -> usize {
        let id = self.blocks.len();
        self.blocks.push(Block::new(seq, upper));
        id
    }

    // checked
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

    // checked
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

    // checked
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

    // checked
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
        let right_upper = ids
            .last()
            .map(|&id| self.nodes[id].value)
            .expect("block cannot be empty");

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

    // checked
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

    // checked
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

    // checked
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

    fn delete_node(&mut self, node_id: usize) {
        if !self.nodes[node_id].alive {
            return;
        }
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
        self.blocks[block_id].len = self.blocks[block_id].len.saturating_sub(1);

        if self.key_to_node.get(&key) == Some(&node_id) {
            self.key_to_node.remove(&key);
        }
        if self.blocks[block_id].len == 0 {
            self.remove_empty_block(block_id);
        }
    }

    fn remove_empty_block(&mut self, block_id: usize) {
        if !self.blocks[block_id].alive {
            return;
        }
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

    fn min_remaining_value(&self) -> Option<u64> {
        let d0_min = self.d0_head.and_then(|id| self.block_min_value(id));
        let d1_min = self
            .d1_upper_index
            .iter()
            .next()
            .and_then(|&(_, id)| self.block_min_value(id));
        match (d0_min, d1_min) {
            (Some(a), Some(b)) => Some(min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    #[cfg(test)]
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
    use super::BlockDs;

    #[test]
    fn insert_update_then_pull_returns_m_smallest() {
        let mut ds = BlockDs::new(32, 2, 999);
        ds.insert(1, 30);
        ds.insert(2, 10);
        ds.insert(3, 20);
        ds.insert(2, 8);
        ds.insert(1, 40);

        let out = ds.pull();
        assert_eq!(out.keys, vec![2, 3]);
        assert_eq!(out.boundary, 30);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn batch_prepend_keeps_smallest_per_key() {
        let mut ds = BlockDs::new(64, 4, 1000);
        ds.insert(10, 100);
        ds.insert(11, 110);
        ds.batch_prepend(&[(1, 5), (2, 3), (1, 4), (10, 90), (10, 95)]);

        let out = ds.pull();
        assert_eq!(out.keys, vec![1, 2, 10, 11]);
        assert_eq!(out.boundary, 1000);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_b_when_structure_becomes_empty() {
        let mut ds = BlockDs::new(16, 2, 777);
        ds.insert(1, 10);
        ds.insert(2, 20);

        let first = ds.pull();
        assert_eq!(first.keys, vec![1, 2]);
        assert_eq!(first.boundary, 777);
        assert!(ds.is_empty());
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_boundary_is_smallest_remaining_value() {
        let mut ds = BlockDs::new(64, 2, 500);
        ds.insert(1, 10);
        ds.insert(2, 20);
        ds.insert(3, 30);
        ds.insert(4, 40);

        let out = ds.pull();
        assert_eq!(out.keys, vec![1, 2]);
        assert_eq!(out.boundary, 30);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }

    #[test]
    fn pull_does_not_mistake_prefix_for_all_elements() {
        let mut ds = BlockDs::new(64, 2, 1000);
        ds.insert(1, 10);
        ds.insert(2, 20);
        ds.insert(3, 30);
        ds.insert(4, 40);
        ds.insert(5, 50);

        let first = ds.pull();
        assert_eq!(first.keys, vec![1, 2]);
        assert_eq!(first.boundary, 30);

        let second = ds.pull();
        assert_eq!(second.keys, vec![3, 4]);
        assert_eq!(second.boundary, 50);
        ds.sanity_check_links();
        ds.sanity_check_keys();
    }
}
