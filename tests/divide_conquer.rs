use hnu_algo::{algorithms, dataset};

#[test]
#[ignore]
fn test_find_min_max_all_sizes() {
    let datasets: Vec<(&str, Vec<usize>)> = vec![
        ("small", dataset::seq::load_normal_small()),
        ("medium", dataset::seq::load_normal_medium()),
        ("large", dataset::seq::load_normal_large()),
    ];

    for (name, data) in datasets {
        println!("testing find_min_max, dataset: {}", name);

        let (min, max) = algorithms::divide_conquer::find_min_max(&data);
        let mut expected_min = 0;
        let mut expected_max = 0;

        for (index, value) in data.iter().enumerate() {
            if value < &data[expected_min] {
                expected_min = index;
            }
            if value > &data[expected_max] {
                expected_max = index;
            }
        }

        assert_eq!(
            min, expected_min,
            "find_min_max on {} dataset has incorrect minimum index",
            name
        );
        assert_eq!(
            max, expected_max,
            "find_min_max on {} dataset has incorrect maximum index",
            name
        );
    }
}

#[test]
#[ignore]
fn test_sort_all_sizes() {
    let datasets: Vec<(&str, Vec<usize>)> = vec![
        ("small", dataset::seq::load_normal_small()),
        ("medium", dataset::seq::load_normal_medium()),
        ("large", dataset::seq::load_normal_large()),
    ];

    for (name, data) in datasets {
        println!("testing sort, dataset: {}", name);

        let sorted_data: Vec<usize> = algorithms::divide_conquer::sort(&data)
            .iter()
            .map(|x| **x)
            .collect();
        let mut expected_data = data.clone();
        expected_data.sort();

        assert_eq!(
            sorted_data, expected_data,
            "sort on {} dataset has incorrect sorted result",
            name
        );
    }
}
