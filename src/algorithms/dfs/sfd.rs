//! 无分隔符字典（SeparatorFreeDictionary, sfd）问题

/// 将长度为 `k` 的符号序列（每位在 `0..alphabet_size`）映射为 \([0, n^k)\) 中的整数。
pub fn pack_word(symbols: &[usize], alphabet_size: usize) -> usize {
    let mut id = 0_usize;
    for &x in symbols {
        debug_assert!(x < alphabet_size);
        id = id * alphabet_size + x;
    }
    id
}

/// [`pack_word`] 的逆。
pub fn unpack_word(mut id: usize, alphabet_size: usize, str_len: usize) -> Vec<usize> {
    let mut v = vec![0_usize; str_len];
    for i in (0..str_len).rev() {
        v[i] = id % alphabet_size;
        id /= alphabet_size;
    }
    v
}

/// 对有序串对 \((a,b)\)（均已编码）生成题设中的 \(k-1\) 个重叠串（长度均为 \(k\)）的编码。
fn overlap_ids(a: usize, b: usize, alphabet_size: usize, str_len: usize) -> Vec<usize> {
    let xa = unpack_word(a, alphabet_size, str_len);
    let xb = unpack_word(b, alphabet_size, str_len);
    let k = str_len;
    let mut out = Vec::with_capacity(k - 1);
    for r in 1..k {
        let mut sym = Vec::with_capacity(k);
        sym.extend_from_slice(&xa[r..k]);
        sym.extend_from_slice(&xb[0..r]);
        debug_assert_eq!(sym.len(), k);
        out.push(pack_word(&sym, alphabet_size));
    }
    out
}

fn dfs(
    idx: usize,
    n: usize,
    k: usize,
    total: usize,
    cur: &mut Vec<usize>,
    in_set: &mut [bool],
    exclude_count: &mut [usize],
    best: &mut usize,
) {
    let cur_len = cur.len();
    if cur_len + (total - idx) <= *best {
        return;
    }
    if idx == total {
        *best = (*best).max(cur_len);
        return;
    }

    if exclude_count[idx] > 0 {
        dfs(idx + 1, n, k, total, cur, in_set, exclude_count, best);
        return;
    }

    // 不选当前串
    dfs(idx + 1, n, k, total, cur, in_set, exclude_count, best);

    // 选入当前串 `idx`
    cur.push(idx);
    in_set[idx] = true;

    let mut valid = true;
    'check: {
        for &y in cur.iter() {
            for o in overlap_ids(idx, y, n, k) {
                if in_set[o] {
                    valid = false;
                    break 'check;
                }
            }
            for o in overlap_ids(y, idx, n, k) {
                if in_set[o] {
                    valid = false;
                    break 'check;
                }
            }
        }
        for &x in cur.iter() {
            if x == idx {
                continue;
            }
            for &y in cur.iter() {
                if y == idx {
                    continue;
                }
                for o in overlap_ids(x, y, n, k) {
                    if o == idx {
                        valid = false;
                        break 'check;
                    }
                }
            }
        }
    }

    if valid {
        let mut stack = Vec::new();
        for &y in cur.iter() {
            for o in overlap_ids(idx, y, n, k) {
                if o > idx {
                    exclude_count[o] += 1;
                    stack.push(o);
                }
            }
        }
        for &x in cur.iter().filter(|&&x| x != idx) {
            for o in overlap_ids(x, idx, n, k) {
                if o > idx {
                    exclude_count[o] += 1;
                    stack.push(o);
                }
            }
        }
        dfs(idx + 1, n, k, total, cur, in_set, exclude_count, best);
        while let Some(o) = stack.pop() {
            exclude_count[o] -= 1;
        }
    }

    in_set[idx] = false;
    cur.pop();
}

/// 见 [`max_sfd_size`] 的说明
pub const MAX_SEPARATOR_SEARCH_STATES: usize = 1 << 22;

/// 求出无分隔符字典最大大小。
///
/// # Parameters
///
/// - `alphabet_size`: 字母表大小 \(n\ge 1\)。
/// - `str_len`: 串长 \(k\ge 1\)。
///
/// # Returns
///
/// 返回大小为 `alphabet_size` 的字母表构成的所有长度为 `str_len` 的字符串中，
/// 无分隔符字典的最大大小。
///
/// # Panics
///
/// - 如果大小为 `alphabet_size` 的字母表构成的所有长度为 `str_len` 的字符串
/// 数量超过了 [`MAX_SEPARATOR_SEARCH_STATES`]，则 panic。
///
/// # Examples
///
/// ```
/// # use hnu_algo::algorithms::dfs::sfd::max_sfd_size;
/// assert_eq!(max_sfd_size(2, 2), 1);
/// ```
pub fn max_sfd_size(alphabet_size: usize, str_len: usize) -> usize {
    let n = alphabet_size;
    let k = str_len;
    if n == 0 || k == 0 {
        return 0;
    }
    let total = n.checked_pow(k as u32).expect("total is too large");
    if total > MAX_SEPARATOR_SEARCH_STATES {
        panic!("total is too large");
    }

    let mut exclude_count = vec![0usize; total];
    let mut in_set = vec![false; total];
    let mut cur = Vec::new();
    let mut best = 0_usize;

    dfs(
        0,
        n,
        k,
        total,
        &mut cur,
        &mut in_set,
        &mut exclude_count,
        &mut best,
    );
    best
}

