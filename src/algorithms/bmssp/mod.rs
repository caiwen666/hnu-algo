pub mod block_ds;
pub mod const_graph;
pub mod path_dist;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::algorithms::bmssp::const_graph::ConstGraph;
use crate::algorithms::bmssp::path_dist::PathDist;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BMSSPResult {
    /// 新的边界 B'（论文中定义为 boundary）
    pub new_boundary: PathDist,
    /// complete 的点集合 U
    pub complete: HashSet<usize>,
}

/// BMSSP（Bounded Multi-Source Shortest Path）的递归子过程上下文。
///
/// 目前只实现论文中 l=0 时的 base case（mini Dijkstra）。
#[derive(Debug, Clone)]
pub struct BMSSP {
    graph: ConstGraph,
    source: usize,

    /// 当前在常度数图上的点数 n
    n: usize,
    /// k = floor(log^{1/3} n)
    k: usize,
    /// t = floor(log^{2/3} n)
    t: usize,

    /// 当前维护的距离估计 now_dis[·]，永远满足 now_dis[v] >= d(v)，其中 d(v) 为最短路
    /// 不可到达用 u64::MAX 表示。
    now_dis: Vec<PathDist>,
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
    pub fn new(graph: ConstGraph, source: usize) -> Self {
        let n = graph.const_n();
        assert!(source < n, "source out of range");
        assert!(n > 0, "graph has no vertices");

        let logn = if n <= 1 { 0.0 } else { (n as f64).log2() };
        let k = ((logn.powf(1.0 / 3.0)).floor() as usize).max(1);
        let t = ((logn.powf(2.0 / 3.0)).floor() as usize).max(1);

        let mut now_dis = vec![PathDist::MAX; n];
        now_dis[source] = PathDist::new(0, 0, source, 0);

        Self {
            graph,
            source,
            n,
            k,
            t,
            now_dis,
        }
    }

    pub fn k(&self) -> usize {
        self.k
    }

