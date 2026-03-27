use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hnu_algo::{
    algorithms::bmssp::{BMSSP, const_graph::ConstGraph},
    dataset::ssp,
};

fn bench_bmssp_solve(c: &mut Criterion) {
    let cases: Vec<_> = (1..=4)
        .map(|index| {
            let (source, graph, _) = ssp::load_normal(index);
            let cg = ConstGraph::from_general_graph(&graph);
            let source2 = cg.orig_to_const(source).expect("source in range");
            (index, cg, source2)
        })
        .collect();

    let mut group = c.benchmark_group("bmssp/solve");

    for (index, cg, source2) in &cases {
        group.bench_function(BenchmarkId::new("ssp_case", index), |b| {
            b.iter_batched(
                || BMSSP::new(cg.clone(), *source2),
                |mut bmssp| {
                    bmssp.solve();
                    black_box(());
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_bmssp_solve);
criterion_main!(benches);
