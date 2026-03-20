//! 图相关数据集

/// 加载 Google 数据集
/// 数据集地址：<https://snap.stanford.edu/data/web-Google.html>
pub fn load_google_dataset() -> Vec<(usize, usize)> {
    std::fs::read_to_string("dataset/web-Google.txt")
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let [from, to] = line.split('\t').collect::<Vec<&str>>().try_into().unwrap();
            let from = from.parse::<usize>().unwrap();
            let to = to.parse::<usize>().unwrap();
            (from, to)
        })
        .collect::<Vec<_>>()
}

/// 加载三体人物关系数据集
/// 数据集地址：<https://pic.caiwen.work/dataset/three_body_edges.csv>
pub fn load_three_body_dataset() -> Vec<(String, String)> {
    std::fs::read_to_string("dataset/three_body_edges.csv")
        .unwrap()
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let [source, target, _] = line.split(',').collect::<Vec<&str>>().try_into().unwrap();
            (source.to_string(), target.to_string())
        })
        .collect::<Vec<_>>()
}

/// 加载 Twitter 数据集
/// 数据集地址：<https://snap.stanford.edu/data/ego-Twitter.html>
pub fn load_twitter_dataset() -> Vec<(usize, usize)> {
    std::fs::read_to_string("dataset/twitter_combined.txt")
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let [from, to] = line.split(' ').collect::<Vec<&str>>().try_into().unwrap();
            let from = from.parse::<usize>().unwrap();
            let to = to.parse::<usize>().unwrap();
            (from, to)
        })
        .collect::<Vec<_>>()
}
