# zParse

A robust parser library for JSON and TOML files written in Rust.

## Features

- Parse JSON and TOML files with detailed error handling
- Convert between JSON and TOML formats
- Pretty printing with customizable formatting
- Zero unsafe code
- Comprehensive test coverage
- Fast and memory efficient

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
zparse = "0.1.0"
```

## Usage

### Command Line

```bash
# Parse a file
zparse --file input.json

# Convert between formats
zparse --file input.json --convert toml --output output.toml
```

### Library

```rust
use zparse::{parse_file, Converter, Result};

// Parse a file
let value = parse_file("config.json")?;

// Convert from JSON to TOML
let toml_value = Converter::json_to_toml(value)?;

// Convert from TOML to JSON
let json_value = Converter::toml_to_json(value)?;
```

## Contributing

Contributions welcome! Please read our [contributing guidelines](CONTRIBUTING.md).

## License

Apache License 2.0
