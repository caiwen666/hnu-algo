use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hnu_algo::{algorithms, dataset};

fn simple_find_min_max(data: &[usize]) -> (usize, usize) {
    let mut min_index = 0;
    let mut max_index = 0;
    for (index, value) in data.iter().enumerate() {
        if value < &data[min_index] {
            min_index = index;
        }
        if value > &data[max_index] {
            max_index = index;
        }
    }
    (min_index, max_index)
}

fn bench_min_max(c: &mut Criterion) {
    let small = dataset::seq::load_normal_small();
    let medium = dataset::seq::load_normal_medium();
    let large = dataset::seq::load_normal_large();

    let datasets = [
        ("small", small.as_slice()),
        ("medium", medium.as_slice()),
        ("large", large.as_slice()),
    ];

    let mut group = c.benchmark_group("divide_conquer/min_max");

    for (size, data) in datasets {
        group.bench_with_input(BenchmarkId::new("naive", size), data, |b, data| {
            b.iter(|| simple_find_min_max(black_box(data)))
        });
        group.bench_with_input(BenchmarkId::new("divide_conquer", size), data, |b, data| {
            b.iter(|| algorithms::divide_conquer::find_min_max(black_box(data)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_min_max);
criterion_main!(benches);
