#!/bin/bash
set -ev

# Move to repo root
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"
cd ..

cargo build
cargo test
cargo clippy
cargo fmt -- --check
uv run autopep8 --diff --exit-code --recursive . --exclude third_party,verilator
./scripts/verible_format.bash --inplace --verify
uv run examples/run.py
./test/test.bash
