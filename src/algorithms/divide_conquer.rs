/// 分治法求最小值和最小值
///
/// 时间复杂度 $O(n)$，空间复杂度 $O(\log n)$
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

/// 分治法排序，将数组从小到大排序
///
/// 时间复杂度 $O(n \log n)$，空间复杂度 $O(\log n)$，排序是稳定的
///
/// # Arguments
///
/// * `arr` - 要排序的数组
///
/// # Returns
///
/// 返回排序后的数组
///
/// # Examples
///
/// ```rust
/// # use hnu_algo::algorithms::divide_conquer::sort;
/// let arr = [1, 1, 4, 5, 1, 4];
/// let sorted_arr = sort(&arr);
/// let expected_arr = [1, 1, 1, 4, 4, 5];
/// assert_eq!(sorted_arr, expected_arr);
/// ```
pub fn sort<T>(arr: &[T]) -> Vec<&T>
where
    T: Ord,
{
    fn merge<'a, T>(left: &[&'a T], right: &[&'a T], target: &mut [&'a T])
    where
        T: Ord,
    {
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;

        while i < left.len() && j < right.len() {
            if left[i] <= right[j] {
                target[k] = left[i];
                i += 1;
            } else {
                target[k] = right[j];
                j += 1;
            }
            k += 1;
        }

        while i < left.len() {
            target[k] = left[i];
            i += 1;
            k += 1;
        }

        while j < right.len() {
            target[k] = right[j];
            j += 1;
            k += 1;
        }
    }

    if arr.is_empty() {
        return Vec::new();
    }

    // 初始把所有元素的引用收集到一个 Vec 中
    let mut result: Vec<&T> = arr.iter().collect();
    // 额外申请一次同样大小的缓冲区，整个排序过程中复用，避免递归中反复分配
    let mut buf: Vec<&T> = result.clone();

    let mut width = 1;
    let len = result.len();

    // 自底向上的迭代归并排序，避免递归开销
    while width < len {
        let mut i = 0;
        while i < len {
            let left = i;
            let mid = (i + width).min(len);
            let right = (i + 2 * width).min(len);
            if mid < right {
                merge(
                    &result[left..mid],
                    &result[mid..right],
                    &mut buf[left..right],
                );
            } else {
                // 这一段不足以形成两个区间，直接拷贝
                buf[left..right].copy_from_slice(&result[left..right]);
            }
            i += 2 * width;
        }
        std::mem::swap(&mut result, &mut buf);
        width *= 2;
    }
    result
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

    #[test]
    fn test_sort_empty() {
        let arr: [i32; 0] = [];
        let sorted_arr = sort(&arr);
        assert_eq!(sorted_arr, Vec::<&i32>::new());
    }

    #[test]
    fn test_sort_one() {
        let arr = [1];
        let sorted_arr = sort(&arr);
        assert_eq!(sorted_arr, vec![&1]);
    }

    #[test]
    fn test_sort_increase() {
        let arr = [1, 2, 3, 4, 5];
        let sorted_arr = sort(&arr);
        assert_eq!(sorted_arr, vec![&1, &2, &3, &4, &5]);
    }

    #[test]
    fn test_sort_decrease() {
        let arr = [5, 4, 3, 2, 1];
        let sorted_arr = sort(&arr);
        assert_eq!(sorted_arr, vec![&1, &2, &3, &4, &5]);
    }
}
