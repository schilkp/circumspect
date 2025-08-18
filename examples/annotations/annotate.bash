#!/bin/bash
set -ev

# Move to location of this script
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# Build annotations elf:
./annotations_elf/build.bash

# Apply annotations:
cargo run -- annotate ./out/trace_annotations_pre.pftrace --disasm --addr2line ./annotations_elf/program.elf -o ./out/trace_annotations_post.pftrace
