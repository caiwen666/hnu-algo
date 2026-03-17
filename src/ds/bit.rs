use std::ops::{AddAssign, Sub};

use crate::utils::low_bit;

/// 树状数组
///
/// 树状数组内的下标从 1 开始
pub struct BinaryIndexedTree<T> {
    capacity: usize,
    data: Vec<T>,
}

impl<T> BinaryIndexedTree<T>
where
    T: Default + AddAssign<T> + Clone,
{
    /// 创建一个树状数组，容量为 capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            data: vec![T::default(); capacity + 1],
        }
    }
    /// 在 index 位置上加上 value
    ///
    /// 时间复杂度为 O(log n)，n 为树状数组的 capacity
    ///
    /// 树状数组的下标从 1 开始
    ///
    /// # Panics
    ///
    /// 如果 index 为 0 或大于 capacity，则 panic
    pub fn add(&mut self, index: usize, value: T) {
        if index == 0 || index > self.capacity {
            panic!("index out of range");
        }
        let mut i = index;
        while i <= self.capacity {
            self.data[i] += value.clone();
            i += low_bit(i);
        }
    }

    /// 计算前 index 个元素的和
    ///
    /// 时间复杂度为 O(log n)，n 为树状数组的 capacity
    ///
    /// 树状数组的下标从 1 开始
    ///
    /// 如果 index 为 0，则直接返回 T::default()
    ///
    /// # Panics
    ///
    /// 如果 index 大于 capacity，则 panic
    pub fn prefix_sum(&self, index: usize) -> T {
        if index == 0 {
            return T::default();
        }
        if index > self.capacity {
            panic!("index out of range");
        }
        let mut i = index;
        let mut sum = T::default();
        while i > 0 {
            sum += self.data[i].clone();
            i -= low_bit(i);
        }
        sum
    }
}

impl<T> BinaryIndexedTree<T>
where
    T: Default + AddAssign<T> + Sub<Output = T> + Clone,
{
    /// 计算区间 [left, right] 的和
    ///
    /// 时间复杂度为 O(log n)，n 为树状数组的 capacity
    ///
    /// 树状数组的下标从 1 开始
    ///
    /// # Panics
    ///
    /// 如果 left 或 right 为 0 或大于 capacity，则 panic
    pub fn range_sum(&self, left: usize, right: usize) -> T {
        if left == 0 || right == 0 || left > right || left > self.capacity || right > self.capacity
        {
            panic!("index out of range");
        }
        self.prefix_sum(right) - self.prefix_sum(left - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit1() {
        let mut bit = BinaryIndexedTree::new(5);
        // 初始值
        bit.add(1, 1);
        bit.add(2, 5);
        bit.add(3, 4);
        bit.add(4, 2);
        bit.add(5, 3);
        // 操作
        bit.add(1, 3);
        assert_eq!(bit.range_sum(2, 5), 14);
        bit.add(3, -1);
        bit.add(4, 2);
        assert_eq!(bit.range_sum(1, 4), 16);
    }
}
