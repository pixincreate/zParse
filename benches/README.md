# Benches

Measures code performance

- In `benches/parser_benchmarks.rs`, it tests:
  ```rust
  - JSON parsing performance
  - TOML parsing performance 
  - Conversion performance
  ```
  
- `parser_benchmarks.rs` contains specific test cases for measuring performance of:

  ```rust
  - JSON parsing performance
  - TOML parsing performance 
  - Conversion performance
  ```
  
- Uses `Criterion.rs` for statistical analysis

## Why do we need this?

Helps optimize performance and catch performance regressions