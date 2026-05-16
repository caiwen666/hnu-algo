pub fn parse_interval_cover(s: &str) -> (Vec<f64>, f64, usize) {
    let lines: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert!(lines.len() >= 2, "interval cover needs at least 2 lines");
    let mut h = lines[0].split_whitespace();
    let n: usize = h.next().unwrap().parse().expect("n");
    let length: f64 = h.next().unwrap().parse().expect("k");
    let points: Vec<f64> = lines[1]
        .split_whitespace()
        .map(|x| x.parse().expect("point"))
        .collect();
    assert_eq!(points.len(), n, "point count must match n");
    let exp = if lines.len() >= 3 {
        lines[2].parse().expect("expected count")
    } else {
        crate::algorithms::greedy::interval_cover_count(&points, length)
    };
    (points, length, exp)
}

/// 加载区间覆盖数据.
///
/// |编号|说明|
/// |-|-|
/// |1|小：\(n=200\)，区间长度 \(k=5.0\)，点坐标为 \([0,10^6]\) 上整数均匀随机|
/// |2|中：\(n=5000\)，\(k=12.5\)，同上|
/// |3|大：\(n=8\times 10^4\)，\(k=12.5\)，同上|
///
/// # Arguments
///
/// - `idx`：1/2/3 为小/中/大。
///
/// # Returns
///
/// `(points, length, expected_count)`，其中 `points` 为点坐标，`length` 为区间长度，`expected_count` 为实际最少区间个数。
pub fn load_interval_cover(idx: usize) -> (Vec<f64>, f64, usize) {
    assert!((1..=3).contains(&idx), "idx must be 1..=3");
    let path_in = format!("dataset/lab2/interval_{idx}.in");
    let path_out = format!("dataset/lab2/interval_{idx}.out");
    let inp = std::fs::read_to_string(&path_in).unwrap_or_else(|e| {
        panic!("read {path_in}: {e}; run `python scripts/gen_lab2_task3_interval.py`",)
    });
    let exp: usize = std::fs::read_to_string(&path_out)
        .unwrap_or_else(|e| {
            panic!("read {path_out}: {e}; run `python scripts/gen_lab2_task3_interval.py`",)
        })
        .trim()
        .parse()
        .expect("interval answer");
    let (points, length, exp2) = parse_interval_cover(&inp);
    debug_assert_eq!(exp2, exp);
    (points, length, exp)
}
