use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hnu_algo::{
    algorithms::ssp::{dijkstra, spfa},
    dataset::ssp,
};

fn bench_ssp(c: &mut Criterion) {
    let cases: Vec<_> = (1..=4)
        .map(|index| {
            let (source, graph, _) = ssp::load_normal(index);
            (index, graph, source)
        })
        .collect();

    let mut group = c.benchmark_group("ssp/dijkstra");
    for (index, graph, source) in &cases {
        group.bench_function(BenchmarkId::new("ssp_case", index), |b| {
            b.iter(|| {
                black_box(dijkstra(black_box(graph.as_slice()), *source));
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("ssp/spfa");
    for (index, graph, source) in &cases {
        group.bench_function(BenchmarkId::new("ssp_case", index), |b| {
            b.iter(|| {
                black_box(spfa(black_box(graph.as_slice()), *source));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ssp);
criterion_main!(benches);
