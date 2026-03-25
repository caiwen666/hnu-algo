use std::collections::{BTreeMap, BTreeSet, HashMap};

/// 常度数图（入度/出度 ≤ 2），用于 BMSSP/论文算法的前置变换。
///
/// 论文（Duan et al. 2025）中的经典转换：
/// - 对每个原图点 `v`，把它替换成一圈 0 权有向环；对 `v` 的每个（入/出）相邻点 `w`，创建一个节点 `x_{v,w}` 放在环上；
/// - 对每条原图边 `(u -> v, w_uv)`，添加一条边 `x_{u,v} -> x_{v,u}`，权重为 `w_uv`。
///
/// 这样得到的新图满足每个点的入/出度最多为 2，且最短路保持（原图中从 `s` 到 `t` 的最短距离
/// 等于新图中从 `rep(s)` 到 `rep(t)` 的最短距离，其中 `rep(v)` 是 `v` 对应环上的任意代表点）。
#[derive(Debug, Clone)]
pub struct ConstGraph {
    /// 常度数图邻接表
    adj: Vec<Vec<(usize, usize)>>,
    /// 原图点 -> 常度数图中代表点（每个原图点只需一个代表点）
    orig_to_const_rep: Vec<usize>,
    /// 常度数图点 -> 属于哪个原图点的“替身环”
    const_to_orig: Vec<usize>,
}

impl ConstGraph {
    /// 直接用“已经是常度数图”的邻接表构造 `ConstGraph`。
    ///
    /// 该构造函数会设置“恒等映射”：认为原图点与常度数图点一一对应。
    pub fn new(adj: Vec<Vec<(usize, usize)>>) -> Self {
        let n = adj.len();
        Self {
            adj,
            orig_to_const_rep: (0..n).collect(),
            const_to_orig: (0..n).collect(),
        }
    }

    /// 从任意有向非负权图构造常度数图，并保留映射。
    ///
    /// 输入图格式与项目中最短路一致：`graph[u]` 是若干 `(v, w)`，表示 `u -> v` 权重 `w`。
    pub fn from_general_graph(graph: &[Vec<(usize, usize)>]) -> Self {
        let n = graph.len();

        // 1) 预处理：合并重边（同一 (u,v) 只保留最小权），以保证 x_{u,v} 只引出一条“原边”。
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

        // 2) 为每个 (v,w)（w 是 v 的入/出相邻点）创建一个常度数图点 x_{v,w}
        let mut pair_to_id: HashMap<(usize, usize), usize> = HashMap::new();
        let mut const_to_orig: Vec<usize> = Vec::new();
        let mut per_v_nodes: Vec<Vec<usize>> = vec![Vec::new(); n];

        for v in 0..n {
            let mut neigh: BTreeSet<usize> = BTreeSet::new();
            neigh.extend(in_set[v].iter().copied());
            neigh.extend(out_min[v].keys().copied());

            // 如果 v 完全没有相邻点，为了能提供映射/代表点，仍创建一个孤点（用 (v,v) 作为键）。
            if neigh.is_empty() {
                neigh.insert(v);
            }

            for w in neigh {
                let id = const_to_orig.len();
                const_to_orig.push(v);
                pair_to_id.insert((v, w), id);
                per_v_nodes[v].push(id);
            }
        }

        // 3) 构造常度数图邻接表
        let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); const_to_orig.len()];

        // 3.1) 每个 v 的替身点用 0 权有向环连接：保证每个点额外 in/out 各 +1
        #[expect(clippy::needless_range_loop)]
        for v in 0..n {
            let nodes = &per_v_nodes[v];
            if nodes.len() <= 1 {
                continue;
            }
            for i in 0..nodes.len() {
                let a = nodes[i];
                let b = nodes[(i + 1) % nodes.len()];
                adj[a].push((b, 0));
            }
        }

        // 3.2) 原图每条边 (u->v, w) => x_{u,v} -> x_{v,u}，权重 w
        #[expect(clippy::needless_range_loop)]
        for u in 0..n {
            for (&v, &w) in out_min[u].iter() {
                let from = *pair_to_id.get(&(u, v)).expect("x_{u,v} must exist");
                let to = *pair_to_id.get(&(v, u)).expect("x_{v,u} must exist");
                adj[from].push((to, w));
            }
        }

        // 4) 建立“原图点 -> 代表点”映射：取环上的第一个即可
        let mut orig_to_const_rep = vec![0usize; n];
        for v in 0..n {
            orig_to_const_rep[v] = per_v_nodes[v][0];
        }

        Self {
            adj,
            orig_to_const_rep,
            const_to_orig,
        }
    }

    /// 常度数图的邻接表
    pub fn adj(&self) -> &[Vec<(usize, usize)>] {
        &self.adj
    }

    /// 常度数图点数
    pub fn const_n(&self) -> usize {
        self.adj.len()
    }

    /// 原图点数
    pub fn orig_n(&self) -> usize {
        self.orig_to_const_rep.len()
    }

    /// 原图点 `v` 映射到常度数图中的一个代表点（`v` 越界则返回 `None`）。
    pub fn orig_to_const(&self, v: usize) -> Option<usize> {
        self.orig_to_const_rep.get(v).copied()
    }

    /// 常度数图点 `x` 属于哪个原图点（`x` 越界则返回 `None`）。
    pub fn const_to_orig(&self, x: usize) -> Option<usize> {
        self.const_to_orig.get(x).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::ConstGraph;
    use crate::algorithms::ssp::dijkstra;

    #[test]
    fn transformation_preserves_shortest_paths_on_representatives() {
        // 0 -> 1 (2), 0 -> 2 (5), 1 -> 2 (1), 1 -> 3 (4), 2 -> 3 (1)
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
        let d2 = dijkstra(cg.adj(), s2);

        for v in 0..g.len() {
            let rv = cg.orig_to_const(v).unwrap();
            assert_eq!(d2[rv], d1[v], "mismatch on vertex {v}");
        }
    }
}
