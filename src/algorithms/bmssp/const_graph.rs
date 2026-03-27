/// 常度数图（入度/出度 ≤ 2），用于 BMSSP/论文算法的前置变换。
///
/// 这里使用 CSR（Compressed Sparse Row）格式存储，内存连续，缓存友好。
///
/// 论文（Duan et al. 2025）中的经典转换：
/// - 对每个原图点 `v`，把它替换成一圈 0 权有向环；对 `v` 的每个（入/出）相邻点 `w`，创建一个节点 `x_{v,w}` 放在环上；
/// - 对每条原图边 `(u -> v, w_uv)`，添加一条边 `x_{u,v} -> x_{v,u}`，权重为 `w_uv`。
///
/// 这样得到的新图满足每个点的入/出度最多为 2，且最短路保持（原图中从 `s` 到 `t` 的最短距离
/// 等于新图中从 `rep(s)` 到 `rep(t)` 的最短距离，其中 `rep(v)` 是 `v` 对应环上的任意代表点）。
#[derive(Debug, Clone)]
pub struct ConstGraph {
    /// CSR offsets: adj_off[u]..adj_off[u+1] 是 u 的出边在 adj_dst/adj_wt 中的范围
    adj_off: Vec<u32>,
    /// 出边的目标顶点
    adj_dst: Vec<u32>,
    /// 出边的权重
    adj_wt: Vec<u32>,
    /// 原图点 -> 常度数图中代表点
    orig_to_const_rep: Vec<u32>,
    /// 常度数图点 -> 属于哪个原图点的"替身环"
    const_to_orig: Vec<u32>,
}

impl ConstGraph {
    /// 直接用"已经是常度数图"的邻接表构造 `ConstGraph`（CSR 格式）。
    pub fn new(adj: Vec<Vec<(usize, usize)>>) -> Self {
        let n = adj.len();
        let total_edges: usize = adj.iter().map(|e| e.len()).sum();
        let mut adj_off = Vec::with_capacity(n + 1);
        let mut adj_dst = Vec::with_capacity(total_edges);
        let mut adj_wt = Vec::with_capacity(total_edges);
        let mut off = 0u32;
        for edges in &adj {
            adj_off.push(off);
            for &(v, w) in edges {
                adj_dst.push(v as u32);
                adj_wt.push(w as u32);
            }
            off += edges.len() as u32;
        }
        adj_off.push(off);
        Self {
            adj_off,
            adj_dst,
            adj_wt,
            orig_to_const_rep: (0..n as u32).collect(),
            const_to_orig: (0..n as u32).collect(),
        }
    }

