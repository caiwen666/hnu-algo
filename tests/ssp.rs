use hnu_algo::{
    algorithms::ssp::{dijkstra, spfa},
    dataset::ssp,
};

#[test]
#[ignore]
fn test_dijkstra() {
    for index in 1..=4 {
        println!("testing dijkstra, ssp case {}", index);

        let (source, graph, expected) = ssp::load_normal(index);
        let mut actual = dijkstra(&graph, source);
        actual[0] = 0;

        assert_eq!(
            actual, expected,
            "dijkstra on ssp case {} has incorrect shortest distances",
            index
        );
    }
}

#[test]
#[ignore]
fn test_spfa() {
    for index in 1..=4 {
        println!("testing spfa, ssp case {}", index);

        let (source, graph, expected) = ssp::load_normal(index);
        let mut actual = spfa(&graph, source);
        actual[0] = 0;

        assert_eq!(
            actual, expected,
            "spfa on ssp case {} has incorrect shortest distances",
            index
        );
    }
}
