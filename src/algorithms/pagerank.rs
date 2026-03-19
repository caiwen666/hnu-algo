use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
};

use ndarray::{Array2, Axis};

use crate::algorithms::matrix::{CSCMatrix, Vector};

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

pub struct SimplePagerank<T> {
    capacity: usize,
    key_to_index: HashMap<T, usize>,
    index_to_key: HashMap<usize, T>,
    edges: Array2<f64>,
    size: usize,
}

impl<T> SimplePagerank<T>
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

pub struct SparsePagerank<T> {
    capacity: usize,
    key_to_index: HashMap<T, usize>,
    index_to_key: HashMap<usize, T>,
    // 按行存储，方便我们后续进行行归一化
    edges: Vec<HashMap<usize, f64>>,
    size: usize,
}

impl<T> SparsePagerank<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            key_to_index: HashMap::with_capacity(capacity),
            index_to_key: HashMap::with_capacity(capacity),
            edges: vec![HashMap::new(); capacity],
            size: 0,
        }
    }
    fn get_key_index(&mut self, key: T) -> usize {
        if self.key_to_index.contains_key(&key) {
            self.key_to_index[&key]
        } else {
            if self.size >= self.capacity {
                panic!("Capacity exceeded");
            }
            let new_index = self.size;
            self.size += 1;
            self.key_to_index.insert(key.clone(), new_index);
            self.index_to_key.insert(new_index, key);
            new_index
        }
    }
    /// 添加一条从 from_key 到 to_key 的边
    ///
    /// 如果边已经被添加过，则不会重复添加
    ///
    /// # Panics
    ///
    /// 如果添加的边涉及到的节点数量超过了 `capacity`，则 panic
    pub fn add_edge(&mut self, from_key: T, to_key: T) {
        let from_index = self.get_key_index(from_key.clone());
        let to_index = self.get_key_index(to_key.clone());
        self.edges[from_index].entry(to_index).or_insert(1.0);
    }

    pub fn rank(&self, following_prob: f64, tolerance: f64) -> Vec<(T, f64)> {
        // 按行归一化，得到转移矩阵
        let mut transition_matrix = CSCMatrix::new(self.capacity, self.capacity);
        // 寻找 dangling nodes
        let mut dangling_nodes_index = HashSet::new();
        for (row_num, row) in self.edges.iter().enumerate() {
            let row_sum = row.values().sum::<f64>();
            if row_sum == 0.0 {
                dangling_nodes_index.insert(row_num);
                continue;
            }
            for (col_num, value) in row.iter() {
                transition_matrix.set(row_num, *col_num, *value / row_sum);
            }
        }
        // 初始化 pagerank
        let mut last = vec![1.0 / self.capacity as f64; self.capacity];
        let personalization = last.clone();
        let dangling_weights = personalization.clone();
        // 不断迭代直到达到收敛条件
        loop {
            let leak_rank = last
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| {
                    if dangling_nodes_index.contains(&i) {
                        Some(x)
                    } else {
                        None
                    }
                })
                .sum();
            let current = transition_matrix
                .left_mul(last.clone())
                .add(dangling_weights.clone().scale(leak_rank))
                .scale(following_prob)
                .add(personalization.clone().scale(1.0 - following_prob));
            let diff = last
                .clone()
                .sub(current.clone())
                .iter()
                .map(|x| x.abs())
                .sum::<f64>();
            if diff < self.capacity as f64 * tolerance {
                // 收敛时应返回本轮的 current
                last = current;
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

impl<T> SparsePagerank<T>
where
    T: Eq + std::hash::Hash + Clone + serde::Serialize,
{
    /// 将图导出到指定文件
    ///
    /// # Panics
    ///
    /// 如果发生文件 IO 相关错误，则会 panic
    pub fn export_graph(&self, file_path: &str) {
        #[derive(serde::Serialize)]
        struct Edge<T> {
            from: T,
            to: T,
        }
        let mut edges = Vec::new();
        for (row_index, row) in self.edges.iter().enumerate() {
            for (col_index, _) in row.iter() {
                edges.push(Edge {
                    from: self.index_to_key[&row_index].clone(),
                    to: self.index_to_key[col_index].clone(),
                });
            }
        }
        let mut file = File::create(file_path).unwrap();
        serde_json::to_writer_pretty(&mut file, &edges).unwrap();
    }
}