    /// 从任意有向非负权图构造常度数图，并保留映射。
    pub fn from_general_graph(graph: &[Vec<(usize, usize)>]) -> Self {
        use rustc_hash::FxHashMap;
        use std::collections::{BTreeMap, BTreeSet};

        let n = graph.len();

        let mut out_min: Vec<BTreeMap<usize, usize>> = vec![BTreeMap::new(); n];
        let mut in_set: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
        for (u, edges) in graph.iter().enumerate() {
            for &(v, w) in edges {
                debug_assert!(v < n);
                out_min[u]
                    .entry(v)
                    .and_modify(|old| *old = (*old).min(w))
                    .or_insert(w);
                in_set[v].insert(u);
            }
        }

        let mut pair_to_id: FxHashMap<(usize, usize), u32> = FxHashMap::default();
        let mut const_to_orig: Vec<u32> = Vec::new();
        let mut per_v_nodes: Vec<Vec<u32>> = vec![Vec::new(); n];

        for v in 0..n {
            let mut neigh: BTreeSet<usize> = BTreeSet::new();
            neigh.extend(in_set[v].iter().copied());
            neigh.extend(out_min[v].keys().copied());

            // 如果 v 完全没有相邻点，为了能提供映射/代表点，仍创建一个孤点（用 (v,v) 作为键）。
            if neigh.is_empty() {
                neigh.insert(v);
            }

            for w in neigh {
                let id = const_to_orig.len() as u32;
                const_to_orig.push(v as u32);
                pair_to_id.insert((v, w), id);
                per_v_nodes[v].push(id);
            }
        }

        let cn = const_to_orig.len();
        let mut adj_lists: Vec<Vec<(u32, u32)>> = vec![Vec::new(); cn];

        // 每个 v 的替身点用 0 权有向环连接：保证每个点额外 in/out 各 +1
        #[expect(clippy::needless_range_loop)]
        for v in 0..n {
            let nodes = &per_v_nodes[v];
            if nodes.len() > 1 {
                for i in 0..nodes.len() {
                    let a = nodes[i];
                    let b = nodes[(i + 1) % nodes.len()];
                    adj_lists[a as usize].push((b, 0));
                }
            }
        }

        // 原图每条边 (u->v, w) => x_{u,v} -> x_{v,u}，权重 w
        #[expect(clippy::needless_range_loop)]
        for u in 0..n {
            for (&v, &w) in out_min[u].iter() {
                let from = *pair_to_id.get(&(u, v)).expect("x_{u,v} must exist");
                let to = *pair_to_id.get(&(v, u)).expect("x_{v,u} must exist");
                adj_lists[from as usize].push((to, w as u32));
            }
        }

        // 建立“原图点 -> 代表点”映射：取环上的第一个即可
        let total_edges: usize = adj_lists.iter().map(|e| e.len()).sum();
        let mut adj_off = Vec::with_capacity(cn + 1);
        let mut adj_dst = Vec::with_capacity(total_edges);
        let mut adj_wt = Vec::with_capacity(total_edges);
        let mut off = 0u32;
        for edges in &adj_lists {
            adj_off.push(off);
            for &(v, w) in edges {
                adj_dst.push(v);
                adj_wt.push(w);
            }
            off += edges.len() as u32;
        }
        adj_off.push(off);

        let mut orig_to_const_rep = vec![0u32; n];
        for v in 0..n {
            orig_to_const_rep[v] = per_v_nodes[v][0];
        }

        Self {
            adj_off,
            adj_dst,
            adj_wt,
            orig_to_const_rep,
            const_to_orig,
        }
    }

    /// 顶点 u 的出边：返回 (dst_slice, wt_slice)
    #[inline(always)]
    pub fn neighbors(&self, u: usize) -> (&[u32], &[u32]) {
        let start = self.adj_off[u] as usize;
        let end = self.adj_off[u + 1] as usize;
        (&self.adj_dst[start..end], &self.adj_wt[start..end])
    }

    /// 常度数图点数
    #[inline]
    pub fn const_n(&self) -> usize {
        self.adj_off.len() - 1
    }

    /// 原图点数
    #[inline]
    pub fn orig_n(&self) -> usize {
        self.orig_to_const_rep.len()
    }

    /// 原图点 `v` 映射到常度数图中的一个代表点
    #[inline]
    pub fn orig_to_const(&self, v: usize) -> Option<usize> {
        self.orig_to_const_rep.get(v).map(|&x| x as usize)
    }

    /// 常度数图点 `x` 属于哪个原图点
    #[inline]
    pub fn const_to_orig(&self, x: usize) -> Option<usize> {
        self.const_to_orig.get(x).map(|&x| x as usize)
    }

    /// 返回兼容旧 API 的邻接表（仅用于测试/外部 dijkstra）
    pub fn to_adj_list(&self) -> Vec<Vec<(usize, usize)>> {
        let n = self.const_n();
        let mut adj = vec![Vec::new(); n];
        #[expect(clippy::needless_range_loop)]
        for u in 0..n {
            let (dsts, wts) = self.neighbors(u);
            for i in 0..dsts.len() {
                adj[u].push((dsts[i] as usize, wts[i] as usize));
            }
        }
        adj
    }
}

#[cfg(test)]
mod tests {
    use super::ConstGraph;
    use crate::algorithms::ssp::dijkstra;

    #[test]
    fn transformation_preserves_shortest_paths_on_representatives() {
        let g = vec![
            vec![(1, 2), (2, 5)],
            vec![(2, 1), (3, 4)],
            vec![(3, 1)],
            vec![],
        ];

        let cg = ConstGraph::from_general_graph(&g);
        let s = 0usize;
        let s2 = cg.orig_to_const(s).unwrap();

        let d1 = dijkstra(&g, s);
        let adj_list = cg.to_adj_list();
        let d2 = dijkstra(&adj_list, s2);

        for v in 0..g.len() {
            let rv = cg.orig_to_const(v).unwrap();
            assert_eq!(d2[rv], d1[v], "mismatch on vertex {v}");
        }
    }
}
