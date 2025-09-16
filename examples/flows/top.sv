`timescale 10ns / 1ns

module top;
  import cspect_pkg::*;


  initial begin
    automatic ctx cspect;

    automatic track producer_track;
    automatic track metadata_track;
    automatic track consumer_track;

    automatic uuid_t data_flow;
    automatic uuid_t metadata_flows[];

    cspect = new("trace_flows.pftrace");

    // Create two tracks
    producer_track = cspect.new_track("Producer", "Data producer track");
    metadata_track = cspect.new_track("Metadata", "Metadata track");
    consumer_track = cspect.new_track("Consumer", "Data consumer track");

    #10;
    // Start a slice on the "producer" track:
    producer_track.slice_begin("produce_data");

    // Create 8 "metadata" instant events:
    for (int i = 0; i < 8; i++) begin
      automatic uuid_t flow_id;
      #10;

      // Create a flow that will connect the metadata event to the
      // "consume_data" slice:
      flow_id = cspect.new_uuid();

      metadata_flows = {metadata_flows, flow_id};

      // Create an instant metadata event, and attach the new flow to it:
      metadata_track.instant_evt("Info", .flows({flow_id}));
    end

    #20;
    // Create a flow that will connect the "produce_data" slice to the
    // "consume_data" slice:
    data_flow = cspect.new_uuid();
    // End producer slice, while attaching a flow to the end of the slice:
    producer_track.slice_end(.flows({data_flow}));

    #10;
    // Start the second slice on consumer track, attaching the flows from
    // the producer slice and metadata events:
    consumer_track.slice_begin("consume_data", .flows_end({data_flow, metadata_flows}));

    #25;
    // End the second slice
    consumer_track.slice_end();

    cspect.finish();

    $finish;
  end

endmodule
