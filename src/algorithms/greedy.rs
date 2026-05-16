/// 给出若干个点与一个区间长度，求出最少需要多少个区间能覆盖所有点。
///
/// 时间复杂度 \(O(n\log n)\)，空间 \(O(n)\)
///
/// # Parameters
///
/// - `points`: 点坐标
/// - `length`: 区间长度
///
/// # Panics
///
/// - 如果存在某个点的坐标为 NaN，则 panic。
/// - 如果区间长度小于等于 0，则 panic。
///
/// # Examples
///
/// ```
/// # use hnu_algo::algorithms::greedy::interval_cover_count;
/// let points = vec![1.0, 2.0, 3.0];
/// let length = 1.5;
/// assert_eq!(interval_cover_count(&points, length), 2);
/// ```
pub fn interval_cover_count(points: &[f64], length: f64) -> usize {
    assert!(length > 0.0, "interval length must be positive");
    if points.is_empty() {
        return 0;
    }
    let mut xs: Vec<f64> = points.iter().copied().filter(|x| x.is_finite()).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut count = 0;
    let mut i = 0;
    while i < xs.len() {
        let left = xs[i];
        let right = left + length;
        count += 1;
        i += 1;
        while i < xs.len() && xs[i] <= right {
            i += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::interval_cover_count;

    #[test]
    fn empty() {
        let points = vec![];
        let length = 1.0;
        assert_eq!(interval_cover_count(&points, length), 0);
    }

    #[test]
    fn all_in_one() {
        let points = vec![0.0, 0.5, 1.0];
        let length = 10.0;
        assert_eq!(interval_cover_count(&points, length), 1);
    }

    #[test]
    fn needs_two() {
        let points = vec![0.0, 2.0];
        let length = 1.0;
        assert_eq!(interval_cover_count(&points, length), 2);
    }

    #[test]
    fn unsorted_input() {
        let points = vec![5.0, 1.0, 3.0];
        let length = 2.0;
        assert_eq!(interval_cover_count(&points, length), 2);
    }
}
