#!/bin/bash
set -ev

# Move to location of this script
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# Compile + Run the examples to generate traces:
python ../../examples/run.py

# Run checking scripts:
uv run ./check_simple.py ../../examples/out/trace_simple.pftrace
uv run ./check_counters.py ../../examples/out/trace_counters.pftrace
uv run ./check_annotations.py ../../examples/out/trace_annotations_post.pftrace
uv run ./check_flows.py ../../examples/out/trace_flows.pftrace
