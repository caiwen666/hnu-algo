use hnu_algo::{
    algorithms::dp::simple_knapsack,
    dataset::{dp::load_batch_scheduling, misc},
};

#[test]
#[ignore]
fn test_simple_knapsack_bzoj1625_all_cases() {
    for index in 1..=3 {
        let (capacity, items, expected_max_value) = misc::load_bzoj1625(index);

        let timer = std::time::Instant::now();
        let result = simple_knapsack(&items, capacity, false);
        let elapsed = timer.elapsed();

        let actual_value: usize = result.iter().map(|item| item.value).sum();

        assert_eq!(
            actual_value, expected_max_value,
            "simple_knapsack on bzoj1625 case {} has incorrect maximum value",
            index
        );

        let actual_weight: usize = result.iter().map(|item| item.weight).sum();
        assert!(
            actual_weight <= capacity,
            "simple_knapsack on bzoj1625 case {} has incorrect total weight",
            index
        );

        println!(
            "simple_knapsack on bzoj1625 case {} took {:?}",
            index, elapsed
        );
    }
}

#[test]
#[ignore]
fn test_batch_scheduling() {
    for idx in 1..=3 {
        let (p, expected) = load_batch_scheduling(idx);
        let timer = std::time::Instant::now();
        assert_eq!(
            p.solve(),
            expected,
            "BatchScheduling::solve on batch scheduling case {} has incorrect cost",
            idx
        );
        println!(
            "BatchScheduling::solve on batch scheduling case {} took {:?}",
            idx,
            timer.elapsed()
        );
    }
}
