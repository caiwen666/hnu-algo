use std::time::Instant;

use hnu_algo::{
    algorithms::bmssp::{BMSSP, const_graph::ConstGraph},
    dataset::ssp,
};

fn main() {
    // 把 SSP 数据集中的原图先转为常度数图，然后在常度数图上跑 bmssp，
    let (source, graph, _) = ssp::load_normal(3);
    let cg = ConstGraph::from_general_graph(&graph);
    let source2 = cg.orig_to_const(source).expect("source in range");

    let mut bmssp = BMSSP::new(cg.clone(), source2);
    println!("bmssp initialized");
    let timer = Instant::now();
    bmssp.solve();
    let duration = timer.elapsed();
    println!("bmssp on ssp case 3 took {:?}", duration);
}
