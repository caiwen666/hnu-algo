pub mod block_ds;
pub mod const_graph;
pub mod path_dist;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::algorithms::bmssp::block_ds::{BlockDs, PullResult};
use crate::algorithms::bmssp::const_graph::ConstGraph;
use crate::algorithms::bmssp::path_dist::PathDist;

#[derive(Debug, Clone)]
pub struct BMSSPResult {
    /// 新的边界 B'（论文中定义为 boundary）
    pub new_boundary: PathDist,
    /// complete 的点集合 U，以 Vec<u32> 形式返回
    pub complete: Vec<u32>,
}

/// BMSSP（Bounded Multi-Source Shortest Path）的递归子过程上下文。
#[derive(Debug, Clone)]
pub struct BMSSP {
    graph: ConstGraph,
    source: usize,

    /// k = floor(log^{1/3} n)
    k: usize,
    /// t = floor(log^{2/3} n)
    t: usize,

    /// 最顶层 bmssp 的层数
    top_l: usize,

    /// 当前维护的距离估计 now_dis[·]，永远满足 now_dis[v] >= d(v)，其中 d(v) 为最短路
    /// 不可到达用 PathDist::MAX 表示。
    now_dis: Vec<PathDist>,

    /// 标记某个点是否在某个集合中（可复用的 bool 标记数组）。主要是 find pivots 在用。
    in_set: Vec<bool>,
    /// 时间戳标记，避免清空 in_set；epoch 递增。主要是 bmssp 用于维护 u_set
    epoch: Vec<u32>,
    cur_epoch: u32,

    /// find_pivots 的 pool: parent[v], children 列表
    parent: Vec<u32>,
    children_head: Vec<u32>,
    children_next: Vec<u32>,

    /// batch_prepend 的 pool
    pool1: Vec<PathDist>,
    /// bmssp 里每轮 batch_prepend 的缓冲
    k_buf: Vec<(u32, PathDist)>,
    /// BlockDs 的 key_to_node 缓冲区，每层一个，预分配避免重复 alloc
    /// block_ds_bufs[i] 表示第 i 层 bmssp 的缓冲区
    block_ds_bufs: Vec<Vec<u32>>,
}

