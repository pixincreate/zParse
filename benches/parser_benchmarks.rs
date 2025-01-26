#![allow(clippy::unwrap_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zparse::parser::{JsonParser, TomlParser};

fn bench_json_parser(c: &mut Criterion) {
    let input = include_str!("../tests/input/large.json");

    c.bench_function("parse_json", |b| {
        b.iter(|| {
            let mut parser = JsonParser::new(black_box(input)).unwrap();
            parser.parse().unwrap()
        })
    });
}

fn bench_toml_parser(c: &mut Criterion) {
    let input = include_str!("../tests/input/large.toml");

    c.bench_function("parse_toml", |b| {
        b.iter(|| {
            let mut parser = TomlParser::new(black_box(input)).unwrap();
            parser.parse().unwrap()
        })
    });
}

criterion_group!(benches, bench_json_parser, bench_toml_parser);
criterion_main!(benches);
