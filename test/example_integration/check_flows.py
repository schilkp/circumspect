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
    # Check that we have the expected tracks
    tracks = list(tp.query("""
        SELECT name, id
        FROM track
        ORDER BY name
    """))

    print("Tracks: ")
    for track in tracks:
        print(f"  {track}")

    assert len(tracks) == 3
    assert tracks[0].name == "Consumer"
    assert tracks[1].name == "Metadata"
    assert tracks[2].name == "Producer"

    # Check that we have the expected slices
    slices = list(tp.query("""
        SELECT s.id, s.name, s.ts, s.dur, t.name as track_name
        FROM slice s
        JOIN track t ON s.track_id = t.id
        ORDER BY s.ts;
    """))

    # producer_track.slice_begin("produce_data", {data_flow}) at #10 (100ns)
    # producer_track.slice_end({data_flow}) at #110 (1100ns) -> duration = 1000ns
    print("Producer slice:", slices[0])
    assert slices[0].name == "produce_data"
    assert slices[0].track_name == "Producer"
    assert slices[0].ts == 100
    assert slices[0].dur == 1000

    # consumer_track.slice_begin("consume_data", {data_flow}) at #120 (1200ns)
    # consumer_track.slice_end({data_flow}) at #145 (1450ns) -> duration = 250ns
    print("Consumer slice:", slices[9])
    assert slices[9].name == "consume_data"
    assert slices[9].track_name == "Consumer"
    assert slices[9].ts == 1200
    assert slices[9].dur == 250

    # Check flows - there should be a flow connecting producer to consumer
    flows = list(tp.query("""
        SELECT f.id, f.slice_out, f.slice_in, f.trace_id,
               s_out.name as slice_out_name, s_in.name as slice_in_name,
               t_out.name as track_out_name, t_in.name as track_in_name
        FROM flow f
        JOIN slice s_out ON f.slice_out = s_out.id
        JOIN slice s_in ON f.slice_in = s_in.id
        JOIN track t_out ON s_out.track_id = t_out.id
        JOIN track t_in ON s_in.track_id = t_in.id
        WHERE f.slice_out != f.slice_in  -- Only inter-slice flows
        ORDER BY f.id
    """))

    print("Flow connections:")
    for flow in flows:
        print(f"  Flow {flow.id}: {flow.slice_out_name} ({flow.track_out_name}) -> {flow.slice_in_name} ({flow.track_in_name})")

    # We should have 9 flows. One connecting producer to consumer, 8 connecting
    # metadata to the consumer.
    assert len(flows) == 9

    flow = flows[0]
    assert flow.slice_out_name == "produce_data"
    assert flow.slice_in_name == "consume_data"
    assert flow.track_out_name == "Producer"
    assert flow.track_in_name == "Consumer"

    for i in range(1, 9):
        flow = flows[i]
        assert flow.slice_out_name == "Info"
        assert flow.track_out_name == "Metadata"
        assert flow.slice_in_name == "consume_data"
        assert flow.track_in_name == "Consumer"

print("OK!")
