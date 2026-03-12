//! 三体人物关系的 PageRank 计算
//! 测试数据集在 <https://pic.caiwen.work/three_body_edges.csv>
use std::time::Instant;

use hnu_algo::algorithms::pagerank::{SimplePagerank, SparsePagerank};

fn main() {
    let data = std::fs::read_to_string("dataset/three_body_edges.csv")
        .unwrap()
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let [source, target, _] = line.split(',').collect::<Vec<&str>>().try_into().unwrap();
            (source.to_string(), target.to_string())
        })
        .collect::<Vec<_>>();
    let node_count = hnu_algo::utils::count_nodes(&data);

    let clock = Instant::now();
    let mut graph = SparsePagerank::new(node_count);
    for (source, target) in &data {
        graph.add_edge(source, target);
    }
    let res = graph.rank(0.85, 1e-6);
    println!(
        "----------------SparsePagerank----------------\nTime: {:?}\nOutput:\n{:?}",
        clock.elapsed(),
        res
    );

    let clock = Instant::now();
    let mut graph = SimplePagerank::new(node_count);
    for (source, target) in &data {
        graph.add_edge(source, target).unwrap();
    }
    let res = graph.rank(0.85, 1e-6);
    println!(
        "----------------SimplePagerank----------------\nTime: {:?}\nOutput:\n{:?}",
        clock.elapsed(),
        res
    );
}
