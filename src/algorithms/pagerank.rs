use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
};

use ndarray::{Array2, Axis};

#[derive(Debug)]
pub enum PagerankError {
    CapacityExceeded,
}

impl Display for PagerankError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded => write!(f, "Capacity exceeded"),
        }
    }
}

impl Error for PagerankError {}

pub struct SimplePagerankGraph<T> {
    capacity: usize,
    key_to_index: HashMap<T, usize>,
    index_to_key: HashMap<usize, T>,
    edges: Array2<f64>,
    size: usize,
}

impl<T> SimplePagerankGraph<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            key_to_index: HashMap::with_capacity(capacity),
            index_to_key: HashMap::with_capacity(capacity),
            edges: Array2::zeros((capacity, capacity)),
            size: 0,
        }
    }
    fn get_key_index(&mut self, key: T) -> Result<usize, PagerankError> {
        if self.key_to_index.contains_key(&key) {
            Ok(self.key_to_index[&key])
        } else {
            if self.size >= self.capacity {
                return Err(PagerankError::CapacityExceeded);
            }
            let new_index = self.size;
            self.size += 1;
            self.key_to_index.insert(key.clone(), new_index);
            self.index_to_key.insert(new_index, key);
            Ok(new_index)
        }
    }
    pub fn add_edge(&mut self, from_key: T, to_key: T) -> Result<(), PagerankError> {
        let from_index = self.get_key_index(from_key.clone())?;
        let to_index = self.get_key_index(to_key.clone())?;
        self.edges[(from_index, to_index)] = 1.0;
        Ok(())
    }
    pub fn rank(&self, following_prob: f64, tolerance: f64) -> Vec<(T, f64)> {
        let row_sum = self.edges.sum_axis(Axis(1));
        // 计算转移矩阵
        let transition_matrix = self.edges.clone()
            / row_sum
                .mapv(|s| if s == 0.0 { 1.0 } else { s })
                .insert_axis(Axis(1));
        // 寻找 dangling nodes
        let dangling_nodes_index = row_sum
            .indexed_iter()
            .filter_map(|(i, &s)| if s == 0.0 { Some(i) } else { None })
            .collect::<Vec<_>>();
        // 初始化 pagerank
        let mut last = Array2::<f64>::from_elem((1, self.capacity), 1.0 / self.capacity as f64);
        let personalization = last.clone();
        let dangling_weights = personalization.clone();
        // 不断迭代直到达到收敛条件
        loop {
            let t = last.select(Axis(1), dangling_nodes_index.as_slice());
            let leak_rank = t.sum();
            let current = following_prob
                * (last.dot(&transition_matrix) + leak_rank * &dangling_weights)
                + (1.0 - following_prob) * &personalization;
            if (&last - &current).abs().sum() < self.capacity as f64 * tolerance {
                break;
            }
            last = current;
        }
        let mut result: Vec<(T, f64)> = last
            .into_iter()
            .enumerate()
            .map(|(i, x)| (self.index_to_key[&i].clone(), x))
            .collect();
        result.sort_by(|a, b| b.1.total_cmp(&a.1));
        result
    }
}
