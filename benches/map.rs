use std::mem;

use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};

use comparison_of_maps::*;

fn fill<V: Default, M: Map<u32, V> + Default>(count: u32) -> M {
    let mut map = M::default();

    for index in 0..count {
        map.insert(index, V::default());
    }

    map
}

fn run_one<V, M>(b: &mut Bencher<'_>, n: u32)
where
    V: Default,
    M: Map<u32, V> + Default,
{
    let map = fill::<V, M>(n);
    let mut index = 0;
    b.iter(|| {
        index += 1;
        map.find(index % n)
    });
}

fn run<V: Default>(c: &mut Criterion) {
    let mut group = c.benchmark_group(&format!("size={}", mem::size_of::<V>()));

    for n in [1u32, 5, 10, 15, 30, 50].iter().cloned() {
        group.bench_with_input(BenchmarkId::new("linear map", n), &n, |b, n| {
            run_one::<V, LinearMap<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("binary map", n), &n, |b, n| {
            run_one::<V, BinaryMap<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("kv map", n), &n, |b, n| {
            run_one::<V, KvMap<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("simd8 map", n), &n, |b, n| {
            run_one::<V, SimdMap8<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("simd16 map", n), &n, |b, n| {
            run_one::<V, SimdMap16<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("hash map", n), &n, |b, n| {
            run_one::<V, HashMap<_, _>>(b, *n)
        });
        group.bench_with_input(BenchmarkId::new("btree map", n), &n, |b, n| {
            run_one::<V, BTreeMap<_, _>>(b, *n)
        });
    }

    group.finish();
}

fn map(c: &mut Criterion) {
    run::<[u64; 1]>(c);
    run::<[u64; 4]>(c);
    run::<[u64; 8]>(c);
    run::<[u64; 16]>(c);
}

criterion_group!(benches, map);
criterion_main!(benches);
