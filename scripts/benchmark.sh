#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running all benchmarks...${NC}"
cargo bench

echo -e "\n${GREEN}Running specific benches...${NC}"
cargo bench --bench json
cargo bench --bench toml
cargo bench --bench yaml
cargo bench --bench xml
cargo bench --bench convert

# Setup and run fuzzing if cargo-fuzz is installed
if command -v cargo-fuzz &> /dev/null && [ -f "fuzz/Cargo.toml" ]; then
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
    if [ -d "fuzz/fuzz_targets" ]; then
        for target_file in fuzz/fuzz_targets/*.rs; do
            target=$(basename "${target_file}" .rs)
            echo -e "\n${GREEN}Fuzzing target: ${target}${NC}"
            cargo +nightly fuzz run ${target} -- -max_total_time=30
        done
    fi
else
    echo -e "\n${RED}cargo-fuzz not found. Install with: cargo install cargo-fuzz${NC}"
fi
