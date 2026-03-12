use std::time::Instant;

use hnu_algo::algorithms::pagerank::SparsePagerank;

fn main() {
    let data = std::fs::read_to_string("dataset/web-Google.txt")
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let [from, to] = line.split('\t').collect::<Vec<&str>>().try_into().unwrap();
            let from = from.parse::<usize>().unwrap();
            let to = to.parse::<usize>().unwrap();
            (from, to)
        })
        .collect::<Vec<_>>();

    let capacity = hnu_algo::utils::count_nodes(&data);
    let mut graph = SparsePagerank::new(capacity);
    for (from, to) in data {
        graph.add_edge(from, to);
    }

    let clock = Instant::now();
    let res = graph
        .rank(0.85, 1e-6)
        .into_iter()
        .take(5)
        .collect::<Vec<_>>();
    let elapsed = clock.elapsed();
    println!("Time: {:?}\nOutput:\n{:#?}", elapsed, res);
}
