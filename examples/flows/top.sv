`timescale 10ns / 1ns

module top;
  import cspect_pkg::*;

  ctx cspect;
  track producer_track;
  track consumer_track;
  uuid_t data_flow;

  initial begin
    cspect = new("trace_flows.pftrace");

    // Create two tracks
    producer_track = cspect.new_track("Producer", "Data producer track");
    consumer_track = cspect.new_track("Consumer", "Data consumer track");

    // Create a flow to connect the slices
    data_flow = producer_track.new_flow();

    #10;
    // Start a slice on the "producer" track:
    producer_track.slice_begin("produce_data");

    #30;
    // End producer slice, while attaching a flow to the end of the slice.
    producer_track.slice_end(.flows({data_flow}));

    #10;

    // Start the second slice on consumer track, attaching the flow which
    // terminates here:
    consumer_track.slice_begin("consume_data", .flows_end({data_flow}));

    #25;
    // End the second slice
    consumer_track.slice_end();

    cspect.finish();

    $finish;
  end

endmodule
