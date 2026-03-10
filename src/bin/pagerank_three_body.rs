//! 三体人物关系的 PageRank 计算
//! 测试数据集在 <https://pic.caiwen.work/three_body_edges.csv>
use hnu_algo::algorithms::pagerank::SimplePagerankGraph;

fn main() {
    let data = std::fs::read_to_string("dataset/three_body_edges.csv")
        .unwrap()
        .lines()
        .into_iter()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let [source, target, _] = line.split(',').collect::<Vec<&str>>().try_into().unwrap();
            (source.to_string(), target.to_string())
        })
        .collect::<Vec<_>>();
    let mut graph = SimplePagerankGraph::new(hnu_algo::utils::count_nodes(&data));
    for (source, target) in data {
        graph.add_edge(source, target).unwrap();
    }
    let res = graph.rank(0.85, 1e-6);
    println!("{:#?}", res);
}
