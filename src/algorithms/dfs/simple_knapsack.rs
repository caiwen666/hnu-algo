use crate::algorithms::dp::SimpleKnapsackItem;

/// 分数背包上界：在已按密度全局排好序的后缀 `ord[idx..]` 上，容量 `cap` 内可得的收益上界，再加 `base`。
fn fractional_upper_bound(
    items: &[SimpleKnapsackItem],
    ord: &[usize],
    pref_w: &[usize],
    pref_v: &[usize],
    idx: usize,
    cap: usize,
    base: usize,
) -> usize {
    let n = ord.len();
    let mut idx = idx;
    let mut base = base;
    while idx < n && items[ord[idx]].weight == 0 {
        base += items[ord[idx]].value;
        idx += 1;
    }
    if idx >= n || cap == 0 {
        return base;
    }
    let base_w = pref_w[idx];
    let mut lo = idx;
    let mut hi = n;
    let mut j = idx;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if pref_w[mid] - base_w <= cap {
            j = mid;
            if mid == n {
                break;
            }
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    let mut val = base + (pref_v[j] - pref_v[idx]);
    let used = pref_w[j] - base_w;
    let rem = cap - used;
    if j < n && rem > 0 {
        let w = items[ord[j]].weight;
        let v = items[ord[j]].value;
        if w > 0 {
            val += v * rem / w;
        }
    }
    val
}

fn dfs(
    items: &[SimpleKnapsackItem],
    ord: &[usize],
    pref_w: &[usize],
    pref_v: &[usize],
    capacity: usize,
    force_full: bool,
    depth: usize,
    cur_w: usize,
    cur_v: usize,
    cur_pick: &mut [bool],
    best_val: &mut usize,
    best_pick: &mut [bool],
) {
    let n = ord.len();
    if depth == n {
        if cur_w <= capacity {
            if force_full && cur_w != capacity {
                return;
            }
            if cur_v > *best_val {
                *best_val = cur_v;
                best_pick.copy_from_slice(cur_pick);
            }
        }
        return;
    }

    let cap_left = capacity - cur_w;
    let ub = fractional_upper_bound(items, ord, pref_w, pref_v, depth, cap_left, cur_v);
    if ub <= *best_val {
        return;
    }

    let orig = ord[depth];
    let w = items[orig].weight;
    let v = items[orig].value;
    if cur_w + w <= capacity {
        cur_pick[orig] = true;
        dfs(
            items,
            ord,
            pref_w,
            pref_v,
            capacity,
            force_full,
            depth + 1,
            cur_w + w,
            cur_v + v,
            cur_pick,
            best_val,
            best_pick,
        );
        cur_pick[orig] = false;
    }
    dfs(
        items,
        ord,
        pref_w,
        pref_v,
        capacity,
        force_full,
        depth + 1,
        cur_w,
        cur_v,
        cur_pick,
        best_val,
        best_pick,
    );
}

/// 用回溯法 + 分支界限法求 0-1 背包，接口与 [crate::algorithms::dp::simple_knapsack] 一致。
///
/// 实现要点：
///
/// - 按价值密度 \(v_i/w_i\) 非升序对物品下标排序（\(w_i=0\) 视为最高密度）
/// - 分支限界搜索按该顺序进行
/// - 上界：
///     - 当前价值 + 剩余容量在已排好序的后缀上的分数背包最优值
///     - 用前缀和 + 二分在 \(O(\log n)\) 内求出「整件拿满」的边界，再对下一件取分数部分，避免每次 DFS 对后缀重新分配向量并排序。
///
/// 时间复杂度最坏 \(O(2^n)\)。
///
/// # Examples
///
/// ```
/// # use hnu_algo::algorithms::dp::{simple_knapsack_backtracking, SimpleKnapsackItem};
/// let items = vec![
///     SimpleKnapsackItem { weight: 71, value: 100 },
///     SimpleKnapsackItem { weight: 69, value: 1 },
///     SimpleKnapsackItem { weight: 1, value: 2 },
/// ];
/// let r = simple_knapsack_backtracking(&items, 70, false);
/// assert_eq!(r, vec![&items[1], &items[2]]);
/// ```
pub fn simple_knapsack_backtracking(
    items: &[SimpleKnapsackItem],
    capacity: usize,
    force_full: bool,
) -> Vec<&SimpleKnapsackItem> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }

    let mut ord: Vec<usize> = (0..n).collect();
    ord.sort_by(|&a, &b| {
        let va = items[a].value * items[b].weight;
        let vb = items[b].value * items[a].weight;
        vb.cmp(&va).then_with(|| a.cmp(&b))
    });

    let mut pref_w = vec![0usize; n + 1];
    let mut pref_v = vec![0usize; n + 1];
    for i in 0..n {
        let o = ord[i];
        pref_w[i + 1] = pref_w[i] + items[o].weight;
        pref_v[i + 1] = pref_v[i] + items[o].value;
    }

    let mut best_val: usize = 0;
    let mut best_pick = vec![false; n];
    let mut cur_pick = vec![false; n];

    dfs(
        items,
        &ord,
        &pref_w,
        &pref_v,
        capacity,
        force_full,
        0,
        0,
        0,
        &mut cur_pick,
        &mut best_val,
        &mut best_pick,
    );

    let mut result = Vec::new();
    for i in 0..n {
        if best_pick[i] {
            result.push(&items[i]);
        }
    }
    result
}

#[cfg(test)]
mod simple_knapsack_tests {
    use super::simple_knapsack_backtracking;
    use crate::algorithms::dp::{SimpleKnapsackItem, simple_knapsack};

    #[test]
    fn matches_dp_small_random() {
        for n in 1..=12 {
            let items: Vec<_> = (0..n)
                .map(|i| SimpleKnapsackItem {
                    weight: (i * 3 + 7) % 15 + 1,
                    value: (i * 5 + 11) % 20 + 1,
                })
                .collect();
            for cap in 0..=50 {
                let r1 = simple_knapsack(&items, cap, false);
                let r2 = simple_knapsack_backtracking(&items, cap, false);
                let v1: usize = r1.iter().map(|x| x.value).sum();
                let v2: usize = r2.iter().map(|x| x.value).sum();
                assert_eq!(v1, v2, "n={n} cap={cap}");
            }
        }
    }

    #[test]
    fn force_full_matches() {
        let items = vec![
            SimpleKnapsackItem {
                weight: 2,
                value: 1,
            },
            SimpleKnapsackItem {
                weight: 4,
                value: 999,
            },
            SimpleKnapsackItem {
                weight: 3,
                value: 2,
            },
        ];
        assert_eq!(
            simple_knapsack_backtracking(&items, 5, true),
            simple_knapsack(&items, 5, true)
        );
    }
}
