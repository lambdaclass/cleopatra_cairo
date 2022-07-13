use std::path::Path;

use cleopatra_cairo::cairo_run;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const BENCH_NAMES: &'static [&'static str] = &[
    "compare_arrays_200000",
    "factorial_multirun",
    "fibonacci_1000",
    "fibonacci_1000_multirun",
    "integration_builtins",
    "linear_search",
];
const BENCH_PATH: &'static str = "cairo_programs/benchmarks/";

pub fn criterion_benchmarks(c: &mut Criterion) {
    for benchmark_name in build_bench_strings() {
        c.bench_function(&benchmark_name.0, |b| {
            b.iter(|| cairo_run::cairo_run(black_box(Path::new(&benchmark_name.1)), "main"))
        });
    }
}

fn build_bench_strings() -> Vec<(String, String)> {
    let mut full_string = Vec::<(String, String)>::new();

    for filename in BENCH_NAMES {
        let file_no_extension = String::from(*filename);
        let file_extension = String::from(".json");
        let bench_path = String::from(BENCH_PATH);
        let cairo_call = String::from("cairo_run(");
        let full_file_path = bench_path + &file_no_extension + &file_extension;
        full_string.push((cairo_call + &full_file_path.clone(), full_file_path));
    }

    full_string
}

criterion_group!(benches, criterion_benchmarks);
criterion_main!(benches);
