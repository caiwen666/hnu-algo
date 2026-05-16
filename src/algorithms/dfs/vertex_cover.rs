use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// 无向简单图上的顶点权最小权顶点覆盖问题。
#[derive(Debug, Clone)]
pub struct WeightedVertexCover {
    vertex_count: usize,
    edges: Vec<(usize, usize)>,
    weight: Vec<u64>,
}

/// 一组最优覆盖及其权值和。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexCoverSolution {
    /// 覆盖集中顶点（升序）。
    pub vertices: Vec<usize>,
    /// \(\sum_{v\in U} w(v)\)。
    pub total_weight: u128,
}

impl WeightedVertexCover {
    /// # Parameters
    ///
    /// - `vertex_count`: 顶点个数 \(n\)。
    /// - `edges`: 边列表 `(u, v)`，边为无向边。
    /// - `weight`: 各顶点权值。
    ///
    /// # Returns
    ///
    /// 返回一个 `WeightedVertexCover` 实例。
    ///
    /// # Panics
    ///
    /// - `vertex_count` 与 `weight.len()` 不一致时 panic。
    /// - `edges` 中有非法顶点时 panic。
    ///
    /// # Preconditions
    ///
    /// `edges` 不应该有重边和自环，否则可能会导致未定义行为。
    pub fn new(vertex_count: usize, edges: Vec<(usize, usize)>, weight: Vec<u64>) -> Self {
        assert_eq!(
            weight.len(),
            vertex_count,
            "weight.len() must equal vertex_count"
        );
        for &(u, v) in &edges {
            if u >= vertex_count || v >= vertex_count {
                panic!("invalid vertex index");
            }
        }
        Self {
            vertex_count,
            edges,
            weight,
        }
    }

    /// 使用优先队列分支限界法求解最小权顶点覆盖问题。
    ///
    /// 其中预估下界为 \(\mathrm{LB}=W+\underline h\)，
    /// 其中 \(W\) 为当前已选顶点权值和，
    /// \(\underline h\) 为贪心极大匹配的 \(\sum\min\)。
    ///
    /// # Examples
    ///
    /// ```
    /// # use hnu_algo::algorithms::dfs::vertex_cover::WeightedVertexCover;
    /// let g = WeightedVertexCover::new(2, vec![(0, 1)], vec![4, 7]);
    /// let sol = g.min_weight_vertex_cover_branch_bound();
    /// assert_eq!(sol.total_weight, 4);
    /// assert_eq!(sol.vertices, vec![0]);
    /// ```
    pub fn min_weight_vertex_cover_branch_bound(&self) -> VertexCoverSolution {
        let n = self.vertex_count;
        let edges = normalize_edges(&self.edges);

        if edges.is_empty() {
            return VertexCoverSolution {
                vertices: Vec::new(),
                total_weight: 0,
            };
        }

        #[derive(Clone)]
        struct Node {
            /// 预估下界
            lb: u128,
            /// 当前已选顶点权值和
            fixed_weight: u128,
            /// 当前已选顶点
            chosen: Vec<u8>,
            /// 未覆盖边
            uncovered: Vec<(usize, usize)>,
        }

        // 构造初始节点
        let chosen0 = vec![0_u8; n];
        let lb0 = lower_bound(0, &edges, n, &self.weight);
        let mut pool: Vec<Node> = vec![Node {
            lb: lb0,
            fixed_weight: 0,
            chosen: chosen0,
            uncovered: edges,
        }];

        let mut heap: BinaryHeap<Reverse<(u128, u64, usize)>> = BinaryHeap::new();
        heap.push(Reverse((lb0, 0_u64, 0_usize)));

        let mut best_weight = u128::MAX;
        let mut best_chosen: Vec<u8> = Vec::new();
        let mut seq_ctr = 1_u64;

        while let Some(Reverse((_lb_key, _tie, idx))) = heap.pop() {
            let node = pool[idx].clone();
            if node.lb >= best_weight {
                continue;
            }

            if node.uncovered.is_empty() {
                if node.fixed_weight < best_weight {
                    best_weight = node.fixed_weight;
                    best_chosen = node.chosen;
                }
                continue;
            }

            let Some((u, v)) = pick_branch_edge(&node.uncovered, n) else {
                continue;
            };

            // Branch 1: take u
            {
                let mut ch = node.chosen.clone();
                ch[u] = 1;
                let unc: Vec<_> = node
                    .uncovered
                    .iter()
                    .copied()
                    .filter(|&(a, b)| a != u && b != u)
                    .collect();
                let fw = node.fixed_weight + u128::from(self.weight[u]);
                let lb = lower_bound(fw, &unc, n, &self.weight);
                if lb < best_weight {
                    seq_ctr += 1;
                    let id = pool.len();
                    pool.push(Node {
                        lb,
                        fixed_weight: fw,
                        chosen: ch,
                        uncovered: unc,
                    });
                    heap.push(Reverse((lb, seq_ctr, id)));
                }
            }

            // Branch 2: take v
            {
                let mut ch = node.chosen.clone();
                ch[v] = 1;
                let unc: Vec<_> = node
                    .uncovered
                    .iter()
                    .copied()
                    .filter(|&(a, b)| a != v && b != v)
                    .collect();
                let fw = node.fixed_weight + u128::from(self.weight[v]);
                let lb = lower_bound(fw, &unc, n, &self.weight);
                if lb < best_weight {
                    seq_ctr += 1;
                    let id = pool.len();
                    pool.push(Node {
                        lb,
                        fixed_weight: fw,
                        chosen: ch,
                        uncovered: unc,
                    });
                    heap.push(Reverse((lb, seq_ctr, id)));
                }
            }
        }

        let mut vertices = Vec::new();
        for i in 0..n {
            if best_chosen.get(i).copied().unwrap_or(0) == 1 {
                vertices.push(i);
            }
        }

        VertexCoverSolution {
            total_weight: best_weight,
            vertices,
        }
    }
}

