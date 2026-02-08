# Changelog

## [Unreleased]

### Feat

- Rewrite core library with unified JSON/TOML/YAML/XML parsing
- Add native TOML datetime types (offset/local date-time/date/time)
- Add XML parser with attributes, entities, and nested elements
- Add cross-format conversion API for JSON/TOML/YAML/XML
- Add CLI parsing and conversion tool with format inference and permissive JSON flags
- Add Axum API server with parse/convert endpoints

### Improved

- Structured error reporting with spans across all parsers
- Expanded fixtures and property tests for TOML/YAML/XML
- Benchmarks for JSON/TOML/YAML/XML parsing and conversions

### CI

- Run workspace tests/clippy across Linux/macOS/Windows
- Align benchmark and fuzz workflows with current targets

### Chore

- Update repository metadata and ignore docs from tracking

### Security

- Enforce size/depth limits in streaming parsers

## [v1.0.0] - 2025-01-26

### Feat

- Complete rewrite of the library with zero external dependencies for core functionality
- JSON parser

  - Support for all JSON data types (null, boolean, number, string, array, object)
  - Strict JSON validation
  - Detailed error reporting with line and column information
  - Support for nested structures
  - Proper handling of escape sequences in strings

- TOML parser

  - Support for basic key-value pairs
  - Table and nested table support
  - Array tables support
  - Inline tables and arrays
  - Number formats (including underscores)
  - Support for bare keys
  - Proper whitespace handling

- Format conversion

  - Bidirectional conversion between JSON and TOML
  - Structural preservation during conversion
  - Proper type mapping between formats

- Pretty printing

  - Configurable indentation
  - Optional key sorting
  - Format-specific output styling
  - Preservation of data structure

- Error handling

  - Custom error types with detailed messages
  - Line and column information for syntax errors
  - Context-aware error reporting
  - Proper error propagation

- Testing infrastructure

  - Comprehensive unit tests
  - Property-based tests using proptest
  - Conversion test suite
  - Performance benchmarks
  - Fuzzing targets

- Development tools

  - Benchmark suite for performance monitoring
  - Fuzzing setup for finding edge cases
  - CI/CD pipeline with GitHub Actions
  - Code formatting and linting checks
  - Security audit workflow

- Command line interface
  - File parsing support
  - Format conversion
  - Custom output formatting
  - Error reporting

### Improved

- Memory efficiency through string interning
- Performance optimizations for parsing
- Thread safety with parking_lot
- Documentation coverage
- Error messages and debugging information
- Code organization and modularity

### Security

- Regular security audits through cargo-audit
- Fuzzing infrastructure for finding vulnerabilities
- No unsafe code usage
- Input validation and size limits

## [v0.1.0] - 2024-12-30

### Feat

- Initial release with basic functionality
- Basic JSON and TOML parsing
- Simple conversion between formats
