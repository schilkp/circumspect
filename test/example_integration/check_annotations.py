# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "perfetto",
#     "assertpy"
# ]
# ///

import sys

from perfetto.trace_processor import TraceProcessor

if len(sys.argv) != 2:
    print("expect exactly one arg")
    exit(1)
trace_file = sys.argv[1]

with TraceProcessor(trace=trace_file) as tp:
    cpu_slices = list(tp.query("""
        SELECT s.*
        FROM slice s
        JOIN track t ON s.track_id = t.id
        WHERE t.name = "Insns"
        ORDER BY s.ts;
    """))

    # insns.slice_begin("$da-rv64:0x37650513") at #10 (10ns)
    # insns..slice_end() at #20 (20ns) -> duration = 10ns
    print(cpu_slices[0])
    assert cpu_slices[0].ts == 10
    assert cpu_slices[0].dur == 10
    assert cpu_slices[0].name == "addi a0, a0, 886 (0x37650513)"

    # cpu_track.slice_begin("$da-rv64:0x1800") at #20 (20ns)
    # cpu_track.slice_end() at #30 (30ns) -> duration = 10ns
    print(cpu_slices[1])
    assert cpu_slices[1].ts == 20
    assert cpu_slices[1].dur == 10
    assert cpu_slices[1].name == "c.addi4spn s0, sp, 48 (0x00001800)"

    # insns.slice_begin("$a2l:0x20001230") at #30 (30ns)
    # insns.slice_end() at #40 (40ns) -> duration = 10ns
    print(cpu_slices[2])
    assert cpu_slices[2].ts == 30
    assert cpu_slices[2].dur == 10
    assert cpu_slices[2].name == "program.c:6:62:main (0x20001230)"

    # insns.slice_begin("$a2l:0x3000bef0") at #40 (40ns)
    # insns.slice_end() at #50 (50ns) -> duration = 10ns
    print(cpu_slices[3])
    assert cpu_slices[3].ts == 40
    assert cpu_slices[3].dur == 10
    assert cpu_slices[3].name == "program.c:2:71:foobar (0x3000bef0)"

print("OK!")
