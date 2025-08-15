#!/bin/bash
set -ev

# Move to location of this script
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

CFLAGS="-march=rv64gc_zifencei -mabi=lp64d -mcmodel=medlow -nostdlib -O0 -g"

riscv64-unknown-elf-as crt0.S -o crt0.o
riscv64-unknown-elf-gcc -c $CFLAGS -I . program.c -o program.o
riscv64-unknown-elf-ld -T link.ld crt0.o program.o -o program.elf
