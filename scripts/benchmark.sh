#!/bin/bash
# scripts/bench.sh

# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench "JSON Parser"
cargo bench "TOML Parser"
cargo bench "Conversions"

# Generate HTML report
cargo bench --bench parser_benchmarks -- --html

cargo fuzz run json_parser
cargo fuzz run toml_parser
cargo fuzz run json_converter
cargo fuzz run toml_converter
