`timescale 1ns / 1ns

module top;
  import cspect_pkg::*;

  ctx   cspect;
  track insns;

  initial begin
    // Initialize the cspect context
    cspect = new("trace_annotations.pftrace");

    // Create a normal track:
    insns  = cspect.new_track("Insns", "RISC-V Instructions Executed");

    // Generate some trace events with annotation placeholders:
    #10;
    insns.slice_begin("$da-rv64:0x37650513");  // "addi a0, a0, 886"

    #10;
    insns.slice_end();
    insns.slice_begin("$da-rv64:0x1800");  // "c.addi4spn s0, sp, 48"

    #10;
    insns.slice_end();

    // Finish
    cspect.finish();

    $finish;
  end

endmodule
