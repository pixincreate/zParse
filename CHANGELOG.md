# Changelog

## [v1.0.0]

### Added
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

## [v0.1.0]

### Added
- Initial release with basic functionality
- Basic JSON and TOML parsing
- Simple conversion between formats