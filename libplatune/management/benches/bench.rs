use std::time::Duration;


use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn from_elem(c: &mut Criterion) {
    let mut group = c.benchmark_group("default");
    // group.sample_size(20);
    group.measurement_time(Duration::from_secs(120));

    // group.bench_function("sync", |b| {
    //     b.to_async(tokio::runtime::Runtime::new().unwrap())
    //         .iter(|| sync());
    // });

    group.finish();
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
