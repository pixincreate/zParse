use criterion::{Criterion, black_box, criterion_group, criterion_main};

use zparse::{Format, convert};

const JSON_INPUT: &str = r#"{"name": "test", "value": 42}"#;
const TOML_INPUT: &str = "name = \"test\"\nvalue = 42\n";
const YAML_INPUT: &str = "name: test\nvalue: 42\n";
const XML_INPUT: &str = "<root><name>test</name><value>42</value></root>";

fn bench_json_to_toml(c: &mut Criterion) {
    c.bench_function("convert_json_toml", |b| {
        b.iter(|| convert(black_box(JSON_INPUT), Format::Json, Format::Toml))
    });
}

fn bench_toml_to_json(c: &mut Criterion) {
    c.bench_function("convert_toml_json", |b| {
        b.iter(|| convert(black_box(TOML_INPUT), Format::Toml, Format::Json))
    });
}

fn bench_yaml_to_json(c: &mut Criterion) {
    c.bench_function("convert_yaml_json", |b| {
        b.iter(|| convert(black_box(YAML_INPUT), Format::Yaml, Format::Json))
    });
}

fn bench_xml_to_json(c: &mut Criterion) {
    c.bench_function("convert_xml_json", |b| {
        b.iter(|| convert(black_box(XML_INPUT), Format::Xml, Format::Json))
    });
}

criterion_group!(
    benches,
    bench_json_to_toml,
    bench_toml_to_json,
    bench_yaml_to_json,
    bench_xml_to_json
);
criterion_main!(benches);