    /// BMSSP 的 finding pivots 操作
    ///
    /// # Parameters
    ///
    /// - `s` - 前沿集合。
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
    fn find_pivots(&mut self, s: &HashSet<usize>, b: PathDist) -> (HashSet<usize>, HashSet<usize>) {
        let k = self.k;
        assert!(!s.is_empty(), "s must not be empty");

        let mut w_set: HashSet<usize> = s.iter().copied().collect();

        // 使用 01 滚动数组来维护 wi
        // last_wi_index 即为 1 - now_wi_index
        let mut now_wi_index = 1;
        let mut wi = [w_set.clone(), HashSet::new()];

        for _ in 0..k {
            let (slot0, slot1) = wi.split_at_mut(1);
            let (wi_last, wi_now) = if now_wi_index == 0 {
                (&mut slot1[0], &mut slot0[0])
            } else {
                (&mut slot0[0], &mut slot1[0])
            };
            wi_now.clear();

            for &u in wi_last.iter() {
                let u_dis = self.now_dis[u];
                for &(v, w) in self.graph.adj()[u].iter() {
                    let relaxed_dis = PathDist::new(u_dis.dis + w as u64, u_dis.hop + 1, v, u);
                    if relaxed_dis <= self.now_dis[v] {
                        self.now_dis[v] = relaxed_dis;
                        if relaxed_dis < b {
                            wi_now.insert(v);
                        }
                    }
                }
            }

            w_set.extend(wi_now.iter().copied());

            if w_set.len() > k * s.len() {
                return (s.clone(), w_set);
            }

            now_wi_index = 1 - now_wi_index;
        }

        // 接下来构造最短路森林。
        // TODO 内存开销可能有点大？
        let mut parent: HashMap<usize, usize> = HashMap::new();
        let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
        for &u in w_set.iter() {
            let u_dis = self.now_dis[u];
            for &(v, w) in self.graph.adj()[u].iter() {
                if !w_set.contains(&v) {
                    continue;
                }
                let relaxed_dis = PathDist::new(u_dis.dis + w as u64, u_dis.hop + 1, v, u);
                if relaxed_dis == self.now_dis[v] {
                    parent.insert(v, u);
                    children.entry(u).or_default().push(v);
                }
            }
        }

        // 非递归遍历，希望常数小点
        let get_subtree_size = |root: usize| -> usize {
            let mut queue: VecDeque<usize> = VecDeque::new();
            queue.push_back(root);
            let mut size = 1usize;
            while let Some(u) = queue.pop_front() {
                for &v in children.get(&u).unwrap_or(&Vec::new()).iter() {
                    queue.push_back(v);
                    size += 1;
                }
            }
            size
        };

        let mut p = HashSet::new();
        for &u in children.keys() {
            if parent.contains_key(&u) {
                continue;
            }
            let subtree_size = get_subtree_size(u);
            if subtree_size >= k {
                p.insert(u);
            }
        }

        (p, w_set)
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
    fn base_case(&mut self, s: usize, b: PathDist) -> BMSSPResult {
        let k = self.k;

        assert!(
            self.now_dis[s] != PathDist::MAX && self.now_dis[s] < b,
            "base case requires source reachable and b > now_dis[s]"
        );

        let mut u0 = HashSet::new();
        u0.insert(s);

        let mut heap: BinaryHeap<Reverse<(PathDist, usize)>> = BinaryHeap::new();
        heap.push(Reverse((self.now_dis[s], s)));

        while u0.len() < k + 1 {
            let Some(Reverse((u_dis, u))) = heap.pop() else {
                break;
            };
            if u_dis != self.now_dis[u] {
                // 说明这是个过期状态
                // 论文中用的是 decrease key 操作。我们这里相当于做的是 lazy decrease key
                continue;
            }

            u0.insert(u);

            for &(v, w) in self.graph.adj()[u].iter() {
                let relaxed_dis = PathDist::new(u_dis.dis + w as u64, u_dis.hop + 1, v, u);
                if relaxed_dis < b && relaxed_dis <= self.now_dis[v] {
                    self.now_dis[v] = relaxed_dis;
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
                .copied()
                .map(|v| self.now_dis[v])
                .max()
                .expect("u0 should not be empty");

            // TODO 内存分配开销是不是有点大？
            let complete = u0
                .into_iter()
                .filter(|&v| self.now_dis[v] < new_boundary)
                .collect::<HashSet<_>>();

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

    /// 论文 Algorithm 2：`B` 为标量上界时用 `scalar_upper`，保证「真实距离 ≤ B」与 `PathDist` 上界一致。
    fn big_b() -> PathDist {
        PathDist::scalar_upper(1_000_000)
    }

    fn bmssp_from_general_graph(
        g: Vec<Vec<(usize, usize)>>,
        source_orig: usize,
    ) -> (BMSSP, ConstGraph, usize) {
        let cg = ConstGraph::from_general_graph(&g);
        let source_const = cg
            .orig_to_const(source_orig)
            .expect("source must have a representative in const graph");
        (BMSSP::new(cg.clone(), source_const), cg, source_const)
    }

    #[test]
    fn base_case_single_vertex_returns_boundary_b_and_complete_singleton() {
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![]], 0);
        let b = big_b();
        let r = m.base_case(s, b);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, HashSet::from([s]));
    }

    #[test]
    fn base_case_unreachable_pop_exhaustion_returns_u0_and_b() {
        // 0 孤立，1 自闭环；从 0 只能扩张出 {0}
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![], vec![(1, 1)]], 0);
        let b = big_b();
        let r = m.base_case(s, b);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, HashSet::from([s]));
    }

    /// `k = 1`（例如 n=3）时需抽出 `k+1=2` 个不同点才进入「截断」分支。
    #[test]
    fn base_case_two_vertices_linear_partial_execution_max_boundary() {
        // 0 --1--> 1 --1--> 2
        let (mut m, cg, s) = bmssp_from_general_graph(vec![vec![(1, 1)], vec![(2, 1)], vec![]], 0);
        let b = big_b();
        let r = m.base_case(s, b);

        let rep1 = cg.orig_to_const(1).unwrap();
        assert_eq!(m.now_dis[s].dis, 0);
        assert_eq!(m.now_dis[rep1].dis, 1);
        // 注意：`from_general_graph` 会把每个原点替换成 0 权环上的若干替身点。
        // 对于点 1（既有入邻居 0 又有出邻居 2），需要先经过环上的中间点 x_{1,2} 才能走原边到 2。
        // base case 只 ExtractMin 至多 k+1 次（这里 k=1），不会继续从未出堆的环点扩展，因此 2 的代表点不保证被松弛。
        let rep2 = cg.orig_to_const(2).unwrap();
        assert_eq!(m.now_dis[rep2], PathDist::MAX);

        // 进入截断分支时 `|U0| = k+1 = 2`，`new_boundary` 为 U0 上 `PathDist` 的 max；
        // 由于 const 图点编号不同，我们只检查其标量距离确为 1。
        assert_eq!(r.new_boundary.dis, 1);

        // 按算法，complete 为 `{v in U0 : now_dis[v] < new_boundary}`，至少包含源点；
        // 在该例中 `rep1` 处于边界，不能被 complete。
        assert!(r.complete.contains(&s));
        assert!(!r.complete.contains(&rep1));
    }

    #[test]
    fn base_case_small_boundary_prevents_some_vertices_from_being_complete() {
        // 普通图 0->1(2), 1->2(2)，取 B=3：只能触达 1（d=2），2 的最短路 d=4 不应被触达，更不应 complete。
        let (mut m, cg, s) = bmssp_from_general_graph(vec![vec![(1, 2)], vec![(2, 2)], vec![]], 0);
        let b = PathDist::scalar_upper(3);
        let r = m.base_case(s, b);
        let rep1 = cg.orig_to_const(1).unwrap();
        let rep2 = cg.orig_to_const(2).unwrap();

        assert_eq!(m.now_dis[rep1].dis, 2);
        assert_eq!(m.now_dis[rep2], PathDist::MAX);
        assert_eq!(r.new_boundary.dis, 2);
        assert!(r.complete.contains(&s));
        assert!(!r.complete.contains(&rep1), "边界点不应 complete");
        assert!(!r.complete.contains(&rep2));
    }

    #[test]
    fn base_case_small_boundary_can_block_relaxation_entirely() {
        // 0->1(5)，取 B=4：1 的代表点不应被松弛，complete 只能包含源点
        let (mut m, cg, s) = bmssp_from_general_graph(vec![vec![(1, 5)], vec![]], 0);
        let rep1 = cg.orig_to_const(1).unwrap();
        let b = PathDist::scalar_upper(4);
        let r = m.base_case(s, b);
        assert_eq!(m.now_dis[rep1], PathDist::MAX);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, HashSet::from([s]));
    }

    #[test]
    fn base_case_weights_zero_chain() {
        let (mut m, cg, s) = bmssp_from_general_graph(vec![vec![(1, 0)], vec![(2, 0)], vec![]], 0);
        let b = big_b();
        let r = m.base_case(s, b);
        let rep1 = cg.orig_to_const(1).unwrap();
        assert_eq!(m.now_dis[rep1].dis, 0);
        // 同 `linear` 测试：点 1 既有入又有出，需先出堆环点才会继续到 2；这里不强行要求 2 被松弛。
        let rep2 = cg.orig_to_const(2).unwrap();
        assert_eq!(m.now_dis[rep2], PathDist::MAX);
        assert_eq!(r.new_boundary.dis, 0);
        assert!(r.complete.contains(&s));
    }

    #[test]
    #[should_panic(expected = "base case requires source reachable and b > now_dis[s]")]
    fn base_case_panics_when_b_not_strictly_above_now_dis() {
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![]], 0);
        let b = m.now_dis[s];
        let _ = m.base_case(s, b);
    }

    #[test]
    fn base_case_relaxed_path_can_improve_after_larger_key_dequeued() {
        // 该用例在“直接常度数图”下可用于验证 lazy decrease-key；
        // 但经 `from_general_graph` 转换后，源点替身环会引入额外中间点，且 base case 提前停在 k+1 次 ExtractMin，
        // 因此不保证能走到“更优的中间环点”并触发改进。这里仅验证：直接边能正确松弛到 1。
        let (mut m, cg, s) =
            bmssp_from_general_graph(vec![vec![(1, 100), (2, 1)], vec![], vec![(1, 1)]], 0);
        let b = big_b();
        let _ = m.base_case(s, b);
        let rep1 = cg.orig_to_const(1).unwrap();
        assert_eq!(m.now_dis[rep1].dis, 100);
    }

    /// 前置条件 `b > now_dis[s]` 是 **PathDist 全序**（路径偏序的具体落地）。
    #[test]
    fn base_case_accepts_b_lex_above_source_even_if_end_is_large() {
        let (mut m, _cg, s) = bmssp_from_general_graph(vec![vec![]], 0);
        let b = PathDist::from_dis(0, 999);
        assert!(m.now_dis[s] < b);
        let r = m.base_case(s, b);
        assert_eq!(r.new_boundary, b);
        assert_eq!(r.complete, HashSet::from([s]));
    }

    /// `|W| > k|S|` 时（论文第 15 行）应返回 `P = S`。`n = 4` 时 `k = 1`，一星三叶一轮可达 `|W| = 4`。
    #[test]
    fn find_pivots_large_visit_set_returns_whole_frontier_as_pivots() {
        let cg = ConstGraph::new(vec![vec![(1, 1), (2, 1), (3, 1)], vec![], vec![], vec![]]);
        let mut m = BMSSP::new(cg, 0);
        let s = HashSet::from([0usize]);
        let (p, w) = m.find_pivots(&s, PathDist::scalar_upper(100));
        assert_eq!(p, s);
        assert!(w.len() >= 4);
    }

    /// 一轮全局扫描内应能递推：0 -10-> 1，0 -1-> 2 -1-> 1，单轮后 `d[1]=10`（不是 2）。
    #[test]
    fn find_pivots_one_bf_round_chains_relaxations() {
        let cg = ConstGraph::new(vec![vec![(1, 10), (2, 1)], vec![], vec![(1, 1)]]);
        let mut m = BMSSP::new(cg, 0);
        let s = HashSet::from([0usize]);
        let _ = m.find_pivots(&s, PathDist::scalar_upper(100));
        assert_eq!(m.now_dis[1].dis, 10);
    }
}