/// 将无向边中小编号顶点放到前面
fn normalize_edges(edges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    edges
        .iter()
        .map(|&(u, v)| if u <= v { (u, v) } else { (v, u) })
        .collect()
}

/// 选择一个未被覆盖的边
fn pick_branch_edge(edges: &[(usize, usize)], n: usize) -> Option<(usize, usize)> {
    if edges.is_empty() {
        return None;
    }
    let mut deg = vec![0_usize; n];
    for &(u, v) in edges {
        if u < n {
            deg[u] += 1;
        }
        if v < n {
            deg[v] += 1;
        }
    }
    // 选择度数乘积最大的
    edges
        .iter()
        .copied()
        .filter(|&(u, v)| u < n && v < n)
        .max_by_key(|&(u, v)| deg[u].saturating_mul(deg[v]))
}

/// 计算预估下界
fn lower_bound(
    fixed_weight: u128,
    uncovered: &[(usize, usize)],
    vertex_count: usize,
    weight: &[u64],
) -> u128 {
    // 贪心极大匹配预估下界
    let mut used = vec![false; vertex_count];
    let mut sum = 0_u128;
    for &(u, v) in uncovered {
        if !used[u] && !used[v] {
            used[u] = true;
            used[v] = true;
            sum += u128::from(weight[u].min(weight[v]));
        }
    }
    fixed_weight + sum
}

#[cfg(test)]
mod tests {
    use super::WeightedVertexCover;

