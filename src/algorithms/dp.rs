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

pub struct BatchScheduling {
    /// 每批开始前的启动时间
    startup: usize,
    /// 每个作业的单独加工时间
    duration: Vec<usize>,
    /// 每个作业的费用系数
    fee: Vec<usize>,
}

impl BatchScheduling {
    /// # Panics
    ///
    /// 若 `duration` 与 `fee` 不等长则 panic。
    pub fn new(startup: usize, duration: Vec<usize>, fee: Vec<usize>) -> Self {
        assert_eq!(
            duration.len(),
            fee.len(),
            "duration and fee must have the same length"
        );
        Self {
            startup,
            duration,
            fee,
        }
    }

    /// 将所有作业任务划分为连续的若干段，然后每段依次执行，
    /// 同一个段内的作业只需要支付一次启动费用，并依次执行，
    /// 每个作业的费用为完成时刻乘以费用系数。
    ///
    /// 时间复杂度 \(O(n^2)\)，空间复杂度 \(O(n)\)。
    ///
    /// # Returns
    ///
    /// 返回最小总费用。
    ///
    /// # Examples
    ///
    /// ```
    /// # use hnu_algo::algorithms::dp::BatchScheduling;
    /// let p = BatchScheduling::new(
    ///     1,
    ///     vec![1, 3, 4, 2, 1],
    ///     vec![3, 2, 3, 3, 4],
    /// );
    /// assert_eq!(p.solve(), 153);
    /// ```
    pub fn solve(&self) -> usize {
        if self.duration.is_empty() {
            return 0;
        }
        let n = self.duration.len();

        let mut prefix_t = vec![0; n + 1];
        let mut prefix_f = vec![0; n + 1];
        for i in 0..n {
            prefix_t[i + 1] = prefix_t[i] + self.duration[i];
            prefix_f[i + 1] = prefix_f[i] + self.fee[i];
        }
        let s = self.startup;

        let mut dp = vec![0; n + 1];
        for i in 1..=n {
            let t_i = prefix_t[i];
            dp[i] = usize::MAX;
            for j in 0..i {
                let prev = dp[j];
                if prev == usize::MAX {
                    continue;
                }
                let delta_f = prefix_f[i] - prefix_f[j];
                let tail_startup = s * (prefix_f[n] - prefix_f[j]);
                let cand = prev + t_i * delta_f + tail_startup;
                if cand < dp[i] {
                    dp[i] = cand;
                }
            }
        }
        dp[n]
    }
}

#[cfg(test)]
mod batch_scheduling_tests {
    use super::BatchScheduling;

    /// 与实验说明一致：按顺序分段，每批先付启动 `startup`，批内加工时间累加；
    /// 该批中每个作业的完成时刻均为「批开始时刻 + startup + 批内 t 之和」。
    /// 枚举 `2^(n-1)` 种分段，用于小 `n` 对照 [`BatchScheduling::solve`]。
    fn reference_optimal(startup: usize, duration: &[usize], fee: &[usize]) -> usize {
        let n = duration.len();
        if n == 0 {
            return 0;
        }
        let mut best = usize::MAX;
        let cuts = n - 1;
        for mask in 0..(1usize << cuts) {
            let mut clock = 0usize;
            let mut total = 0usize;
            let mut start = 0usize;
            while start < n {
                clock += startup;
                let mut end = start;
                loop {
                    clock += duration[end];
                    end += 1;
                    if end >= n {
                        break;
                    }
                    if (mask >> (end - 1)) & 1 != 0 {
                        break;
                    }
                }
                for k in start..end {
                    total += clock * fee[k];
                }
                start = end;
            }
            best = best.min(total);
        }
        best
    }

    fn single_batch_upper_bound(startup: usize, duration: &[usize], fee: &[usize]) -> usize {
        let sum_t: usize = duration.iter().sum();
        let sum_f: usize = fee.iter().sum();
        (startup + sum_t) * sum_f
    }

    #[test]
    fn sample_from_statement() {
        let p = BatchScheduling::new(1, vec![1, 3, 4, 2, 1], vec![3, 2, 3, 3, 4]);
        assert_eq!(p.solve(), 153);
    }

    #[test]
    fn single_job() {
        let p = BatchScheduling::new(5, vec![10], vec![7]);
        // one batch: completion = S + t = 15, cost = 15 * 7 = 105
        assert_eq!(p.solve(), 105);
    }

    #[test]
    fn two_jobs_one_batch() {
        let p = BatchScheduling::new(2, vec![1, 2], vec![3, 4]);
        // completion both = 2 + 3 = 5, cost = 5*3 + 5*4 = 35
        assert_eq!(p.solve(), 35);
    }

