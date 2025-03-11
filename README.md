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
- Security protections against DoS attacks
- Detailed error messages with line/column information

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
zparse = "1.0.0"
```

## Verifying Downloads

For security, zParse release binaries come with both SHA-256 checksums (`.sha256` files) and SSH signatures (`.sig` files). It is recommended to verify both before using downloaded binaries.

### Verifying SSH Signatures

SSH signatures cryptographically verify that the binary was signed by the zParse project maintainer's private key.

To verify the digital signatures of the downloads, follow [the steps here](https://github.com/pixincreate/pixincreate/blob/main/VERIFY_SSH_SIGNATURES.md).

### Verifying SHA-256 Checksums

SHA-256 checksums verify file integrity, ensuring the download wasn't corrupted or tampered with.

To verify the SHA256 checksums, use the following commands:

- For Unix-like systems and Windows (Command Prompt):

  ```bash
  sha256sum -c <filename>.sha256
  ```

- For Windows (PowerShell):

  ```powershell
  Get-FileHash "target/package/zparse-<platform>-<platform-arch>.<extension>" -Algorithm SHA256
  ```

If the file is successfully verified, the output will be:

```shell
target/package/<BINARY_NAME>: OK
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

### Library Usage

#### Basic Parsing

```rust
use zparse::{parse_file, utils::{parse_json, parse_toml}, Result, Value};

// Parse a file with automatic format detection
let value = parse_file("config.json")?;

// Parse strings directly
let json_str = r#"{"name": "test", "value": 42}"#;
let json_value = parse_json(json_str)?;

let toml_str = r#"
name = "test"
value = 42
"#;
let toml_value = parse_toml(toml_str)?;
```

#### Format Conversion

```rust
use zparse::{converter::Converter, utils::{parse_json, format_toml}, Result};

// Convert JSON to TOML
let json_str = r#"{"name": "test", "value": 42, "nested": {"key": "value"}}"#;
let json_value = parse_json(json_str)?;
let toml_value = Converter::json_to_toml(&json_value)?;

// Format as TOML string
let toml_str = format_toml(&toml_value)?;
println!("TOML output:\n{}", toml_str);
```

#### Accessing Data

```rust
use zparse::{utils::parse_json, ValueExt, Result};

let json_str = r#"{"name": "test", "values": [1, 2, 3], "settings": {"enabled": true}}"#;
let value = parse_json(json_str)?;

// Use ValueExt trait methods
let name = value.get_string("name").unwrap_or_default();
let enabled = value.get_bool("settings.enabled").unwrap_or(false);

// Direct access
if let Some(Value::Array(values)) = value.as_map().and_then(|m| m.get("values")) {
    for value in values {
        println!("Value: {}", value);
    }
}
```

#### Custom Formatting

```rust
use zparse::{formatter::{FormatConfig, JsonFormatter, Formatter}, utils::parse_json, Result};

let json_str = r#"{"compact": true, "indentation": 4}"#;
let value = parse_json(json_str)?;

// Custom formatting options
let config = FormatConfig {
    indent_spaces: 4,
    sort_keys: true,
};

let formatter = JsonFormatter;
let formatted = formatter.format(&value, &config)?;
println!("Custom formatted:\n{}", formatted);
```

### Error Handling

The library provides detailed error information:

```rust
use zparse::{parse_file, Result};

match parse_file("config.json") {
    Ok(value) => println!("Parsed successfully: {}", value),
    Err(e) => {
        eprintln!("Error parsing file:");
        if let Some(loc) = e.location() {
            eprintln!("  At line {}, column {}", loc.line, loc.column);
        }
        eprintln!("  {}", e);

        // Check specific error types
        match e.kind() {
            ParseErrorKind::Syntax(_) => eprintln!("Syntax error in the file"),
            ParseErrorKind::Semantic(_) => eprintln!("Semantic error in the file"),
            _ => eprintln!("Other error type"),
        }
    }
}
```

## Security Features

zParse includes built-in protection against:

- Stack overflows from deeply nested structures
- Memory exhaustion from large inputs
- CPU denial of service from pathological inputs

These limits can be customized:

```rust
use zparse::{parser::{json::JsonParser, config::ParserConfig}, Result};

let input = r#"{"key": "value"}"#;
let config = ParserConfig {
    max_depth: 32,         // Maximum nesting depth
    max_size: 1_048_576,   // Maximum input size (1MB)
    max_string_length: 10_000, // Maximum string length
    max_object_entries: 1_000, // Maximum entries in an object
};

let parser = JsonParser::new(input)?.with_config(config);
let value = parser.parse()?;
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
