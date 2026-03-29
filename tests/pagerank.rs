use std::collections::HashMap;
use std::fmt::Debug;

use hnu_algo::{
    algorithms::pagerank::{SimplePagerank, SparsePagerank},
    dataset,
};

const EPSILON: f64 = 1e-10;

/// 加载 Networkx 在三体人物关系数据集上进行 PageRank 得到的结果
/// 数据地址：<https://pic.caiwen.work/dataset/pagerank_networkx_output.zip>
/// 计算参数：alpha=0.85, max_iter=100, tol=1e-6
fn load_three_body_networkx_output() -> HashMap<String, f64> {
    let data = std::fs::read_to_string("dataset/three_body_networkx.json").unwrap();
    serde_json::from_str::<HashMap<String, f64>>(&data).unwrap()
}

/// 判断 SimplePagerank 计算结果是否正确
fn judge_simple<T>(data: Vec<(T, T)>, networkx_output: HashMap<T, f64>)
where
    T: Eq + std::hash::Hash + Clone + Debug,
{
    let capacity = hnu_algo::utils::count_nodes(&data);
    let mut graph = SimplePagerank::new(capacity);
    for (from, to) in data {
        graph.add_edge(from, to).unwrap();
    }
    let res = graph.rank(0.85, 1e-6);
    println!(
        "Top 10 nodes: {:?}",
        res.iter().take(10).collect::<Vec<_>>()
    );
    for (node, rank) in res {
        let networkx_rank = networkx_output[&node];
        assert!((rank - networkx_rank).abs() < EPSILON);
    }
}

/// 判断 SparsePagerank 计算结果是否正确
fn judge_sparse<T>(data: Vec<(T, T)>, networkx_output: HashMap<T, f64>)
where
    T: Eq + std::hash::Hash + Clone + Debug,
{
    let capacity = hnu_algo::utils::count_nodes(&data);
    let mut graph = SparsePagerank::new(capacity);
    for (from, to) in data {
        graph.add_edge(from, to);
    }
    let res = graph.rank(0.85, 1e-6);
    println!(
        "Top 10 nodes: {:?}",
        res.iter().take(10).collect::<Vec<_>>()
    );
    for (node, rank) in res {
        let networkx_rank = networkx_output[&node];
        assert!((rank - networkx_rank).abs() < EPSILON);
    }
}

#[test]
#[ignore]
fn test_three_body_simple() {
    // 使用 SimplePagerank
    let data = dataset::graph::load_three_body_dataset();
    let networkx_output = load_three_body_networkx_output();
    judge_simple(data, networkx_output);
}

#[test]
#[ignore]
fn test_three_body_sparse() {
    // 使用 SparsePagerank
    let data = dataset::graph::load_three_body_dataset();
    let networkx_output = load_three_body_networkx_output();
    judge_sparse(data, networkx_output);
}

/// 加载 Networkx 在 Google 数据集上进行 PageRank 得到的结果
/// 数据地址：<https://pic.caiwen.work/dataset/pagerank_networkx_output.zip>
/// 计算参数：alpha=0.85, max_iter=100, tol=1e-6
fn load_google_networkx_output() -> HashMap<usize, f64> {
    let data = std::fs::read_to_string("dataset/web-Google_networkx.json").unwrap();
    serde_json::from_str::<HashMap<usize, f64>>(&data).unwrap()
}

#[test]
#[ignore]
fn test_google() {
    let data = dataset::graph::load_google_dataset();
    let networkx_output = load_google_networkx_output();
    judge_sparse(data, networkx_output);
}

/// 加载 Networkx 在 twitter 数据集上进行 PageRank 得到的结果
/// 数据地址：<https://pic.caiwen.work/dataset/pagerank_networkx_output.zip>
/// 计算参数：alpha=0.85, max_iter=100, tol=1e-6
fn load_twitter_networkx_output() -> HashMap<usize, f64> {
    let data = std::fs::read_to_string("dataset/twitter_networkx.json").unwrap();
    serde_json::from_str::<HashMap<usize, f64>>(&data).unwrap()
}

#[test]
#[ignore]
fn test_twitter() {
    let data = dataset::graph::load_twitter_dataset();
    let networkx_output = load_twitter_networkx_output();
    judge_sparse(data, networkx_output);
}
