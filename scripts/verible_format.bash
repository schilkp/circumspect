#!/bin/bash
find . -name "*.sv" -o -name "*.svh" -o -name "*.v" | grep -v -E "(third_party|verilator)" | xargs verible-verilog-format $@
