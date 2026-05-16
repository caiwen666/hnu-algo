use crate::algorithms::dfs::vertex_cover::WeightedVertexCover;

fn parse_separator_free_dictionary(s: &str) -> (usize, usize, usize) {
    let lines: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert!(!lines.is_empty(), "empty sfd input");
    let mut h = lines[0].split_whitespace();
    let n: usize = h.next().unwrap().parse().expect("n");
    let k: usize = h.next().unwrap().parse().expect("k");
    let exp = if lines.len() >= 2 {
        lines[1].parse().expect("expected max size")
    } else {
        crate::algorithms::dfs::sfd::max_sfd_size(n, k)
    };
    (n, k, exp)
}

/// 加载无分隔符字典数据。
///
/// |编号|说明|
/// |-|-|
/// |1|小：字母表大小 \(n=2\)，串长 \(k=5\)（\(n^k=32\)）|
/// |2|中：\(n=3\)，\(k=3\)（\(n^k=27\)）|
/// |3|大：\(n=2\)，\(k=6\)（\(n^k=64\)）|
///
/// # Arguments
///
/// - `idx`：1/2/3 为小/中/大。
///
/// # Returns
///
/// `(alphabet_size, str_len, expected_max_size)`，与 [`crate::algorithms::dfs::sfd::max_sfd_size`] 配套。
pub fn load_separator_free_dictionary(idx: usize) -> (usize, usize, usize) {
    assert!((1..=3).contains(&idx), "idx must be 1..=3");
    let path_in = format!("dataset/lab2/sfd_{idx}.in");
    let path_out = format!("dataset/lab2/sfd_{idx}.out");
    let inp = std::fs::read_to_string(&path_in).unwrap_or_else(|e| {
        panic!("read {path_in}: {e}; run `python scripts/gen_lab2_task5_sfd.py`",)
    });
    let exp: usize = std::fs::read_to_string(&path_out)
        .unwrap_or_else(|e| {
            panic!("read {path_out}: {e}; run `python scripts/gen_lab2_task5_sfd.py`",)
        })
        .trim()
        .parse()
        .expect("sfd answer");
    let (n, k, exp2) = parse_separator_free_dictionary(&inp);
    debug_assert_eq!(exp2, exp);
    (n, k, exp)
}

fn parse_vertex_cover(s: &str) -> (WeightedVertexCover, u128) {
    let lines: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 3,
        "vertex cover needs header, edges, weights"
    );
    let mut h = lines[0].split_whitespace();
    let n: usize = h.next().unwrap().parse().expect("n");
    let m: usize = h.next().unwrap().parse().expect("m");
    assert!(lines.len() >= 2 + m, "not enough edge lines");
    let mut edges = Vec::with_capacity(m);
    for line in &lines[1..=m] {
        let mut e = line.split_whitespace();
        let u1: usize = e.next().unwrap().parse().expect("u");
        let v1: usize = e.next().unwrap().parse().expect("v");
        assert!((1..=n).contains(&u1) && (1..=n).contains(&v1));
        edges.push((u1 - 1, v1 - 1));
    }
    let weight: Vec<u64> = lines[1 + m]
        .split_whitespace()
        .map(|x| x.parse().expect("weight"))
        .collect();
    assert_eq!(weight.len(), n, "weight count");
    let g = WeightedVertexCover::new(n, edges, weight.clone());
    let exp = if lines.len() >= 3 + m {
        lines[2 + m].parse::<u128>().expect("expected total weight")
    } else {
        // 无标答行时仅构造实例，期望由外部提供
        g.min_weight_vertex_cover_branch_bound().total_weight
    };
    (g, exp)
}

/// 加载最小权顶点覆盖数据。
///
/// |编号|说明|
/// |-|-|
/// |1|小：\(n=8\)，\(m=14\)，边随机无重边无自环，点权 \([1,10]\)|
/// |2|中：\(n=14\)，\(m=28\)，同上|
/// |3|大：\(n=20\)，\(m=48\)，同上|
///
/// # Arguments
///
/// - `idx`：1/2/3 为小/中/大。
///
/// # Returns
///
/// [`WeightedVertexCover`] 与实际最小权和。
pub fn load_vertex_cover(idx: usize) -> (WeightedVertexCover, u128) {
    assert!((1..=3).contains(&idx), "idx must be 1..=3");
    let path_in = format!("dataset/lab2/vcover_{idx}.in");
    let path_out = format!("dataset/lab2/vcover_{idx}.out");
    let inp = std::fs::read_to_string(&path_in).unwrap_or_else(|e| {
        panic!("read {path_in}: {e}; run `python scripts/gen_lab2_task6_vertex_cover.py`",)
    });
    let exp: u128 = std::fs::read_to_string(&path_out)
        .unwrap_or_else(|e| {
            panic!("read {path_out}: {e}; run `python scripts/gen_lab2_task6_vertex_cover.py`",)
        })
        .trim()
        .parse()
        .expect("vcover answer");
    let (g, exp2) = parse_vertex_cover(&inp);
    debug_assert_eq!(exp2, exp);
    (g, exp)
}
