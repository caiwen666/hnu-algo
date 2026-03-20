use crate::algorithms::dp::SimpleKnapsackItem;

/// 加载 bzoj1625 的题目数据集
///
/// 数据集数据范围：
/// * 背包容量: [1, 12880]
/// * 物品数量: [1, 3402]
/// * 物品代价: [1, 400]
/// * 物品收益: [1, 100]
///
/// 共 10 组数据。
///
/// 数据下载链接：<https://pic.caiwen.work/dataset/bzoj1625.zip>
///
/// # Arguments
///
/// - `index`: 加载第几组数据，范围为 [1, 10]
///
/// # Returns
///
/// 返回一个元组，第一个元素为背包容量，第二个元素为物品列表，第三个元素为最大收益
pub fn load_bzoj1625(index: usize) -> (usize, Vec<SimpleKnapsackItem>, usize) {
    let input = std::fs::read_to_string(format!("dataset/misc/bzoj1625/{}.in", index)).unwrap();
    let mut input = input.lines().flat_map(|line| {
        line.split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect::<Vec<usize>>()
    });
    let n = input.next().unwrap();
    let capacity = input.next().unwrap();
    let mut items = Vec::new();
    for _ in 0..n {
        let weight = input.next().unwrap();
        let value = input.next().unwrap();
        items.push(SimpleKnapsackItem { weight, value });
    }
    let output: usize = std::fs::read_to_string(format!("dataset/misc/bzoj1625/{}.out", index))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    (capacity, items, output)
}
