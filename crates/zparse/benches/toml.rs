use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zparse::from_toml_str;

const SIMPLE_TOML: &str = "title = \"TOML\"\ncount = 3\n";
const NESTED_TOML: &str = "[owner]\nname = \"Tom\"\n[database]\nports = [8001, 8001, 8002]\n";

fn bench_simple(c: &mut Criterion) {
    c.bench_function("zparse_toml_simple", |b| {
        b.iter(|| from_toml_str(black_box(SIMPLE_TOML)))
    });

    c.bench_function("toml_simple", |b| {
        b.iter(|| toml::from_str::<toml::Value>(black_box(SIMPLE_TOML)))
    });
}

fn bench_nested(c: &mut Criterion) {
    c.bench_function("zparse_toml_nested", |b| {
        b.iter(|| from_toml_str(black_box(NESTED_TOML)))
    });

    c.bench_function("toml_nested", |b| {
        b.iter(|| toml::from_str::<toml::Value>(black_box(NESTED_TOML)))
    });
}

criterion_group!(benches, bench_simple, bench_nested);
criterion_main!(benches);
