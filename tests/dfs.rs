use hnu_algo::{
    algorithms::dfs::{sfd::max_sfd_size, simple_knapsack::simple_knapsack_backtracking},
    dataset::{
        dfs::{load_separator_free_dictionary, load_vertex_cover},
        misc,
    },
};

#[test]
#[ignore]
fn test_sfd() {
    for idx in 1..=3 {
        let (n, k, expected) = load_separator_free_dictionary(idx);
        let timer = std::time::Instant::now();
        assert_eq!(
            max_sfd_size(n, k),
            expected,
            "max_sfd_size on sfd case {} has incorrect size",
            idx
        );
        println!(
            "max_sfd_size on sfd case {} took {:?}",
            idx,
            timer.elapsed()
        );
    }
}

#[test]
#[ignore]
fn test_weighted_vertex_cover() {
    for idx in 1..=3 {
        let (g, expected) = load_vertex_cover(idx);
        let timer = std::time::Instant::now();
        let sol = g.min_weight_vertex_cover_branch_bound();
        assert_eq!(
            sol.total_weight, expected,
            "min_weight_vertex_cover_branch_bound on weighted vertex cover case {} has incorrect total weight",
            idx
        );
        println!(
            "min_weight_vertex_cover_branch_bound on weighted vertex cover case {} took {:?}",
            idx,
            timer.elapsed()
        );
    }
}

#[test]
#[ignore]
fn test_simple_knapsack_backtracking() {
    for index in 1..=10 {
        println!(
            "testing simple_knapsack_backtracking, dataset: bzoj1625 case {}",
            index
        );

        let (capacity, items, expected_max_value) = misc::load_bzoj1625(index);

        let result = simple_knapsack_backtracking(&items, capacity, false);

        let actual_value: usize = result.iter().map(|item| item.value).sum();

        assert_eq!(
            actual_value, expected_max_value,
            "simple_knapsack_backtracking on bzoj1625 case {} has incorrect maximum value",
            index
        );

        let actual_weight: usize = result.iter().map(|item| item.weight).sum();
        assert!(
            actual_weight <= capacity,
            "simple_knapsack_backtracking on bzoj1625 case {} has incorrect total weight",
            index
        );
    }
}
