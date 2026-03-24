use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// 使用 Dijkstra 算法求解单源最短路。
///
/// 使用优先队列实现，时间复杂度 O((|E| + |V|) log |V|)
///
/// # Parameters
///
/// - `graph`：邻接表，`graph[u]` 中每个元素是 `(v, w)`，表示 `u -> v` 的边及其非负权值。
/// - `source`：源点下标。
///
/// # Returns
///
/// 返回长度为 `graph.len()` 的距离数组：
/// - 可达点返回最短距离
/// - 不可达点返回 `u64::MAX`
///
/// 若 `source` 越界或图为空，则返回全 `u64::MAX` 的数组（空图返回空数组）。
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::ssp::dijkstra;
/// let graph = vec![
///     vec![(1, 2), (2, 5)],
///     vec![(2, 1), (3, 4)],
///     vec![(3, 1)],
///     vec![],
/// ];
/// let dist = dijkstra(&graph, 0);
/// assert_eq!(dist, vec![0, 2, 3, 4]);
/// ```
pub fn dijkstra(graph: &[Vec<(usize, usize)>], source: usize) -> Vec<u64> {
    let n = graph.len();
    let mut dist = vec![u64::MAX; n];
    if source >= n {
        return dist;
    }

    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    dist[source] = 0;
    heap.push(Reverse((0, source)));

    while let Some(Reverse((cur_dist, u))) = heap.pop() {
        // 堆中可能存在过期状态，直接丢弃即可。
        if cur_dist != dist[u] {
            continue;
        }
        for &(v, w) in &graph[u] {
            if v >= n {
                continue;
            }
            let Some(next_dist) = cur_dist.checked_add(w as u64) else {
                continue;
            };
            if next_dist < dist[v] {
                dist[v] = next_dist;
                heap.push(Reverse((next_dist, v)));
            }
        }
    }

    dist
}

/// 使用 SPFA 算法求解单源最短路（不检测负环）。
///
/// 没有带任何优化。
///
/// # Parameters
///
/// - `graph`：邻接表，`graph[u]` 中每个元素是 `(v, w)`，表示 `u -> v` 的边及权值。
/// - `source`：源点下标。
///
/// # Returns
///
/// 返回长度为 `graph.len()` 的距离数组：
/// - 可达点返回最短距离（在无负环可达的情况下）
/// - 不可达点返回 `u64::MAX`
///
/// 若 `source` 越界或图为空，则返回全 `u64::MAX` 的数组（空图返回空数组）。
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::ssp::spfa;
/// let graph = vec![
///     vec![(1, 2), (2, 5)],
///     vec![(2, 1), (3, 4)],
///     vec![(3, 1)],
///     vec![],
/// ];
/// let dist = spfa(&graph, 0);
/// assert_eq!(dist, vec![0, 2, 3, 4]);
/// ```
pub fn spfa(graph: &[Vec<(usize, usize)>], source: usize) -> Vec<u64> {
    let n = graph.len();
    let mut dist = vec![u64::MAX; n];
    if source >= n {
        return dist;
    }

    let mut queue = VecDeque::new();
    let mut in_queue = vec![false; n];

    dist[source] = 0;
    queue.push_back(source);
    in_queue[source] = true;

    while let Some(u) = queue.pop_front() {
        in_queue[u] = false;
        let base = dist[u];
        if base == u64::MAX {
            continue;
        }

        for &(v, w) in &graph[u] {
            if v >= n {
                continue;
            }
            let Some(next_dist) = base.checked_add(w as u64) else {
                continue;
            };
            if next_dist < dist[v] {
                dist[v] = next_dist;
                if !in_queue[v] {
                    queue.push_back(v);
                    in_queue[v] = true;
                }
            }
        }
    }

    dist
}

#[cfg(test)]
mod tests {
    use super::{dijkstra, spfa};

    fn sample_graph() -> Vec<Vec<(usize, usize)>> {
        // 0 -> 1 (2), 0 -> 2 (5), 1 -> 2 (1), 1 -> 3 (4), 2 -> 3 (1)
        vec![
            vec![(1, 2), (2, 5)],
            vec![(2, 1), (3, 4)],
            vec![(3, 1)],
            vec![],
        ]
    }

    #[test]
    fn unreachable_nodes_are_max() {
        let graph = vec![vec![(1, 1)], vec![], vec![]];
        let d1 = dijkstra(&graph, 0);
        let d2 = spfa(&graph, 0);
        assert_eq!(d1, vec![0, 1, u64::MAX]);
        assert_eq!(d2, vec![0, 1, u64::MAX]);
    }

    #[test]
    fn out_of_range_source_and_empty_graph() {
        let graph = sample_graph();
        assert_eq!(dijkstra(&graph, 10), vec![u64::MAX; graph.len()]);
        assert_eq!(spfa(&graph, 10), vec![u64::MAX; graph.len()]);

        let empty: Vec<Vec<(usize, usize)>> = vec![];
        assert!(dijkstra(&empty, 0).is_empty());
        assert!(spfa(&empty, 0).is_empty());
    }
}
