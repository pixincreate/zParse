use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zparse::from_yaml_str;

const SIMPLE_YAML: &str = "name: John\nage: 30\nactive: true\n";
const NESTED_YAML: &str = "person:\n  name: Jane\n  skills:\n    - rust\n    - yaml\n";

fn bench_simple(c: &mut Criterion) {
    c.bench_function("zparse_yaml_simple", |b| {
        b.iter(|| from_yaml_str(black_box(SIMPLE_YAML)))
    });

    c.bench_function("serde_yaml_simple", |b| {
        b.iter(|| serde_yaml::from_str::<serde_yaml::Value>(black_box(SIMPLE_YAML)))
    });
}

fn bench_nested(c: &mut Criterion) {
    c.bench_function("zparse_yaml_nested", |b| {
        b.iter(|| from_yaml_str(black_box(NESTED_YAML)))
    });

    c.bench_function("serde_yaml_nested", |b| {
        b.iter(|| serde_yaml::from_str::<serde_yaml::Value>(black_box(NESTED_YAML)))
    });
}

criterion_group!(benches, bench_simple, bench_nested);
criterion_main!(benches);
