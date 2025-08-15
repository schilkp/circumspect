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

    // Generate some trace events with annotation placeholders.
    // $da-rv64 disassembles the number into a 64-bit RISC-V instruction:
    #10;
    insns.slice_begin("$da-rv64:0x37650513");  // "addi a0, a0, 886"

    #10;
    insns.slice_end();
    insns.slice_begin("$da-rv64:0x1800");  // "c.addi4spn s0, sp, 48"

    // $a2l uses the debug information inside an ELF file to convert an instruction
    // address into a function name + source file location.
    #10;
    insns.slice_end();
    insns.slice_begin("$a2l:0x20001230");  // "program.c:6:62:main"

    #10;
    insns.slice_end();
    insns.slice_begin("$a2l:0x3000bef0");  // "program.c:2:71:foobar"

    #10;
    insns.slice_end();

    // Finish
    cspect.finish();

    $finish;
  end

endmodule