    #[test]
    fn statement_scheme_cost_matches_reference() {
        // task.md 中方案 {1,2},{3},{4,5} 完成时刻 (5,5,10,14,14)，总费用 153
        let startup = 1usize;
        let duration = [1usize, 3, 4, 2, 1];
        let fee = [3usize, 2, 3, 3, 4];
        let mut clock = 0usize;
        let mut total = 0usize;
        // batch 0..2
        clock += startup + duration[0] + duration[1];
        for k in 0..2 {
            total += clock * fee[k];
        }
        // batch 2..3
        clock += startup + duration[2];
        total += clock * fee[2];
        // batch 3..5
        clock += startup + duration[3] + duration[4];
        for k in 3..5 {
            total += clock * fee[k];
        }
        assert_eq!(total, 153);
        assert_eq!(reference_optimal(startup, &duration, &fee), 153);
    }

    #[test]
    fn multiple_batches_better_than_one() {
        // 较大 S 下前面轻权、后面重权：拆批让重权作业不必承担整批长加工时间
        let startup = 5usize;
        let duration = vec![1usize, 1, 100];
        let fee = vec![50usize, 50, 1];
        let one = single_batch_upper_bound(startup, &duration, &fee);
        let p = BatchScheduling::new(startup, duration.clone(), fee.clone());
        let opt = p.solve();
        assert!(opt < one, "expected splitting to beat single batch");
        assert_eq!(opt, reference_optimal(startup, &duration, &fee));
    }

    #[test]
    fn ordering_matters_same_aggregate_different_sequence() {
        let startup = 1usize;
        let p_short_heavy_first = BatchScheduling::new(startup, vec![1, 10], vec![10, 1]);
        let p_long_heavy_first = BatchScheduling::new(startup, vec![10, 1], vec![1, 10]);
        assert_eq!(p_short_heavy_first.solve(), 33);
        assert_eq!(p_long_heavy_first.solve(), 132);
        assert_ne!(
            p_short_heavy_first.solve(),
            p_long_heavy_first.solve(),
            "same multiset (t,w) swapped order should change optimum"
        );
    }

    #[test]
    fn zero_startup_favors_more_batches() {
        let startup = 0usize;
        let n = 6usize;
        let duration = vec![1usize; n];
        let fee = vec![1usize; n];
        let p = BatchScheduling::new(startup, duration.clone(), fee.clone());
        let opt = p.solve();
        // 单批：全员在同一完成时刻 n；拆成 n 批：完成时刻 1..=n，总费用 n(n+1)/2
        assert_eq!(opt, n * (n + 1) / 2);
        assert_eq!(opt, reference_optimal(startup, &duration, &fee));
    }

    #[test]
    fn larger_n_deterministic_matches_reference() {
        let startup = 3usize;
        let n = 14usize;
        let duration: Vec<usize> = (0..n).map(|i| (i * 7 + 11) % 9 + 1).collect();
        let fee: Vec<usize> = (0..n).map(|i| (i * 5 + 13) % 12 + 1).collect();
        let p = BatchScheduling::new(startup, duration.clone(), fee.clone());
        assert_eq!(p.solve(), reference_optimal(startup, &duration, &fee));
    }

    #[test]
    fn larger_n_fast_and_le_single_batch_bound() {
        let startup = 4usize;
        let n = 32usize;
        let duration: Vec<usize> = (0..n).map(|i| (i * 13 + 5) % 20 + 1).collect();
        let fee: Vec<usize> = (0..n).map(|i| (i * 11 + 3) % 15 + 1).collect();
        let p = BatchScheduling::new(startup, duration.clone(), fee.clone());
        let opt = p.solve();
        let naive = single_batch_upper_bound(startup, &duration, &fee);
        assert!(opt <= naive);
    }

    #[test]
    fn zero_weights_and_mixed_durations() {
        let startup = 2usize;
        let duration = vec![0usize, 5, 0, 3];
        let fee = vec![100usize, 0, 7, 0];
        let p = BatchScheduling::new(startup, duration.clone(), fee.clone());
        assert_eq!(p.solve(), reference_optimal(startup, &duration, &fee));
    }

    #[test]
    fn brute_matches_dp_exhaustive_grid() {
        for n in 1..=10usize {
            for s in 0..=3usize {
                let duration: Vec<usize> = (0..n).map(|i| (i * 3 + n) % 5).collect();
                let fee: Vec<usize> = (0..n).map(|i| (i + s + n) % 4).collect();
                let p = BatchScheduling::new(s, duration.clone(), fee.clone());
                assert_eq!(
                    p.solve(),
                    reference_optimal(s, &duration, &fee),
                    "n={n} s={s}"
                );
            }
        }
    }
}
