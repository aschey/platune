use std::time::Duration;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use libplatune_management::traverse;
use libplatune_management::traverse_async;

fn from_elem(c: &mut Criterion) {
    let mut group = c.benchmark_group("default");
    // group.sample_size(20);
    group.measurement_time(Duration::from_secs(120));

    group.bench_function("walkdir", |b| {
        b.iter(|| traverse());
    });
    group.bench_function("walkdir_async", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| traverse_async());
    });
    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
