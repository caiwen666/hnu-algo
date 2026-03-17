use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hnu_algo::{algorithms, dataset};

fn std_sort(data: &[usize]) -> Vec<usize> {
    let mut result = data.to_vec();
    result.sort();
    result
}

fn bench_sort(c: &mut Criterion) {
    let small = dataset::seq::load_normal_small();
    let medium = dataset::seq::load_normal_medium();
    let large = dataset::seq::load_normal_large();

    let datasets = [
        ("small", small.as_slice()),
        ("medium", medium.as_slice()),
        ("large", large.as_slice()),
    ];

    let mut group = c.benchmark_group("divide_conquer/sort");

    for (size, data) in datasets {
        group.bench_with_input(BenchmarkId::new("std", size), data, |b, data| {
            b.iter(|| std_sort(black_box(data)))
        });
        group.bench_with_input(BenchmarkId::new("divide_conquer", size), data, |b, data| {
            b.iter(|| algorithms::divide_conquer::sort(black_box(data)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
