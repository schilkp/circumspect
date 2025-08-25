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
        WHERE t.name = "CPU"
        ORDER BY s.ts;
    """))

    # cpu_track.slice_begin("instruction_fetch") at #10 (100ns)
    # cpu_track.slice_set() at #40 (400ns) -> duration = 300ns
    print(cpu_slices[0])
    assert cpu_slices[0].ts == 100
    assert cpu_slices[0].dur == 300
    assert cpu_slices[0].name == "instruction_fetch"

    # cpu_track.slice_begin("instruction_fetch_child") at #20 (200ns)
    # cpu_track.slice_end() at #30 (300ns) -> duration = 100ns
    print(cpu_slices[1])
    assert cpu_slices[1].ts == 200
    assert cpu_slices[1].dur == 100
    assert cpu_slices[1].name == "instruction_fetch_child"

    # cpu_track.slice_set("instruction_post_fetch") at #40 (400ns)
    # cpu_track.slice_end() at #85 (850ns) -> duration = 450ns
    print(cpu_slices[2])
    assert cpu_slices[2].ts == 400
    assert cpu_slices[2].dur == 450
    assert cpu_slices[2].name == "instruction_post_fetch"

    # cpu_track.slice_begin("instruction_decode") at #90 (900ns)
    # cpu_track.slice_end() at #105 (1050ns) -> duration = 250ns
    print(cpu_slices[3])
    assert cpu_slices[3].ts == 900
    assert cpu_slices[3].dur == 250
    assert cpu_slices[3].name == "instruction_decode"

    instant_events = list(tp.query("""
        SELECT i.*, t.name as track_name
        FROM instant i
        JOIN track t ON i.track_id = t.id
        ORDER BY i.ts;
    """))

    print("Instant events:", len(instant_events))

    # bus_track.instant_evt("read_request") at #10 (100ns)
    print(instant_events[0])
    assert instant_events[0].ts == 100
    assert instant_events[0].name == "read_request"
    assert instant_events[0].track_name == "Bus"

    # memory_track.instant_evt("cache_hit") at #75 (750ns)
    print(instant_events[1])
    assert instant_events[1].ts == 750
    assert instant_events[1].name == "cache_hit"
    assert instant_events[1].track_name == "Memory"

    # bus_track.instant_evt("write_request") at #90 (900ns)
    print(instant_events[2])
    assert instant_events[2].ts == 900
    assert instant_events[2].name == "write_request"
    assert instant_events[2].track_name == "Bus"

    # cpu_track.instant_evt("pipeline_stall") at #125 (1250ns)
    print(instant_events[3])
    assert instant_events[3].ts == 1250
    assert instant_events[3].name == "pipeline_stall"
    assert instant_events[3].track_name == "CPU"

print("OK!")