    #[test]
    fn single_edge_picks_cheaper_endpoint() {
        let n = 2;
        let edges = vec![(0, 1)];
        let w = vec![4, 7];
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 4);
        assert_eq!(sol.vertices, vec![0]);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn triangle() {
        let n = 3;
        let edges = vec![(0, 1), (1, 2), (0, 2)];
        let w = vec![1, 1, 1];
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 2);
        assert_eq!(sol.vertices.len(), 2);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn no_edges() {
        let g = WeightedVertexCover::new(3, vec![], vec![5, 6, 7]);
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 0);
        assert!(sol.vertices.is_empty());
    }

    fn min_vertex_cover_bruteforce(n: usize, edges: &[(usize, usize)], weight: &[u64]) -> u128 {
        assert_eq!(weight.len(), n);
        assert!(n <= 20, "bruteforce n too large");
        let mut best = u128::MAX;
        for mask in 0usize..(1usize << n) {
            let mut ok = true;
            for &(u, v) in edges {
                if (mask >> u) & 1 == 0 && (mask >> v) & 1 == 0 {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            let mut sum = 0u128;
            for i in 0..n {
                if (mask >> i) & 1 != 0 {
                    sum += u128::from(weight[i]);
                }
            }
            best = best.min(sum);
        }
        best
    }

    fn assert_is_vertex_cover(n: usize, edges: &[(usize, usize)], vertices: &[usize]) {
        let mut in_cov = vec![false; n];
        for &v in vertices {
            assert!(v < n);
            in_cov[v] = true;
        }
        for &(u, v) in edges {
            assert!(
                in_cov[u] || in_cov[v],
                "edge ({u},{v}) not covered by {:?}",
                vertices
            );
        }
    }

    fn assert_solution_consistent(
        n: usize,
        edges: &[(usize, usize)],
        weight: &[u64],
        sol: &super::VertexCoverSolution,
    ) {
        assert_is_vertex_cover(n, edges, &sol.vertices);
        let sum: u128 = sol.vertices.iter().map(|&i| u128::from(weight[i])).sum();
        assert_eq!(sol.total_weight, sum);
    }

    #[test]
    fn complete_graph_uniform_weight() {
        let n = 6;
        let mut edges = Vec::new();
        for u in 0..n {
            for v in (u + 1)..n {
                edges.push((u, v));
            }
        }
        let w = vec![1u64; n];
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 5);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn star_prefers_leaves_when_cheaper() {
        let n = 8;
        let edges: Vec<_> = (1..n).map(|i| (0, i)).collect();
        let mut w = vec![1u64; n];
        w[0] = 100;
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 7);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn star_prefers_center_when_cheaper() {
        let n = 7;
        let edges: Vec<_> = (1..n).map(|i| (0, i)).collect();
        let mut w = vec![1u64; n];
        w[0] = 3;
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, 3);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn odd_cycle_c7_uniform() {
        let n = 7;
        let edges: Vec<_> = (0..n).map(|i| (i, (i + 1) % n)).collect();
        let w = vec![1u64; n];
        let brute = min_vertex_cover_bruteforce(n, &edges, &w);
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, brute);
        assert_eq!(sol.total_weight, 4);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn grid_2x3_dense_weights() {
        let n = 6;
        let edges = vec![
            (0, 1),
            (1, 2),
            (3, 4),
            (4, 5),
            (0, 3),
            (1, 4),
            (2, 5),
        ];
        let w = vec![4, 9, 2, 7, 3, 8];
        let brute = min_vertex_cover_bruteforce(n, &edges, &w);
        let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(sol.total_weight, brute);
        assert_solution_consistent(n, &edges, &w, &sol);
    }

    #[test]
    fn random_small_graphs_match_bruteforce() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        for _ in 0..80 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let n = 2 + (state % 11) as usize;
            let p_edge = ((state >> 8) % 100) as usize;
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if (state % 100) < p_edge as u64 {
                        edges.push((u, v));
                    }
                }
            }
            let mut weight = vec![0u64; n];
            for i in 0..n {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                weight[i] = 1 + (state % 30) as u64;
            }
            let brute = min_vertex_cover_bruteforce(n, &edges, &weight);
            let g = WeightedVertexCover::new(n, edges.clone(), weight.clone());
            let sol = g.min_weight_vertex_cover_branch_bound();
            assert_eq!(
                sol.total_weight, brute,
                "n={n} p={p_edge} |E|={} w={weight:?}",
                edges.len()
            );
            assert_solution_consistent(n, &edges, &weight, &sol);
        }
    }

    #[test]
    fn matches_bruteforce_small_instances() {
        let cases: Vec<(usize, Vec<(usize, usize)>, Vec<u64>)> = vec![
            (1, vec![], vec![9]),
            (2, vec![(0, 1)], vec![3, 5]),
            (4, vec![(0, 1), (1, 2), (2, 3)], vec![2, 4, 1, 3]),
            (4, vec![(0, 1), (0, 2), (1, 3), (2, 3)], vec![5, 2, 2, 5]),
        ];
        for (n, edges, w) in cases {
            let g = WeightedVertexCover::new(n, edges.clone(), w.clone());
            let sol = g.min_weight_vertex_cover_branch_bound();
            let brute = min_vertex_cover_bruteforce(n, &edges, &w);
            assert_eq!(sol.total_weight, brute, "n={n} e={edges:?}");
            if !edges.is_empty() || !sol.vertices.is_empty() {
                assert_solution_consistent(n, &edges, &w, &sol);
            } else {
                assert_eq!(sol.total_weight, 0);
                assert!(sol.vertices.is_empty());
            }
        }
    }
}
