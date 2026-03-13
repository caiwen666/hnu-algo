use hnu_algo::{algorithms, dataset};

#[test]
#[ignore]
fn test_find_min_max_small() {
    let data = dataset::seq::load_normal_small();
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
    assert_eq!(min, expected_min);
    assert_eq!(max, expected_max);
}

#[test]
#[ignore]
fn test_find_min_max_medium() {
    let data = dataset::seq::load_normal_medium();
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
    assert_eq!(min, expected_min);
    assert_eq!(max, expected_max);
}

#[test]
#[ignore]
fn test_find_min_max_large() {
    let data = dataset::seq::load_normal_large();
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
    assert_eq!(min, expected_min);
    assert_eq!(max, expected_max);
}
