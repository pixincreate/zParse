# zParse

## Introduction

zParse is a high-performance Rust library and toolchain for parsing and converting JSON, TOML, YAML, and XML. It ships as a library, a CLI, and an HTTP API so you can integrate it in other Rust projects or expose it to a frontend.

## Features

- Native parsers for JSON, TOML (with native datetime types), YAML 1.2, and XML
- Streaming/event-based parsing with depth and size limits
- Format conversion between all supported formats
- CLI for conversion with stdin/stdout support
- Axum API for programmatic access

## Usage

### Library

```rust
use zparse::{from_str, from_toml_str, from_yaml_str, from_xml_str};

let json_value = from_str(r#"{"name": "zparse"}"#)?;
let toml_value = from_toml_str("name = \"zparse\"\n")?;
let yaml_value = from_yaml_str("name: zparse\n")?;
let xml_doc = from_xml_str("<root><name>zparse</name></root>")?;
# Ok::<(), zparse::Error>(())
```

### Conversion

```rust
use zparse::{convert, Format};

let out = convert(r#"{"name":"zparse"}"#, Format::Json, Format::Toml)?;
# Ok::<(), zparse::Error>(())
```

### CLI

```bash
zparse parse --from json input.json
zparse parse --from toml input.toml
zparse convert --from json --to toml input.json
zparse convert --to yaml input.toml
cat input.xml | zparse convert --from xml --to json
zparse convert --from json --to yaml --json-comments --json-trailing-commas input.json
```

### API

```bash
cargo run -p zparse-api
```

```bash
curl -s http://127.0.0.1:3000/api/health
curl -s -X POST http://127.0.0.1:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{"content":"{\"name\":\"zparse\"}","from":"json","to":"toml"}'
```

## Contribution

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and workflow details.

## License

This project is licensed under the [GPL-3.0 License](LICENSE).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.
