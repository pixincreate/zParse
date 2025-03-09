#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic_in_result_fn)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zparse::test_utils::*;

// Benchmark JSON parsing
fn bench_json_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("JSON Parser");

    let inputs = [
        ("small", include_str!("../tests/input/small.json")),
        ("medium", include_str!("../tests/input/file.json")),
        ("large", include_str!("../tests/input/large.json")),
    ];

    for (size, input) in &inputs {
        group.bench_with_input(BenchmarkId::new("parse", size), input, |b, input| {
            b.iter(|| {
                let mut parser = JsonParser::new(black_box(input)).unwrap();
                parser.parse().unwrap()
            });
        });
    }

    group.finish();
}

// Benchmark TOML parsing
fn bench_toml_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("TOML Parser");

    let inputs = [
        ("small", include_str!("../tests/input/small.toml")),
        ("medium", include_str!("../tests/input/file.toml")),
        ("large", include_str!("../tests/input/large.toml")),
    ];

    for (size, input) in &inputs {
        group.bench_with_input(BenchmarkId::new("parse", size), input, |b, input| {
            b.iter(|| {
                let mut parser = TomlParser::new(black_box(input)).unwrap();
                parser.parse().unwrap()
            });
        });
    }

    group.finish();
}

// Benchmark conversions
fn bench_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Conversions");

    // Setup test data
    let json_input = include_str!("../tests/input/file.json");
    let mut json_parser = JsonParser::new(json_input).unwrap();
    let json_value = json_parser.parse().unwrap();

    let toml_input = include_str!("../tests/input/file.toml");
    let mut toml_parser = TomlParser::new(toml_input).unwrap();
    let toml_value = toml_parser.parse().unwrap();

    group.bench_function("json_to_toml", |b| {
        b.iter(|| Converter::json_to_toml(black_box(&json_value)).unwrap());
    });

    group.bench_function("toml_to_json", |b| {
        b.iter(|| Converter::toml_to_json(black_box(&toml_value)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_parser,
    bench_toml_parser,
    bench_conversions
);
criterion_main!(benches);
