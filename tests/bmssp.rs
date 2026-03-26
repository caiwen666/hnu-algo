use hnu_algo::{
    algorithms::{
        bmssp::{BMSSP, const_graph::ConstGraph},
        ssp::dijkstra,
    },
    dataset::ssp,
};

#[test]
#[ignore]
fn test_const_graph() {
    // 把 SSP 数据集中的原图先转为常度数图，然后在常度数图上跑 dijkstra，
    // 再把“代表点距离”投影回原图点进行验证。
    for index in 1..=4 {
        println!("testing dijkstra on const-degree graph, ssp case {}", index);

        let (source, graph, expected) = ssp::load_normal(index);
        let cg = ConstGraph::from_general_graph(&graph);
        let source2 = cg.orig_to_const(source).expect("source in range");

        let dist2 = dijkstra(cg.adj(), source2);
        let mut actual = vec![u64::MAX; graph.len()];
        for v in 0..graph.len() {
            let rv = cg.orig_to_const(v).expect("v in range");
            actual[v] = dist2[rv];
        }

        // 数据集是 1-indexed，0 号点是占位位。
        actual[0] = 0;

        assert_eq!(
            actual, expected,
            "dijkstra on const-degree graph, ssp case {} has incorrect shortest distances",
            index
        );
    }
}

#[test]
#[ignore]
fn test_bmssp() {
    // 把 SSP 数据集中的原图先转为常度数图，然后在常度数图上跑 bmssp，
    for index in 1..=4 {
        println!("testing bmssp on ssp case {}", index);

        let (source, graph, expected) = ssp::load_normal(index);
        let cg = ConstGraph::from_general_graph(&graph);
        let source2 = cg.orig_to_const(source).expect("source in range");

        let bmssp = BMSSP::new(cg.clone(), source2);
        let dist2 = bmssp.solve();

        let mut actual = vec![u64::MAX; graph.len()];
        for v in 0..graph.len() {
            let rv = cg.orig_to_const(v).expect("v in range");
            actual[v] = dist2[rv];
        }

        // 数据集是 1-indexed，0 号点是占位位。
        actual[0] = 0;

        assert_eq!(
            actual, expected,
            "bmssp on ssp case {} has incorrect shortest distances",
            index
        );
    }
}