impl BMSSP {
    /// 创建一个新的 BMSSP 实例。
    ///
    /// # Parameters
    ///
    /// - `graph` - 常度数图。一般图需要转换为常度数图后调用。
    /// - `source` - 源点。原图上的编号。
    ///
    /// # Panics
    ///
    /// - `source` 超出顶点范围时 panic。
    /// - 图上节点数量为 0 时会 panic。
    /// - 图上的节点数量必须小于 2097152，否则会 panic。
    pub fn new(graph: ConstGraph, source: usize) -> Self {
        let n = graph.const_n();
        assert!(n < 2097152, "graph has too many vertices");
        assert!(source < n, "source out of range");
        assert!(n > 0, "graph has no vertices");

        let logn = if n <= 1 { 0.0 } else { (n as f64).log2() };
        let k = ((logn.powf(1.0 / 3.0)).floor() as usize).max(1);
        let t = ((logn.powf(2.0 / 3.0)).floor() as usize).max(1);
        let top_l = ((logn / t as f64).ceil() as usize).max(1);

        let mut now_dis = vec![PathDist::MAX; n];
        now_dis[source] = PathDist::new(0, 0, source as u32, 0);

        let mut block_ds_bufs = Vec::with_capacity(top_l + 1);
        for _ in 0..=top_l {
            block_ds_bufs.push(vec![u32::MAX; n]);
        }

        Self {
            graph,
            source,
            k,
            t,
            top_l,
            now_dis,
            in_set: vec![false; n],
            epoch: vec![0; n],
            cur_epoch: 0,
            parent: vec![u32::MAX; n],
            children_head: vec![u32::MAX; n],
            children_next: vec![u32::MAX; n],
            pool1: vec![PathDist::MAX; n],
            k_buf: Vec::new(),
            block_ds_bufs,
        }
    }

    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }

    /// 求 source 到所有点的最短路
    pub fn solve(&mut self) {
        let source = self.source as u32;
        self.bmssp(self.top_l, &[source], PathDist::MAX);
    }

    /// 获取最短路结果
    ///
    /// # Returns
    ///
    /// 返回一个数组 v，v(i) 表示 source 到 i 的最短路距离，不可到达用 u64::MAX 表示。
    pub fn fetch_result(&self) -> Vec<u64> {
        self.now_dis.iter().map(|&d| d.dis()).collect()
    }

    /// BMSSP 算法主体
    ///
    /// # Parameters
    ///
    /// - `l` - 当前层数。
    /// - `s` - 前沿顶点切片（与原先 `HashSet` 语义相同，避免每层额外分配）。
    /// - `b` - 上界。
    ///
    /// # Returns
    ///
    /// 返回一个 BMSSPResult 结构体，包含新的边界 B' 和 complete 的点集合 U。
    ///
    ///
    /// # Preconditions
    ///
    /// - 所有没 complete 的，且最短路小于 `b` 的点，其最短路必须要经过 `s` 中某个已经 complete 的点
    ///
    /// # Panics
    ///
    /// - 如果 `s` 为空，则 panic。
    /// - 如果 `s` 的大小大于了 $2^{lt}$，则 panic。
    fn bmssp(&mut self, l: usize, s: &[u32], b: PathDist) -> BMSSPResult {
        let t = self.t;
        let k = self.k;
        let size_limit = 2usize.pow(t as u32 * l as u32);
        let k_size_limit = k * size_limit;
        assert!(!s.is_empty(), "s must not be empty");
        assert!(
            s.len() <= size_limit,
            "s is too large: {} > {}",
            s.len(),
            size_limit
        );

        if l == 0 {
            assert!(s.len() == 1, "s must have exactly one element when l = 0");
            return self.base_case(s[0], b);
        }

        let (p_set, w_set) = self.find_pivots(s, b);

        let buf = &mut self.block_ds_bufs[l] as *mut Vec<u32>;
        // SAFETY: block_ds 的生命周期不超过本函数；递归调用使用不同的 l，因此缓冲区不会冲突
        let buf_ref = unsafe { &mut *buf };
        let mut block_ds = BlockDs::new(2usize.pow(((l - 1) * t) as u32), b, buf_ref);
        let mut now_boundary = PathDist::MAX;
        for &x in p_set.iter() {
            let dis = self.now_dis[x as usize];
            block_ds.insert(x, dis);
            now_boundary = now_boundary.min(dis);
        }

        let mut u_set: Vec<u32> = Vec::new();
        let mut u_count = 0usize;

        // 用 epoch 标记 u_set 成员，避免 HashSet
        self.cur_epoch = self.cur_epoch.wrapping_add(1);
        let ep = self.cur_epoch;

        while u_count < k_size_limit && !block_ds.is_empty() {
            let PullResult {
                boundary: upper_boundary,
                keys,
            } = block_ds.pull();

            let BMSSPResult {
                new_boundary,
                complete,
            } = self.bmssp(l - 1, &keys, upper_boundary);

            self.k_buf.clear();
            self.k_buf
                .reserve(complete.len().saturating_mul(2).saturating_add(keys.len()));

            for &u in complete.iter() {
                if self.epoch[u as usize] != ep {
                    self.epoch[u as usize] = ep;
                    u_set.push(u);
                    u_count += 1;
                }
                let u_dis = self.now_dis[u as usize];
                let (dsts, wts) = self.graph.neighbors(u as usize);
                for i in 0..dsts.len() {
                    let v = dsts[i];
                    let w = wts[i];
                    let relaxed_dis = PathDist::new(u_dis.dis() + w as u64, u_dis.hop() + 1, v, u);
                    if relaxed_dis <= self.now_dis[v as usize] {
                        self.now_dis[v as usize] = relaxed_dis;
                        if relaxed_dis >= upper_boundary && relaxed_dis < b {
                            block_ds.insert(v, relaxed_dis);
                        } else if relaxed_dis >= new_boundary && relaxed_dis < upper_boundary {
                            self.k_buf.push((v, relaxed_dis));
                        }
                    }
                }
            }

            for x in keys {
                let x_dis = self.now_dis[x as usize];
                if x_dis >= new_boundary && x_dis < upper_boundary {
                    self.k_buf.push((x, x_dis));
                }
            }
            block_ds.batch_prepend(&self.k_buf, &mut self.pool1);
            now_boundary = new_boundary;
        }

        block_ds.cleanup();
        drop(block_ds);

        now_boundary = now_boundary.min(b);

        for x in w_set {
            if self.now_dis[x as usize] < now_boundary
                && self.epoch[x as usize] != ep {
                    self.epoch[x as usize] = ep;
                    u_set.push(x);
                }
        }

        BMSSPResult {
            new_boundary: now_boundary,
            complete: u_set,
        }
    }

    /// BMSSP 的 finding pivots 操作
    ///
    /// # Parameters
    ///
    /// - `s` - 前沿顶点切片。
    /// - `b` - 上界。
    ///
    /// # Returns
    ///
    /// 返回一个二元组，第一个元素对应于论文中的 P 集合，第二个元素对应论文中的 W 集合。
    ///
    /// # Preconditions
    ///
    /// 所有最短路小于 b 的点，最短路必须经过 s 中某个已经 complete 的点。
    ///
    /// # Panics
    ///
    /// - 如果 s 为空，则 panic。
    fn find_pivots(&mut self, s: &[u32], b: PathDist) -> (Vec<u32>, Vec<u32>) {
        let k = self.k;
        assert!(!s.is_empty(), "s must not be empty");

        // w_set: 用 in_set 标记 + w_list 记录成员
        // wi: 滚动数组用两个 Vec
        let mut w_list: Vec<u32> = Vec::with_capacity(k * s.len() + s.len());
        for &v in s {
            self.in_set[v as usize] = true;
            w_list.push(v);
        }

        // TODO 复用公共内存以减少开销
        let mut wi_cur: Vec<u32> = s.to_vec();
        let mut wi_next: Vec<u32> = Vec::new();

        for _ in 0..k {
            wi_next.clear();

            for &u in wi_cur.iter() {
                let u_dis = self.now_dis[u as usize];
                let (dsts, wts) = self.graph.neighbors(u as usize);
                for i in 0..dsts.len() {
                    let v = dsts[i];
                    let w = wts[i];
                    let relaxed_dis = PathDist::new(u_dis.dis() + w as u64, u_dis.hop() + 1, v, u);
                    if relaxed_dis <= self.now_dis[v as usize] {
                        self.now_dis[v as usize] = relaxed_dis;
                        if relaxed_dis < b {
                            if !self.in_set[v as usize] {
                                self.in_set[v as usize] = true;
                                w_list.push(v);
                            }
                            wi_next.push(v);
                        }
                    }
                }
            }

            if w_list.len() > k * s.len() {
                let p_set: Vec<u32> = s.to_vec();
                // 清理 in_set
                for &v in w_list.iter() {
                    self.in_set[v as usize] = false;
                }
                return (p_set, w_list);
            }

            std::mem::swap(&mut wi_cur, &mut wi_next);
        }

        // 构造最短路森林
        // parent[v] = u 表示 v 的父节点是 u
        // children 用链表：children_head[u] 是 u 的第一个子节点，children_next[v] 是 v 的下一个兄弟
        for &u in w_list.iter() {
            let u_dis = self.now_dis[u as usize];
            let (dsts, wts) = self.graph.neighbors(u as usize);
            for i in 0..dsts.len() {
                let v = dsts[i];
                let w = wts[i];
                if !self.in_set[v as usize] {
                    continue;
                }
                let relaxed_dis = PathDist::new(u_dis.dis() + w as u64, u_dis.hop() + 1, v, u);
                if relaxed_dis == self.now_dis[v as usize] {
                    self.parent[v as usize] = u;
                    self.children_next[v as usize] = self.children_head[u as usize];
                    self.children_head[u as usize] = v;
                }
            }
        }

        let mut p_set: Vec<u32> = Vec::new();
        // BFS 栈，复用 wi_next。看起来 stack 后面还会 clear，是没意义的，但其实这里少了一次内存分配
        let stack = &mut wi_next;

        for &u in w_list.iter() {
            if self.parent[u as usize] == u32::MAX {
                // u 是某棵树的根
                let mut subtree_size = 0u32;
                stack.clear();
                stack.push(u);
                while let Some(x) = stack.pop() {
                    subtree_size += 1;
                    let mut child = self.children_head[x as usize];
                    while child != u32::MAX {
                        stack.push(child);
                        child = self.children_next[child as usize];
                    }
                }
                if subtree_size as usize >= k {
                    p_set.push(u);
                }
            }
        }

        // 清理 parent, children, in_set
        for &v in w_list.iter() {
            self.parent[v as usize] = u32::MAX;
            self.children_head[v as usize] = u32::MAX;
            self.children_next[v as usize] = u32::MAX;
            self.in_set[v as usize] = false;
        }

        (p_set, w_list)
    }

    /// BMSSP 的 base case
    ///
    /// # Parameters
    ///
    /// - `s` - 源点。到达 base case 时 S 集合内肯定只有一个点。这里用 `s` 表示这个点。
    /// - `b` - 上界。
    ///
    /// # Preconditions
    ///
    /// - `s` 必须是 complete 的
    /// - 所有没被 complete 的，且最短路小于 b 的点的最短路必须经过 x
    ///
    /// # Panics
    ///
    /// - 如果 `s` 不可达，则 panic。
    /// - 如果 `b` 小于 `now_dis[s]`，则 panic。
    fn base_case(&mut self, s: u32, b: PathDist) -> BMSSPResult {
        let k = self.k;

        assert!(
            self.now_dis[s as usize] != PathDist::MAX && self.now_dis[s as usize] < b,
            "base case requires source reachable and b > now_dis[s]"
        );

        let mut u0: Vec<u32> = Vec::with_capacity(k + 2);

        // 用 epoch 标记已经 extract 的点
        self.cur_epoch = self.cur_epoch.wrapping_add(1);
        let ep = self.cur_epoch;

        let mut heap: BinaryHeap<Reverse<(PathDist, u32)>> = BinaryHeap::new();
        heap.push(Reverse((self.now_dis[s as usize], s)));

        while u0.len() < k + 1 {
            let Some(Reverse((u_dis, u))) = heap.pop() else {
                break;
            };
            if u_dis != self.now_dis[u as usize] {
                // 说明这是个过期状态
                // 论文中用的是 decrease key 操作。我们这里相当于做的是 lazy decrease key
                continue;
            }
            if self.epoch[u as usize] == ep {
                // 已经是 complete，说明之前已经被松弛地更优了
                continue;
            }
            self.epoch[u as usize] = ep;
            u0.push(u);

            let (dsts, wts) = self.graph.neighbors(u as usize);
            for i in 0..dsts.len() {
                let v = dsts[i];
                let w = wts[i];
                let relaxed_dis = PathDist::new(u_dis.dis() + w as u64, u_dis.hop() + 1, v, u);
                if relaxed_dis < b && relaxed_dis <= self.now_dis[v as usize] {
                    self.now_dis[v as usize] = relaxed_dis;
                    heap.push(Reverse((relaxed_dis, v)));
                }
            }
        }

        if u0.len() <= k {
            BMSSPResult {
                new_boundary: b,
                complete: u0,
            }
        } else {
            let new_boundary = u0
                .iter()
                .map(|&v| self.now_dis[v as usize])
                .max()
                .expect("u0 should not be empty");

            // TODO 内存分配开销是不是有点大？
            // 这里其实主要就是想把 u0 中最短路最大的那个点的距离作为 new_boundary
            // 然后再把最大的那个点筛掉
            let complete: Vec<u32> = u0
                .into_iter()
                .filter(|&v| self.now_dis[v as usize] < new_boundary)
                .collect();

            BMSSPResult {
                new_boundary,
                complete,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big_b() -> PathDist {
        PathDist::scalar_upper(1_000_000)
    }

    fn bmssp_from_general_graph(
        g: Vec<Vec<(usize, usize)>>,
        source_orig: usize,
    ) -> (BMSSP, ConstGraph, u32) {
        let cg = ConstGraph::from_general_graph(&g);
        let source_const =
            cg.orig_to_const(source_orig)
                .expect("source must have a representative in const graph") as u32;
        (
            BMSSP::new(cg.clone(), source_const as usize),
            cg,
            source_const,
        )
    }

    #[test]
    fn base_case_single_vertex_returns_boundary_b_and_complete_singleton() {
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![]], 0);
        let b = big_b();
        let r = m.base_case(s, b);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, vec![s]);
    }

    #[test]
    fn base_case_unreachable_pop_exhaustion_returns_u0_and_b() {
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![], vec![(1, 1)]], 0);
        let b = big_b();
        let r = m.base_case(s, b);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, vec![s]);
    }

    #[test]
    fn find_pivots_large_visit_set_returns_whole_frontier_as_pivots() {
        let cg = ConstGraph::new(vec![vec![(1, 1), (2, 1), (3, 1)], vec![], vec![], vec![]]);
        let mut m = BMSSP::new(cg, 0);
        let s = [0u32];
        let (p, w) = m.find_pivots(&s, PathDist::scalar_upper(100));
        assert_eq!(p, vec![0u32]);
        assert!(w.len() >= 4);
    }

    #[test]
    fn find_pivots_one_bf_round_chains_relaxations() {
        let cg = ConstGraph::new(vec![vec![(1, 10), (2, 1)], vec![], vec![(1, 1)]]);
        let mut m = BMSSP::new(cg, 0);
        let s = [0u32];
        let _ = m.find_pivots(&s, PathDist::scalar_upper(100));
        assert_eq!(m.now_dis[1].dis(), 10);
    }

    #[test]
    fn test_bmssp_simple() {
        let g = vec![vec![(1, 4), (2, 1)], vec![], vec![(1, 2)]];
        let cg = ConstGraph::from_general_graph(&g);
        let source = cg.orig_to_const(0).unwrap();
        let mut bmssp = BMSSP::new(cg.clone(), source);
        bmssp.solve();
        let dist = bmssp.fetch_result();
        let mut actual = vec![u64::MAX; g.len()];
        for v in 0..g.len() {
            let rv = cg.orig_to_const(v).unwrap();
            actual[v] = dist[rv];
        }
        actual[0] = 0;
        assert_eq!(actual, vec![0, 3, 1]);
    }
}
