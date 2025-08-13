#!/bin/bash
set -ev

# Move to repo root
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"
cd ..

uv run autopep8 --recursive . --exclude third_party,verilator --in-place
./scripts/verible_format.bash --inplace
cargo fmt
