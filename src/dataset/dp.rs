use crate::algorithms::dp::BatchScheduling;

pub fn parse_batch_scheduling(s: &str) -> (BatchScheduling, usize) {
    let lines: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 4,
        "batch scheduling input needs at least 4 lines"
    );
    let startup: usize = lines[0].parse().expect("startup S");
    let n: usize = lines[1].parse().expect("n");
    let duration: Vec<usize> = lines[2]
        .split_whitespace()
        .map(|x| x.parse().expect("t_i"))
        .collect();
    let fee: Vec<usize> = lines[3]
        .split_whitespace()
        .map(|x| x.parse().expect("f_i"))
        .collect();
    assert_eq!(duration.len(), n, "duration count");
    assert_eq!(fee.len(), n, "fee count");
    let exp = if lines.len() >= 5 {
        lines[4].parse().expect("expected cost")
    } else {
        BatchScheduling::new(startup, duration.clone(), fee.clone()).solve()
    };
    (BatchScheduling::new(startup, duration, fee), exp)
}

/// 加载任务批处理问题的数据。
///
/// |编号|说明|
/// |-|-|
/// |1|小：\(n=50\)，\(S=4\)，\(t_i\in[1,50]\)，\(f_i\in[1,20]\)|
/// |2|中：\(n=800\)，\(S=9\)，取值范围同上|
/// |3|大：\(n=6000\)，\(S=11\)，取值范围同上|
///
/// # Arguments
///
/// - `idx`：取 1、2、3 分别对应小/中/大。
///
/// # Returns
///
/// [BatchScheduling] 与实际最小总费用。
pub fn load_batch_scheduling(idx: usize) -> (BatchScheduling, usize) {
    assert!((1..=3).contains(&idx), "idx must be 1..=3");
    let path_in = format!("dataset/lab2/batch_{idx}.in");
    let path_out = format!("dataset/lab2/batch_{idx}.out");
    let inp = std::fs::read_to_string(&path_in).unwrap_or_else(|e| {
        panic!("read {path_in}: {e}; run `python scripts/gen_lab2_task2_batch.py` from repo root",)
    });
    let exp: usize = std::fs::read_to_string(&path_out)
        .unwrap_or_else(|e| {
            panic!("read {path_out}: {e}; run `python scripts/gen_lab2_task2_batch.py`",)
        })
        .trim()
        .parse()
        .expect("batch answer");
    let (p, exp_computed) = parse_batch_scheduling(&inp);
    debug_assert_eq!(exp_computed, exp);
    (p, exp)
}
