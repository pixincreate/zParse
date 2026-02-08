use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use zparse::from_xml_str;

const SIMPLE_XML: &str = "<root><child>text</child></root>";
const ATTR_XML: &str = "<root id=\"1\" name='test'><item value=\"42\" /></root>";

fn bench_simple(c: &mut Criterion) {
    c.bench_function("zparse_xml_simple", |b| {
        b.iter(|| from_xml_str(black_box(SIMPLE_XML)))
    });
}

fn bench_attr(c: &mut Criterion) {
    c.bench_function("zparse_xml_attr", |b| {
        b.iter(|| from_xml_str(black_box(ATTR_XML)))
    });
}

criterion_group!(benches, bench_simple, bench_attr);
criterion_main!(benches);
