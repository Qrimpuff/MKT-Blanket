use criterion::{criterion_group, criterion_main, Criterion};
use mkt_coverage::*;


fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("draw_ line", |b| b.iter(|| draw_line()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);