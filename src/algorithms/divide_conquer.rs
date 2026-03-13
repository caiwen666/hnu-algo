/// 分治法求最小值和最小值
///
/// # Arguments
///
/// * `arr` - 要查找的数组
///
/// # Returns
///
/// 返回一个 tuple，第一个元素是最小值，第二个元素是最大值
///
/// 如果有多个最小值或最大值，则返回第一个最小值或最大值
///
/// # Panics
///
/// 如果数组为空，则 panic
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::divide_conquer::find_min_max;
/// let arr = [1, 1, 4, 5, 1, 4];
/// let (min, max) = find_min_max(&arr);
/// assert_eq!(min, 0);
/// assert_eq!(max, 3);
/// ```
pub fn find_min_max<T>(arr: &[T]) -> (usize, usize)
where
    T: Ord,
{
    if arr.is_empty() {
        panic!("Array is empty");
    }
    if arr.len() == 1 {
        return (0, 0);
    }
    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);
    let (left_min, left_max) = find_min_max(left);
    let (right_min, right_max) = find_min_max(right);
    // find_min_max 返回的是相对于传入参数的索引。在分治递归时，需要把递归结果加个偏移来得到相对于当前数组的索引
    let right_min = right_min + mid;
    let right_max = right_max + mid;
    let min = if arr[left_min] <= arr[right_min] {
        left_min
    } else {
        right_min
    };
    let max = if arr[left_max] >= arr[right_max] {
        left_max
    } else {
        right_max
    };
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Array is empty")]
    fn test_find_min_max_empty() {
        let arr: [i32; 0] = [];
        let _ = find_min_max(&arr);
    }

    #[test]
    fn test_find_min_max_one() {
        let arr = [1];
        let (min, max) = find_min_max(&arr);
        assert_eq!(min, 0);
        assert_eq!(max, 0);
    }

    #[test]
    fn test_find_min_max_high() {
        // 最值在数组的右边
        let arr = [1, 1, 4, 5, 1, 4, -1, 10];
        let (min, max) = find_min_max(&arr);
        assert_eq!(min, 6);
        assert_eq!(max, 7);
    }
}
