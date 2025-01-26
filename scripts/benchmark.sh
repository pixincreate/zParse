#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running all benchmarks...${NC}"
cargo bench

echo -e "\n${GREEN}Running specific benchmark groups...${NC}"
cargo bench "JSON Parser"
cargo bench "TOML Parser"
cargo bench "Conversions"

echo -e "\n${GREEN}Generating HTML report...${NC}"
cargo bench --bench parser_benchmarks -- --output-format=criterion

# Setup and run fuzzing if cargo-fuzz is installed
if command -v cargo-fuzz &> /dev/null; then
    echo -e "\n${GREEN}Running fuzz tests...${NC}"

    # Check if nightly is installed
    if ! rustup toolchain list | grep -q "nightly"; then
        echo -e "${GREEN}Installing nightly toolchain...${NC}"
        rustup toolchain install nightly
    fi

    # Initialize fuzzing if not already initialized
    if [ ! -d "fuzz" ]; then
        cargo +nightly fuzz init
    fi

    # Run each fuzzing target for a short duration
    for target in json_parser toml_parser json_converter toml_converter; do
        echo -e "\n${GREEN}Fuzzing target: ${target}${NC}"
        cargo +nightly fuzz run ${target} -- -max_total_time=30
    done
else
    echo -e "\n${RED}cargo-fuzz not found. Install with: cargo install cargo-fuzz${NC}"
fi
