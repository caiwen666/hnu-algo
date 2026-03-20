/// 加载 normal_small.txt 数据集
/// 序列长度：100，在 [0, 1_000_000] 范围内随机生成
/// 数据集地址：<https://pic.caiwen.work/dataset/seq_normal.zip>
pub fn load_normal_small() -> Vec<usize> {
    std::fs::read_to_string("dataset/seq/normal_small.txt")
        .unwrap()
        .lines()
        .map(|line| line.parse::<usize>().unwrap())
        .collect()
}

/// 加载 normal_medium.txt 数据集
/// 序列长度：10000，在 [0, 1_000_000] 范围内随机生成
/// 数据集地址：<https://pic.caiwen.work/dataset/seq_normal.zip>
pub fn load_normal_medium() -> Vec<usize> {
    std::fs::read_to_string("dataset/seq/normal_medium.txt")
        .unwrap()
        .lines()
        .map(|line| line.parse::<usize>().unwrap())
        .collect()
}

/// 加载 normal_large.txt 数据集
/// 序列长度：10000000，在 [0, 1_000_000] 范围内随机生成
/// 数据集地址：<https://pic.caiwen.work/dataset/seq_normal.zip>
pub fn load_normal_large() -> Vec<usize> {
    std::fs::read_to_string("dataset/seq/normal_large.txt")
        .unwrap()
        .lines()
        .map(|line| line.parse::<usize>().unwrap())
        .collect()
}
