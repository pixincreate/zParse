use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zparse::from_str;

// Test data - include inline for simplicity
const SIMPLE_JSON: &str = r#"{"name": "test", "value": 42}"#;
const NESTED_JSON: &str = r#"{"a": {"b": {"c": [1,2,3]}}}"#;
const ARRAY_JSON: &str = r#"[1, 2, 3, "four", true, null, {"x": 1}]"#;

fn bench_simple(c: &mut Criterion) {
    c.bench_function("zparse_simple", |b| {
        b.iter(|| from_str(black_box(SIMPLE_JSON)))
    });

    c.bench_function("serde_simple", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(SIMPLE_JSON)))
    });
}

fn bench_nested(c: &mut Criterion) {
    c.bench_function("zparse_nested", |b| {
        b.iter(|| from_str(black_box(NESTED_JSON)))
    });

    c.bench_function("serde_nested", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(NESTED_JSON)))
    });
}

fn bench_array(c: &mut Criterion) {
    c.bench_function("zparse_array", |b| {
        b.iter(|| from_str(black_box(ARRAY_JSON)))
    });

    c.bench_function("serde_array", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(ARRAY_JSON)))
    });
}

criterion_group!(benches, bench_simple, bench_nested, bench_array);
criterion_main!(benches);
