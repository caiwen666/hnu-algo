use crate::ds::BinaryIndexedTree;

/// 康托展开
///
/// 计算给定排列在同长度的全排列中的排名
///
/// # Preconditions
///
/// `permutation` 必须是一个 1 到 n 的排列。该函数并没有校验 `permutation` 是否合法，不合法的话会出现未定义行为。
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::misc::cantor_expansion;
///
/// let permutation = [2, 1, 3];
/// let rank = cantor_expansion(&permutation);
/// assert_eq!(rank, 3);
/// ```
pub fn cantor_expansion(permutation: &[usize]) -> usize {
    let mut rank = 0;
    let mut fac = 1;
    let n = permutation.len();
    let mut bit = BinaryIndexedTree::new(n);
    for (i, &x) in permutation.iter().enumerate().rev() {
        rank += bit.prefix_sum(x - 1) * fac;
        bit.add(x, 1);
        fac *= n - i;
    }
    rank + 1
}

/// 逆康托展开
///
/// 计算给定排名在同长度的全排列中的排列
///
/// # Preconditions
///
/// `rank` 必须是一个 1 到 n! 的整数。该函数并没有校验 `rank` 是否合法，不合法的话会出现未定义行为。
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::misc::inverse_cantor_expansion;
///
/// let rank = 3;
/// let n = 3;
/// let permutation = inverse_cantor_expansion(rank, n);
/// assert_eq!(permutation, [2, 1, 3]);
/// ```
pub fn inverse_cantor_expansion(rank: usize, n: usize) -> Vec<usize> {
    let mut rank = rank - 1;
    // 将排名转换为阶乘进制，即排列的 lahmer 码
    let mut lahmer = vec![0; n];
    for i in 1..=n {
        lahmer[n - i] = rank % i;
        rank /= i;
    }
    let mut bit = BinaryIndexedTree::new(n);
    for i in 1..=n {
        bit.add(i, 1_isize);
    }
    let mut permutation = vec![0; n];
    for i in 0..n {
        permutation[i] = bit
            .lower_bound(lahmer[i] as isize + 1)
            .expect("unexpected panic: lower_bound returned None");
        bit.add(permutation[i], -1);
    }
    permutation
}
