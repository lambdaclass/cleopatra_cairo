use std::path::Path;

use cleopatra_cairo::cairo_run;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark_fibonacci(c: &mut Criterion) {
    c.bench_function("cairo_run(bench/criterion/fibonacci_1000.json", |b| {
        b.iter(|| {
            cairo_run::cairo_run(
                black_box(Path::new("bench/criterion/fibonacci_1000.json")),
                "main",
            )
        })
    });
}

pub fn criterion_benchmark_integration(c: &mut Criterion) {
    c.bench_function("cairo_run(bench/criterion/integration.json", |b| {
        b.iter(|| {
            cairo_run::cairo_run(
                black_box(Path::new("bench/criterion/integration.json")),
                "main",
            )
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark_fibonacci,
    criterion_benchmark_integration
);
criterion_main!(benches);
