use hnu_algo::{algorithms::greedy::interval_cover_count, dataset::greedy::load_interval_cover};

#[test]
#[ignore]
fn test_interval_cover() {
    for idx in 1..=3 {
        let (points, length, expected) = load_interval_cover(idx);
        let timer = std::time::Instant::now();
        assert_eq!(
            interval_cover_count(&points, length),
            expected,
            "interval_cover_count on interval cover case {} has incorrect count",
            idx
        );
        println!(
            "interval_cover_count on interval cover case {} took {:?}",
            idx,
            timer.elapsed()
        );
    }
}
