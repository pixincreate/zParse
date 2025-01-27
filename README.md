# zParse

A zero-dependency robust JSON/TOML parser and converter that handles your data like a cybernetic octopus juggling bits through a quantum circus 🦀🎪✨

## Features

- Parse JSON and TOML files with detailed error handling
- Convert between JSON and TOML formats
- Pretty printing with customizable formatting
- Zero unsafe code
- Comprehensive test coverage including property-based tests
- Extensive fuzzing infrastructure
- Fast and memory efficient

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
zparse = "1.0.0"
```

## Usage

### Command Line

```bash
# Parse and display a file
zparse --file input.json

# Convert between formats
zparse --file input.json --convert toml
zparse --file config.toml --convert json

# Convert and save to file
zparse --file input.json --convert toml --output output.toml
```

### Library

```rust
use zparse::{parse_file, Converter, Result, Value};

// Parse a file
let value = parse_file("config.json")?;

// Parse strings directly
let json_value = zparse::utils::parse_json(json_str)?;
let toml_value = zparse::utils::parse_toml(toml_str)?;

// Convert between formats
let toml_value = Converter::json_to_toml(json_value)?;
let json_value = Converter::toml_to_json(toml_value)?;

// Pretty print values
let formatted_json = zparse::utils::format_json(&value);
let formatted_toml = zparse::utils::format_toml(&value);
```

### Error Handling

The library provides detailed error information:

```rust
match parse_file("config.json") {
    Ok(value) => println!("Parsed successfully: {}", value),
    Err(e) => eprintln!("Error parsing file: {} at line {}, column {}",
        e, e.location().line, e.location().column),
}
```

## Contributing

Contributions welcome! Please read our [contributing guidelines](CONTRIBUTING.md).

To run tests and benchmarks:

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run fuzz testing (requires nightly)
cargo +nightly fuzz run json_parser
cargo +nightly fuzz run toml_parser
```

## License

This project is licensed under GPL-3.0. See [LICENSE](LICENSE) for more details.

