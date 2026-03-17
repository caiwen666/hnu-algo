use hnu_algo::{algorithms::dp::simple_knapsack, dataset::misc};

#[test]
#[ignore]
fn test_simple_knapsack_bzoj1625_all_cases() {
    for index in 1..=10 {
        println!("testing simple_knapsack, dataset: bzoj1625 case {}", index);

        let (capacity, items, expected_max_value) = misc::load_bzoj1625(index);

        let result = simple_knapsack(&items, capacity, false);

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
    }
}
