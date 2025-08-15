`timescale 10ns / 1ns

module top;
  import cspect_pkg::*;

  ctx   cspect;
  track cpu_track;
  track memory_track;
  track bus_track;

  initial begin
    // Initialize the cspect context
    cspect = new("trace_simple.pftrace");

    // Create tracks with hierarchical structure
    cpu_track = cspect.new_track("CPU", "CPU execution trace");
    memory_track = cspect.new_track("Memory", "Memory operations");
    bus_track = memory_track.new_track("Bus", "Bus transactions");

    // Generate some trace events
    #10;
    cpu_track.slice_begin("instruction_fetch");
    bus_track.instant_evt("read_request");

    #10;
    // Strictly-nested slices on the same track are supported:
    cpu_track.slice_begin("instruction_fetch_child");

    #10;
    // But you can only ever stop the deepest/newest slice:
    cpu_track.slice_end();

    #10;
    // `slice_set` will stop the previous slice (if one is currently
    // active) and start a new slice:
    cpu_track.slice_set("instruction_post_fetch");  // <- ends "instruction_fetch"!

    #10;
    // `slice_set` has a `compress` option, which will only stop the previous
    // slice and start a new one if the name (or flows) are different:
    cpu_track.slice_set("instruction_post_fetch", .compress(1));  // does nothing

    #10;
    memory_track.slice_begin("cache_lookup");

    #15;
    memory_track.instant_evt("cache_hit");
    memory_track.slice_end();

    #10;
    cpu_track.slice_end();

    #5;
    cpu_track.slice_begin("instruction_decode");
    bus_track.instant_evt("write_request");

    #25;
    cpu_track.slice_end();

    #10;
    cpu_track.instant_evt("pipeline_stall");

    // Finish
    cspect.finish();

    $finish;
  end

endmodule
