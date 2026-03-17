use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hnu_algo::{algorithms::dp, dataset::misc};

/**
 * 数据集名称和对应 bzoj1625 的 case 编号
 * small
 *  n: 32, capacity: 82, nc ~ 2.6k
 * medium
 *  n: 293, capacity: 801, nc ~ 235k
 * medium_large
 *  n: 551 capacity: 3488, nc ~ 1.9M
 * large
 *  n: 2975, capacity: 10553, nc ~ 31M
 */
const BZOJ1625_BENCH_CASES: &[(&str, usize)] = &[
    ("small", 9),
    ("medium", 10),
    ("medium_large", 8),
    ("large", 6),
];

fn bench_simple_knapsack(c: &mut Criterion) {
    let mut group = c.benchmark_group("dp/simple_knapsack");

    for &(name, index) in BZOJ1625_BENCH_CASES {
        let (capacity, items, _expected) = misc::load_bzoj1625(index);

        group.bench_with_input(
            BenchmarkId::new("bzoj1625", name),
            &(items, capacity),
            |b, (items, capacity)| {
                b.iter(|| dp::simple_knapsack(black_box(items), black_box(*capacity), false))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_simple_knapsack);
criterion_main!(benches);
