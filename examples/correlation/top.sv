`timescale 10ns / 1ns

module top;
  import cspect_pkg::*;


  initial begin
    automatic ctx cspect;

    automatic track track1, track2, track3;

    automatic uuid_t group1, group2, group3;

    cspect = new("trace_correlation.pftrace");

    // Multiple events/slices that are related, can be assigned the
    // same correlation ID to be visually grouped in the UI.

    // Create three tracks:
    track1 = cspect.new_track("Track 1");
    track2 = cspect.new_track("Track 2");
    track3 = cspect.new_track("Track 3");

    // Create three groups/correlations:
    group1 = cspect.new_uuid();
    group2 = cspect.new_uuid();
    group3 = cspect.new_uuid();

    #10;
    track1.slice_begin("track1-group1", .correlation_id(group1));
    track2.slice_begin("track2-group2", .correlation_id(group2));
    track3.slice_begin("track3-group3", .correlation_id(group3));

    #10;
    track1.slice_end();
    track2.slice_end();
    track3.slice_end();

    #10;
    track1.slice_begin("track1-group2", .correlation_id(group2));
    track2.slice_begin("track2-group3", .correlation_id(group3));
    track3.slice_begin("track3-group1", .correlation_id(group1));

    #10;
    track1.slice_end();
    track2.slice_end();
    track3.slice_end();

    #10;
    track1.slice_begin("track1-group3", .correlation_id(group3));
    track2.slice_begin("track2-group1", .correlation_id(group1));
    track3.slice_begin("track3-group2", .correlation_id(group2));

    #10;
    track1.slice_end();
    track2.slice_end();
    track3.slice_end();

    cspect.finish();

    $finish;
  end

endmodule