#[cfg(test)]
mod tests {
    use super::{max_sfd_size, pack_word, unpack_word};

    /// 与实现中 [`super::overlap_ids`] 同义，供暴力对照（测试模块无法访问私有函数）。
    fn overlap_ids_naive(a: usize, b: usize, n: usize, k: usize) -> Vec<usize> {
        let xa = unpack_word(a, n, k);
        let xb = unpack_word(b, n, k);
        let mut out = Vec::with_capacity(k.saturating_sub(1));
        for r in 1..k {
            let mut sym = Vec::with_capacity(k);
            sym.extend_from_slice(&xa[r..k]);
            sym.extend_from_slice(&xb[0..r]);
            out.push(pack_word(&sym, n));
        }
        out
    }

    /// 按题设：对 \(S\) 中任意有序对 \((a,b)\)，其 \(k-1\) 个重叠串均不在 \(S\) 中。
    fn is_separator_free_subset(ids: &[usize], n: usize, k: usize, total: usize) -> bool {
        let mut in_s = vec![false; total];
        for &id in ids {
            in_s[id] = true;
        }
        for &x in ids {
            for &y in ids {
                for o in overlap_ids_naive(x, y, n, k) {
                    if in_s[o] {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// 子集暴力最大规模，仅用于 `n^k` 很小的情况。
    fn max_sfd_bruteforce(n: usize, k: usize) -> usize {
        let total = n.pow(k as u32);
        assert!(total <= 14, "bruteforce only for tiny universes");
        let mut best = 0usize;
        for mask in 0usize..(1usize << total) {
            let mut ids = Vec::new();
            for i in 0..total {
                if (mask >> i) & 1 != 0 {
                    ids.push(i);
                }
            }
            if is_separator_free_subset(&ids, n, k, total) {
                best = best.max(ids.len());
            }
        }
        best
    }

    #[test]
    fn bruteforce_matches_implementation_small_grid() {
        for n in 1usize..=3 {
            for k in 1usize..=3 {
                let total = n.pow(k as u32);
                if total <= 14 {
                    let a = max_sfd_size(n, k);
                    let b = max_sfd_bruteforce(n, k);
                    assert_eq!(a, b, "n={n} k={k} total={total}");
                }
            }
        }
    }

    #[test]
    fn n3_k2_max_is_three() {
        assert_eq!(max_sfd_size(3, 2), 3);
        assert_eq!(max_sfd_bruteforce(3, 2), 3);
    }

    #[test]
    fn k1_any_subset_is_valid_max_is_n() {
        for n in 1..=7 {
            assert_eq!(max_sfd_size(n, 1), n);
            assert_eq!(max_sfd_bruteforce(n, 1), n);
        }
    }

    #[test]
    fn n1_alphabet_edge_cases() {
        // k=1：无重叠约束，唯一字符的 n 条串互异，可全选；此处 n=1 只有 1 条串
        assert_eq!(max_sfd_size(1, 1), 1);
        // k≥2：全集只有一个串 id=0，但 (0,0) 的重叠会落在自身上，不能构成非空合法字典
        for k in 2..=6 {
            assert_eq!(max_sfd_size(1, k), 0);
        }
    }

    #[test]
    fn n2_k3_bruteforce_agrees() {
        assert_eq!(max_sfd_size(2, 3), max_sfd_bruteforce(2, 3));
    }

    #[test]
    fn overlap_known_instance_n2_k2() {
        // 00=0, 01=1, 10=2, 11=3；可手算 (0,3) 与 (3,0) 的重叠是否落在 {0,3}
        let n = 2;
        let k = 2;
        assert_eq!(overlap_ids_naive(0, 3, n, k), vec![1]);
        assert_eq!(overlap_ids_naive(3, 0, n, k), vec![2]);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let n: usize = 3;
        let k: usize = 4;
        for id in 0..n.pow(k as u32) {
            let w = unpack_word(id, n, k);
            assert_eq!(pack_word(&w, n), id);
        }
    }

    #[test]
    fn n2_k2_max_is_1() {
        assert_eq!(max_sfd_size(2, 2), 1);
    }

    #[test]
    fn singleton_always_ok() {
        assert!(max_sfd_size(3, 3) >= 1);
    }
}
