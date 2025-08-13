#!/bin/bash
set -ev

# Move to location of this script
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# Clean run:
rm -rf build
mkdir build

# Compile cspect to generate cbindgen DPI header:
pushd ../../
rm -rf target/debug/cspect_dpi.h
cargo build --package cspect
popd
cp ../../target/debug/cspect_dpi.h ./build/dpi_hdr_cbindgen.h

# Generate DPI header using verilator:
verilator --dpi-hdr-only ../../include/cspect_dpi.svh -Mdir build
mv ./build/Vcspect_dpi__Dpi.h ./build/dpi_hdr_verilator.h

# Compile basic C program including both headers (Would fail if prototypes
# don't match):
gcc ./main.c -Ibuild/ -I../../cspect/ -o ./build/a.out -Wall -Wextra -Wpedantic

./build/a.out
