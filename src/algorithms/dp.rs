#[derive(Debug, PartialEq)]
pub struct SimpleKnapsackItem {
    // 物品的代价
    pub weight: usize,
    // 物品的收益
    pub value: usize,
}
/// 0-1 背包问题求解
///
/// 如果有 $n$ 个物品，背包的容量为 $C$，则该算法的时间复杂度为 $O(nC)$，空间复杂度为 $O(nC)$。
///
/// # Arguments
///
/// - `items`: 物品列表
/// - `capacity`: 背包的容量
/// - `force_full`: 是否强制背包填满
///
/// # Returns
///
/// 返回一个物品列表，使得物品的代价之和不超过背包的容量，且物品的收益之和最大。
///
/// 返回物品列表中物品的顺序是按照 `items` 中的顺序排列的。
///
/// 如果设置了 `force_full`，如果给定物品无法把背包刚好填满，则返回一个空列表。否则，返回的列表内的物品代价之和必然等于背包的容量。
///
/// # Panics
///
/// 该函数会申请一块大小为 `(n + 1) * (capacity + 1)` 的 usize 数组用于执行 dp 算法。如果 `capacity` 过大，可能会导致内存分配失败从而 panic。
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::dp::simple_knapsack;
/// # use hnu_algo::algorithms::dp::SimpleKnapsackItem;
/// let items = vec![
///     SimpleKnapsackItem { weight: 71, value: 100 },
///     SimpleKnapsackItem { weight: 69, value: 1 },
///     SimpleKnapsackItem { weight: 1, value: 2 },
/// ];
/// let result = simple_knapsack(&items, 70, false);
///
/// assert_eq!(result, vec![&items[1], &items[2]]);
/// ```
///
/// ```rust
/// # use hnu_algo::algorithms::dp::simple_knapsack;
/// # use hnu_algo::algorithms::dp::SimpleKnapsackItem;
/// let items = vec![
///     SimpleKnapsackItem { weight: 2, value: 1 },
///     SimpleKnapsackItem { weight: 4, value: 999 },
///     SimpleKnapsackItem { weight: 3, value: 2 },
/// ];
/// let result1 = simple_knapsack(&items, 5, true);
/// assert_eq!(result1, vec![&items[0], &items[2]]);
///
/// let result2 = simple_knapsack(&items, 5, false);
/// assert_eq!(result2, vec![&items[1]]);
/// ```
pub fn simple_knapsack(
    items: &[SimpleKnapsackItem],
    capacity: usize,
    force_full: bool,
) -> Vec<&SimpleKnapsackItem> {
    let n = items.len();
    // dp[i][j]：考虑前 i 个物品、容量 j 时的最大收益。None 表示无法构成
    let mut dp: Vec<Vec<Option<usize>>> = vec![vec![None; capacity + 1]; n + 1];
    dp[0][0] = Some(0);

    for (i, item) in items.iter().enumerate() {
        let idx = i + 1;
        for j in 0..=capacity {
            dp[idx][j] = dp[idx - 1][j];
            if j >= item.weight
                && let Some(prev_value) = dp[idx - 1][j - item.weight]
            {
                let new_value = prev_value + item.value;
                if dp[idx][j].is_none_or(|v| new_value > v) {
                    dp[idx][j] = Some(new_value);
                }
            }
        }
    }

    let mut now_p = if force_full {
        if dp[n][capacity].is_none() {
            return Vec::new();
        } else {
            capacity
        }
    } else {
        // 寻找能够得到最大收益的背包容量
        dp[n]
            .iter()
            .enumerate()
            .filter_map(|(size, value)| value.as_ref().map(|v| (size, *v)))
            .max_by_key(|&(_size, value)| value)
            .map(|(size, _value)| size)
            .unwrap_or(0)
    };

    let mut result = Vec::new();
    for idx in (1..=n).rev() {
        let item = &items[idx - 1];
        if now_p >= item.weight
            && let Some(prev_value) = dp[idx - 1][now_p - item.weight]
        {
            // 当前状态能够由前面的这个转移过来，说明 item 被选中
            if dp[idx][now_p] == Some(prev_value + item.value) {
                result.push(item);
                now_p -= item.weight;
            }
        }
    }
    result.reverse();
    result
}
