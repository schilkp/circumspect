#!/bin/bash
set -ev

# Move to location of this script
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# Clean:
rm -rf out
mkdir out

# Compile + Run the examples, grab the outputs:
python ../../examples/run.py
cp ../../examples/simple/out/trace_simple.pftrace ./out/simple.pftrace
cp ../../examples/counters/out/trace_counters.pftrace ./out/counters.pftrace
cp ../../examples/annotations/out/trace_annotations.pftrace ./out/annotations.pftrace
cp ../../examples/flows/out/trace_flows.pftrace ./out/flows.pftrace

# Apply disasembly + addr2line annotations to annotations example trace:
./annotations_elf/build.bash
cargo run -- annotate ./out/annotations.pftrace --disasm --addr2line ./annotations_elf/program.elf -o ./out/annotations2.pftrace

uv run ./check_simple.py ./out/simple.pftrace
uv run ./check_counters.py ./out/counters.pftrace
uv run ./check_annotations.py ./out/annotations2.pftrace
uv run ./check_flows.py ./out/flows.pftrace
