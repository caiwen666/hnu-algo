/// 加载随机生成的最短路数据集
///
/// 均为有向图，边权均在 1e9 以内，均没有重边和自环
///
/// |编号|说明|
/// |-|-|
/// |1|随机生成，点数为 10，边数为 30|
/// |2|随机生成，点数为 1e3，边数为 2e4|
/// |3|随机生成，点数为 5e5，边数为 1e6|
/// |4|hack spfa（假的），点数为 5e5，边数为 1.5 * 5e5|
///
/// 数据下载链接：TODO
///
/// # Arguments
///
/// - `idx`: 加载第几组数据，范围为 [1, 4]
///
/// # Returns
///
/// 返回一个元组，第一个元素为起点，第二个元素为边列表（邻接表），第三个元素为到各个点的距离
///
/// 边列表和距离列表中，均为 1-indexed
///
/// 距离列表中，u64::MAX 表示不可到达
#[expect(clippy::type_complexity)]
pub fn load_normal(idx: usize) -> (usize, Vec<Vec<(usize, usize)>>, Vec<u64>) {
    let input = std::fs::read_to_string(format!("dataset/ssp/ssp{}.in", idx)).unwrap();
    let mut input = input
        .lines()
        .flat_map(|line| line.split_whitespace().map(|s| s.parse::<usize>().unwrap()));
    let n = input.next().unwrap();
    let m = input.next().unwrap();
    let s = input.next().unwrap();
    let mut edges: Vec<Vec<(usize, usize)>> = vec![vec![]; n + 1];
    for _ in 0..m {
        let u = input.next().unwrap();
        let v = input.next().unwrap();
        let w = input.next().unwrap();
        edges[u].push((v, w));
    }
    let output = std::fs::read_to_string(format!("dataset/ssp/ssp{}.out", idx)).unwrap();
    let mut output = output.lines().flat_map(|line| {
        line.split_whitespace().map(|s| {
            if s == "-1" {
                u64::MAX
            } else {
                s.parse::<u64>().unwrap()
            }
        })
    });
    let mut distances = vec![0; n + 1];

    for distance in distances.iter_mut().skip(1) {
        *distance = output.next().unwrap() as u64;
    }

    (s, edges, distances)
}
