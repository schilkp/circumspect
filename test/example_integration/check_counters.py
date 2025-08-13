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
    # Check counter tracks exist
    counter_tracks = list(tp.query("""
        SELECT t.* 
        FROM track t 
        WHERE t.type = "global_counter_track_event"
        ORDER BY t.name;
    """))

    print(f"Counter tracks: {len(counter_tracks)}")
    # 11 counter tracks defined (excluding parent track)
    assert len(counter_tracks) == 11

    # Check basic counters
    basic_counter_track = next(
        t for t in counter_tracks if t.name == "My Counter")
    basic_counter2_track = next(
        t for t in counter_tracks if t.name == "My Other Counter")

    print(f"Basic counter track ID: {basic_counter_track.id}")
    print(f"Basic counter2 track ID: {basic_counter2_track.id}")

    # Check counter values for basic_counter
    basic_counter_values = list(tp.query(f"""
        SELECT c.* 
        FROM counter c 
        WHERE c.track_id = {basic_counter_track.id}
        ORDER BY c.ts;
    """))

    print(f"Basic counter values: {len(basic_counter_values)}")
    assert len(basic_counter_values) == 4

    # Check specific values for basic_counter
    assert basic_counter_values[0].ts == 0
    assert basic_counter_values[0].value == 0

    assert basic_counter_values[1].ts == 10
    assert basic_counter_values[1].value == 42

    assert basic_counter_values[2].ts == 20
    assert abs(basic_counter_values[2].value - 3.14159) < 0.00001

    assert basic_counter_values[3].ts == 30
    assert basic_counter_values[3].value == 42

    # Check counter values for basic_counter2
    basic_counter2_values = list(tp.query(f"""
        SELECT c.* 
        FROM counter c 
        WHERE c.track_id = {basic_counter2_track.id}
        ORDER BY c.ts;
    """))

    assert len(basic_counter2_values) == 3
    assert basic_counter2_values[0].value == 0
    assert basic_counter2_values[1].value == 21
    assert basic_counter2_values[2].value == 0

    # Check incremental counters
    total_ops_track = next(t for t in counter_tracks if t.name == "TotalOps")
    cumulative_energy_track = next(
        t for t in counter_tracks if t.name == "CumulativeEnergy")

    total_ops_values = list(tp.query(f"""
        SELECT c.* 
        FROM counter c 
        WHERE c.track_id = {total_ops_track.id}
        ORDER BY c.ts;
    """))

    # Check incremental accumulation - values should accumulate
    assert len(total_ops_values) == 3
    assert total_ops_values[0].value == 5   # Initial: 5
    assert total_ops_values[1].value == 8   # 5 + 3 = 8
    assert total_ops_values[2].value == 4   # 8 - 4 = 4

    # Check unit-based counters
    exec_time_track = next(t for t in counter_tracks if t.name == "ExecTime")
    instruction_track = next(
        t for t in counter_tracks if t.name == "Instructions")
    temp_track = next(t for t in counter_tracks if t.name == "Temperature")

    exec_time_values = list(tp.query(f"""
        SELECT c.* 
        FROM counter c 
        WHERE c.track_id = {exec_time_track.id}
        ORDER BY c.ts;
    """))

    assert len(exec_time_values) == 2
    assert exec_time_values[0].value == 1500
    assert exec_time_values[1].value == 2300

print("OK!")
