use std::collections::HashSet;

/// 计算一张图中的点的数量
/// # Arguments
///
/// - `data`: 图的边集，tuple 的元素为边的顶点
///
/// # Returns
///
/// 图中点的数量
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::utils::count_nodes;
/// let data = vec![(0, 1), (0, 2), (1, 2)];
/// let count = count_nodes(&data);
/// assert_eq!(count, 3);
/// ```
pub fn count_nodes<T>(data: &Vec<(T, T)>) -> usize
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut nodes = HashSet::new();
    for (source, target) in data {
        nodes.insert(source);
        nodes.insert(target);
    }
    nodes.len()
}
