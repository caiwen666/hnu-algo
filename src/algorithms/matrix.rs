use std::{
    collections::HashMap,
    ops::{Add, Mul},
};

/// 稀疏矩阵，把列压缩掉
pub struct CSCMatrix<T> {
    col_elements: Vec<HashMap<usize, T>>,
    row: usize,
    col: usize,
}

impl<T> CSCMatrix<T>
where
    T: Clone + Copy + Default + Mul<Output = T> + Add<Output = T>,
{
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            col_elements: vec![HashMap::new(); col],
            row,
            col,
        }
    }

    /// 设置矩阵的元素
    ///
    /// # Arguments
    ///
    /// - `row`: 行索引，从 0 开始
    /// - `col`: 列索引，从 0 开始
    /// - `element`: 要设置的元素
    ///
    /// # Returns
    ///
    /// 如果元素被替换，则返回旧元素，否则返回 None
    ///
    /// # Panics
    ///
    /// 如果行或列超出范围，则 panic
    pub fn set(&mut self, row: usize, col: usize, element: T) -> Option<T> {
        if row >= self.row || col >= self.col {
            panic!("Index out of bounds");
        }
        self.col_elements[col].insert(row, element)
    }

    /// 左乘行向量
    ///
    /// # Arguments
    ///
    /// - `vector`: 要乘的行向量，向量的长度必须与矩阵的行数相同
    ///
    /// # Returns
    ///
    /// 乘积结果，将会得到一个行向量，该行向量的长度等于矩阵的列数
    ///
    /// # Panics
    ///
    /// 如果 `vector` 长度不等于矩阵行数，则 panic
    pub fn left_mul(&self, vector: Vec<T>) -> Vec<T> {
        if vector.len() != self.row {
            panic!("Vector length must be equal to matrix row");
        }
        let mut result = vec![T::default(); self.col];
        for (col, col_list) in self.col_elements.iter().enumerate() {
            for (row, value) in col_list.iter() {
                result[col] = result[col] + vector[*row] * *value;
            }
        }
        result
    }
}

pub trait Vector {
    type Item;
    fn scale(self, s: Self::Item) -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}

impl Vector for Vec<f64> {
    type Item = f64;

    fn scale(self, s: Self::Item) -> Self {
        self.into_iter().map(|x| x * s).collect()
    }

    /// 将两个向量逐元素相加
    ///
    /// # Panics
    ///
    /// 如果两个向量的长度不相同，则 panic
    fn add(self, other: Self) -> Self {
        if self.len() != other.len() {
            panic!("Vector length must be equal");
        }
        self.into_iter()
            .zip(other)
            .map(|(x, y)| x + y)
            .collect()
    }

    /// 将两个向量逐元素相减
    ///
    /// # Arguments
    ///
    /// - `other`: 要减的向量
    ///
    /// # Returns
    ///
    /// 当前向量**和** `other`的差值
    ///
    /// # Panics
    ///
    /// 如果两个向量的长度不相同，则 panic
    fn sub(self, other: Self) -> Self {
        if self.len() != other.len() {
            panic!("Vector length must be equal");
        }
        self.into_iter()
            .zip(other)
            .map(|(x, y)| x - y)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_csc_matrix() {
        let mut matrix = CSCMatrix::new(3, 2);
        matrix.set(0, 0, 1);
        matrix.set(0, 1, 2);
        matrix.set(1, 0, 4);
        matrix.set(1, 1, 5);
        matrix.set(2, 0, 7);
        matrix.set(2, 1, 8);
        let vector = vec![1, 2, 3];
        let result = matrix.left_mul(vector);
        assert_eq!(result, vec![30, 36]);
    }
}
