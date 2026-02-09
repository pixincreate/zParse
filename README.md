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

Top-level flags (`--parse` / `--convert`) mirror the subcommands and still accept `--from` / `--to` when you want to be explicit.
Root-level flags cannot be combined with subcommands.

```bash
# Auto-detect format from file extension and validate
zparse --parse input.json

# Explicitly set the input format while using top-level flags
zparse --parse input.json --from json

# Auto-detect format from file extension and echo on success
zparse --parse input.json --print-output

# Validate JSON input and print "ok" on success
zparse parse --from json input.json

# Validate TOML input and print "ok" on success
zparse parse --from toml input.toml

# Validate JSON and echo the original content on success
zparse parse --from json --print-output input.json

# Convert JSON to TOML and print "ok" on success
zparse convert --from json --to toml input.json

# Convert TOML to YAML with format inference for the input
zparse convert --to yaml input.toml

# Auto-detect input format and convert to TOML
zparse --convert input.json --to toml

# Convert JSON to YAML and print the converted output on success
zparse convert --from json --to yaml --print-output input.json

# Convert XML from stdin to JSON and write to stdout
cat input.xml | zparse convert --from xml --to json

# Convert permissive JSON (comments + trailing commas) to YAML
zparse convert --from json --to yaml --json-comments --json-trailing-commas input.json
```

#### Quick usage rules

- Use either a subcommand (`parse`/`convert`) or a top-level flag (`--parse`/`--convert`), not both.
- `--to` is required for convert.
- `--from` is optional when an input file path is provided (auto-detects by extension).
- When reading from stdin, you must pass `--from`.

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
