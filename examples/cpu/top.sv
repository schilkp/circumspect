`timescale 1ns / 1ps

//--------------------------------------------------------------------------
// Instruction Set and Opcodes
//--------------------------------------------------------------------------
typedef enum logic [4:0] {
  OP_LDI_A  = 5'h00, // Load Immediate to r_a
  OP_LDI_B  = 5'h01, // Load Immediate to r_b
  OP_ADD    = 5'h02, // r_b <= r_b + r_a
  OP_DEC    = 5'h03, // r_a <= r_a - 1
  OP_BZ_A   = 5'h04, // Branch if r_a is zero
  OP_BZ_B   = 5'h05, // Branch if r_b is zero
  OP_JMP    = 5'h06, // Unconditional Jump
  OP_LD_A   = 5'h07, // Load from RAM to r_a
  OP_ST_A   = 5'h08, // Store r_a to RAM
  OP_LD_B   = 5'h09, // Load from RAM to r_b
  OP_ST_B   = 5'h0A, // Store r_b to RAM
  OP_CALL   = 5'h0B, // Call function (push PC+1 to stack, jump to address)
  OP_RET    = 5'h0C, // Return from function (pop PC from stack)
  OP_PUSH_A = 5'h0D, // Push r_a to stack
  OP_PUSH_B = 5'h0E, // Push r_b to stack
  OP_POP_A  = 5'h0F, // Pop from stack to r_a
  OP_POP_B  = 5'h10, // Pop from stack to r_b
  OP_SWAP   = 5'h11, // Swap register A and B
  OP_NOP    = 5'h12, // No Operation
  OP_HALT   = 5'h13  // Halt execution
} opcode_t;

`define INSTR(opcode, literal) {opcode, 3'b000, literal}

//--------------------------------------------------------------------------
// A simple CPU
//--------------------------------------------------------------------------
module cpu (
    input logic clk_i,
    input logic rst_ni,

    // cspect interface
    input cspect_pkg::scope cspect_scope_i,
    input event             cspect_ready_i,

    // ROM Interface
    output logic [ 7:0] rom_addr_o,
    input  logic [15:0] rom_data_i,

    // RAM Interface
    output logic [7:0] ram_addr_o,
    output logic [7:0] ram_data_o,
    output logic       ram_write_o,
    output logic       ram_read_o,
    input  logic [7:0] ram_data_i,

    // Status
    output logic halt_o
);


  // ==== Types + Utils ========================================================

  typedef enum logic [2:0] {
    S_FETCH_EXEC,  // Combined Fetch, Decode, and Execute
    S_LD_A_WAIT,   // Wait for RAM data for LD_A or POP_A
    S_LD_B_WAIT,   // Wait for RAM data for LD_B or POP_B
    S_RET_WAIT,    // Wait for RET stack pop data from RAM
    S_HALT         // Halted state
  } state_t;

  function automatic string state_to_string(input state_t state);
    case (state)
      S_FETCH_EXEC: return "FETCH_EXEC";
      S_LD_A_WAIT:  return "LD_A_WAIT";
      S_LD_B_WAIT:  return "LD_B_WAIT";
      S_RET_WAIT:   return "RET_WAIT";
      S_HALT:       return "HALT";
      default:      return "UNKNOWN";
    endcase
  endfunction

  function automatic string disasm(input logic [15:0] instr);
    opcode_t op;
    logic [7:0] payload;

    op = opcode_t'(instr[15:11]);
    payload = instr[7:0];

    case (op)
      OP_LDI_A:  return $sformatf("LDI_A #%0d", payload);
      OP_LDI_B:  return $sformatf("LDI_B #%0d", payload);
      OP_ADD:    return "ADD";
      OP_DEC:    return "DEC";
      OP_BZ_A:   return $sformatf("BZ_A %0d", payload);
      OP_BZ_B:   return $sformatf("BZ_B %0d", payload);
      OP_JMP:    return $sformatf("JMP %0d", payload);
      OP_LD_A:   return $sformatf("LD_A [%0d]", payload);
      OP_ST_A:   return $sformatf("ST_A [%0d]", payload);
      OP_LD_B:   return $sformatf("LD_B [%0d]", payload);
      OP_ST_B:   return $sformatf("ST_B [%0d]", payload);
      OP_CALL:   return $sformatf("CALL %0d", payload);
      OP_RET:    return "RET";
      OP_PUSH_A: return "PUSH_A";
      OP_PUSH_B: return "PUSH_B";
      OP_POP_A:  return "POP_A";
      OP_POP_B:  return "POP_B";
      OP_SWAP:   return "SWAP";
      OP_NOP:    return "NOP";
      OP_HALT:   return "HALT";
      default:  return $sformatf("UNK_0x%x", op);
    endcase
  endfunction

  // ==== CPU State ============================================================

  state_t state_q, state_d;

  logic [7:0] pc_q, pc_d;  // Program Counter
  logic [7:0] r_a_q, r_a_d;  // Register A
  logic [7:0] r_b_q, r_b_d;  // Register B
  logic halt_q, halt_d;  // Halt flag

  // RAM-based stack with top-of-stack pointer (starts at 0xFF, grows down)
  logic [7:0] stack_ptr_q, stack_ptr_d;  // Stack pointer (0xFF down to 0x00)

  // ==== Execution Logic ======================================================

  // Decode instruction:
  logic [4:0] opcode;
  logic [7:0] payload;

  always_comb begin
    opcode  = rom_data_i[15:11];
    payload = rom_data_i[7:0];
  end

  // FSM:
  always_comb begin
    // Default Assignment
    state_d     = state_q;
    pc_d        = pc_q;
    r_a_d       = r_a_q;
    r_b_d       = r_b_q;
    halt_d      = halt_q;
    stack_ptr_d = stack_ptr_q;

    rom_addr_o  = pc_q;
    ram_addr_o  = '0;
    ram_data_o  = '0;
    ram_write_o = 1'b0;
    ram_read_o  = 1'b0;

    case (state_q)
      S_FETCH_EXEC: begin

        case (opcode)
          OP_LDI_A: begin
            r_a_d = payload;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_LDI_B: begin
            r_b_d = payload;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_ADD: begin
            r_b_d = r_b_q + r_a_q;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_DEC: begin
            r_a_d = r_a_q - 1;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_BZ_A: begin
            if (r_a_q == 8'h00) begin
              pc_d = payload;
            end else begin
              pc_d = pc_q + 1;
            end
            state_d = S_FETCH_EXEC;
          end
          OP_BZ_B: begin
            if (r_b_q == 8'h00) begin
              pc_d = payload;
            end else begin
              pc_d = pc_q + 1;
            end
            state_d = S_FETCH_EXEC;
          end
          OP_JMP: begin
            pc_d = payload;
            state_d = S_FETCH_EXEC;
          end
          OP_LD_A: begin
            ram_addr_o = payload;
            ram_read_o = 1'b1;
            state_d = S_LD_A_WAIT;
          end
          OP_ST_A: begin
            ram_addr_o  = payload;
            ram_data_o  = r_a_q;
            ram_write_o = 1'b1;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_LD_B: begin
            ram_addr_o = payload;
            ram_read_o = 1'b1;
            state_d = S_LD_B_WAIT;
          end
          OP_ST_B: begin
            ram_addr_o  = payload;
            ram_data_o  = r_b_q;
            ram_write_o = 1'b1;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_CALL: begin
            // Push return address (PC+1) onto RAM stack
            ram_addr_o = stack_ptr_q;
            ram_data_o = pc_q + 1;
            ram_write_o = 1'b1;
            stack_ptr_d = stack_ptr_q - 1;  // Stack grows down
            // Jump to called function
            pc_d = payload;
            state_d = S_FETCH_EXEC;
          end
          OP_RET: begin
            // Pop return address from RAM stack
            stack_ptr_d = stack_ptr_q + 1;  // Move up to last pushed value
            ram_addr_o = stack_ptr_q + 1;
            ram_read_o = 1'b1;
            state_d = S_RET_WAIT;
          end
          OP_PUSH_A: begin
            // Push register A onto RAM stack
            ram_addr_o = stack_ptr_q;
            ram_data_o = r_a_q;
            ram_write_o = 1'b1;
            stack_ptr_d = stack_ptr_q - 1;  // Stack grows down
            pc_d = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_PUSH_B: begin
            // Push register B onto RAM stack
            ram_addr_o = stack_ptr_q;
            ram_data_o = r_b_q;
            ram_write_o = 1'b1;
            stack_ptr_d = stack_ptr_q - 1;  // Stack grows down
            pc_d = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_POP_A: begin
            // Pop from RAM stack to register A
            stack_ptr_d = stack_ptr_q + 1;  // Move up to last pushed value
            ram_addr_o = stack_ptr_q + 1;
            ram_read_o = 1'b1;
            state_d = S_LD_A_WAIT;
          end
          OP_POP_B: begin
            // Pop from RAM stack to register B
            stack_ptr_d = stack_ptr_q + 1;  // Move up to last pushed value
            ram_addr_o = stack_ptr_q + 1;
            ram_read_o = 1'b1;
            state_d = S_LD_B_WAIT;
          end
          OP_SWAP: begin
            r_a_d = r_b_q;
            r_b_d = r_a_q;
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_NOP: begin
            pc_d    = pc_q + 1;
            state_d = S_FETCH_EXEC;
          end
          OP_HALT: begin
            halt_d  = 1'b1;
            state_d = S_HALT;
          end
          default: begin
            // Treat unknown opcodes as HALT
            halt_d  = 1'b1;
            state_d = S_HALT;
          end
        endcase
      end

      S_LD_A_WAIT: begin
        // Latch data from RAM into register A and proceed (LD_A or POP_A)
        r_a_d = ram_data_i;
        pc_d = pc_q + 1;
        state_d = S_FETCH_EXEC;
      end

      S_LD_B_WAIT: begin
        // Latch data from RAM into register B and proceed (LD_B or POP_B)
        r_b_d = ram_data_i;
        pc_d = pc_q + 1;
        state_d = S_FETCH_EXEC;
      end

      S_RET_WAIT: begin
        // RET stack pop completed, jump to return address
        pc_d = ram_data_i;
        state_d = S_FETCH_EXEC;
      end

      S_HALT: begin
        // Stay in HALT state
        pc_d = pc_q;
        state_d = S_HALT;
      end

      default: state_d = S_FETCH_EXEC;
    endcase
  end

  assign halt_o = halt_q;

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      state_q     <= S_FETCH_EXEC;
      pc_q        <= '0;
      r_a_q       <= '0;
      r_b_q       <= '0;
      halt_q      <= 1'b0;
      stack_ptr_q <= 8'hFF;  // Stack starts at top of RAM and grows down
    end else begin
      state_q     <= state_d;
      pc_q        <= pc_d;
      r_a_q       <= r_a_d;
      r_b_q       <= r_b_d;
      halt_q      <= halt_d;
      stack_ptr_q <= stack_ptr_d;
    end
  end

  // ==== CircumSpect ==========================================================

  // basic CPU state:
  cspect_pkg::track track_reg_a;
  cspect_pkg::track track_reg_b;
  cspect_pkg::track track_pc;
  cspect_pkg::track track_insn;
  cspect_pkg::track track_state;
  cspect_pkg::track track_stack_ptr;

  // stack content:
  cspect_pkg::track track_stack;

  // Setup:
  always @(cspect_ready_i) begin
    track_reg_a = cspect_scope_i.new_track("Register A");
    track_reg_b = cspect_scope_i.new_track("Register B");
    track_pc = cspect_scope_i.new_track("Program Counter");
    track_insn = cspect_scope_i.new_track("Current Instruction");
    track_state = cspect_scope_i.new_track("CPU State");
    track_stack_ptr = cspect_scope_i.new_track("Stack Pointer");

    track_stack = cspect_scope_i.new_track("Stack");
  end

  // Tracing:
  always @(posedge clk_i) begin
    track_reg_a.slice_set($sformatf("0x%02x", r_a_d), .compress(1));
    track_reg_b.slice_set($sformatf("0x%02x", r_b_d), .compress(1));
    track_pc.slice_set($sformatf("0x%02x", pc_d), .compress(1));
    track_insn.slice_set($sformatf("%s (0x%02x)", disasm(rom_data_i), rom_data_i), .compress(1));
    track_state.slice_set(state_to_string(state_d), .compress(1));
    track_stack_ptr.slice_set($sformatf("0x%02x", stack_ptr_d), .compress(1));

    if (rst_ni && state_q == S_FETCH_EXEC) begin
      case (opcode)
        OP_CALL: track_stack.slice_begin($sformatf("0x%02x (CALL)", pc_q + 1));
        OP_PUSH_A: track_stack.slice_begin($sformatf("0x%02x (PUSH_A)", r_a_q));
        OP_PUSH_B: track_stack.slice_begin($sformatf("0x%02x (PUSH_B)", r_b_q));
        OP_RET, OP_POP_B, OP_POP_A: track_stack.slice_end();
        default: begin  /* nothing */
        end
      endcase
    end
  end
endmodule

//===----------------------------------------------------------------------===//
// Combinational ROM Module
//===----------------------------------------------------------------------===//
module rom (
    input  logic [ 7:0] address_i,
    output logic [15:0] data_o
);

  always_comb begin
    case (address_i)
      // Program: Calculate sum of 5 down to 1 (5+4+3+2+1=15) iteratively and recursively

      // === Part 1: Iterative calculation (stores result in RAM[0]) ===========
      // r_a = counter, r_b = sum
      8'h00: data_o = `INSTR(OP_LDI_A, 8'h05);  // counter = 5
      8'h01: data_o = `INSTR(OP_LDI_B, 8'h00);  // sum = 0
      // loop_start:
      8'h02: data_o = `INSTR(OP_ADD, 8'h00);  // sum = sum + counter
      8'h03: data_o = `INSTR(OP_DEC, 8'h00);  // counter = counter - 1
      8'h04: data_o = `INSTR(OP_BZ_A, 8'h06);  // if r_a is zero, jump to end_iterative
      8'h05: data_o = `INSTR(OP_JMP, 8'h02);  // jump back to loop_start
      // end_iterative:
      8'h06: data_o = `INSTR(OP_ST_B, 8'h00);  // Store iterative sum to RAM[0]

      // === Part 2: Recursive calculation (stores result in RAM[1]) ===========
      // Call recursive function with initial value 5
      8'h07: data_o = `INSTR(OP_LDI_A, 8'h05);  // load initial value
      8'h08: data_o = `INSTR(OP_CALL, 8'h0B);  // call recursive function
      8'h09: data_o = `INSTR(OP_ST_A, 8'h01);  // store result in RAM[1]
      8'h0A: data_o = `INSTR(OP_JMP, 8'h17);  // jump to end

      // Recursive function: fn(x) = x == 0 ? 0 : fn(x-1) + x
      // Input: r_a = x, Output: r_a = result
      // recursive_fn:
      8'h0B: data_o = `INSTR(OP_BZ_A, 8'h15);  // if x == 0, return 0
      8'h0C: data_o = `INSTR(OP_PUSH_A, 8'h00);  // save current x
      8'h0D: data_o = `INSTR(OP_DEC, 8'h00);  // x = x - 1
      8'h0E: data_o = `INSTR(OP_CALL, 8'h0B);  // recursive call fn(x-1)
      8'h0F: data_o = `INSTR(OP_PUSH_A, 8'h00);  // save fn(x-1) result
      8'h10: data_o = `INSTR(OP_POP_B, 8'h00);  // r_b = fn(x-1)
      8'h11: data_o = `INSTR(OP_POP_A, 8'h00);  // r_a = original x
      8'h12: data_o = `INSTR(OP_ADD, 8'h00);  // r_b = fn(x-1) + x
      8'h13: data_o = `INSTR(OP_SWAP, 8'h00);  // r_a = result
      8'h14: data_o = `INSTR(OP_RET, 8'h00);  // return result in r_a
      // base_case:
      8'h15: data_o = `INSTR(OP_LDI_A, 8'h00);  // return 0 for base case
      8'h16: data_o = `INSTR(OP_RET, 8'h00);  // return 0

      // === Part 3: END =======================================================
      8'h17: data_o = `INSTR(OP_NOP, 8'h00);
      8'h18: data_o = `INSTR(OP_NOP, 8'h00);
      8'h19: data_o = `INSTR(OP_NOP, 8'h00);
      8'h1A: data_o = `INSTR(OP_HALT, 8'h00);

      default: data_o = `INSTR(OP_HALT, 8'h00);  // default to HALT
    endcase
  end

endmodule

//===----------------------------------------------------------------------===//
// Synchronous RAM Module
//===----------------------------------------------------------------------===//
module ram (
    input logic clk_i,
    input logic rst_ni,

    // cspect interface
    input cspect_pkg::scope cspect_scope_i,
    input event             cspect_ready_i,

    input  logic [7:0] addr_i,
    input  logic [7:0] data_i,
    input  logic       write_i,
    input  logic       read_i,
    output logic [7:0] data_o
);

  logic [7:0] mem_q[256], mem_d[256];
  logic [7:0] read_data_q, read_data_d;

  assign data_o = read_data_q;

  always_comb begin
    mem_d = mem_q;
    read_data_d = read_data_q;

    if (write_i) begin
      mem_d[addr_i] = data_i;
    end

    if (read_i) begin
      read_data_d = mem_q[addr_i];
    end

  end

  always_ff @(posedge clk_i or negedge rst_ni) begin
    if (!rst_ni) begin
      for (int i = 0; i < 256; i++) begin
        mem_q[i] <= '0;
      end
    end else begin
      mem_q <= mem_d;
      read_data_q <= read_data_d;
    end
  end


  // ==== CircumSpect ==========================================================

  // RAM content:
  // Parent track for all RAM-content tracks:
  cspect_pkg::track track_content;
  // We only create tracks for the first 4 and last 16 memory locations:
  cspect_pkg::track tracks_begin[4];
  cspect_pkg::track tracks_end[16];

  // RAM Operations
  cspect_pkg::track track_ops;

  // Setup:
  always @(cspect_ready_i) begin
    // Parent track with explicit child ordering:
    // (children are ordered by their child_order_rank property.
    track_content = cspect_scope_i.new_track("RAM Content", .child_ordering(cspect_pkg::Explicit));

    // Track for first and last 16 memory locs:
    for (int i = 0; i < 4; i++) begin
      string name = $sformatf("ram[0x%02x]", i);
      tracks_begin[i] = track_content.new_track(name, .child_order_rank(i));
    end
    for (int i = 0; i < 16; i++) begin
      string name = $sformatf("ram[0x%02x]", 255 - i);
      tracks_end[i] = track_content.new_track(name, .child_order_rank(255 - i));
    end

    // Track for memory operations (reads/writes):
    track_ops = cspect_scope_i.new_track("Operations");
  end

  // Tracing:
  always_ff @(posedge clk_i) begin
    for (int i = 0; i < 4; i++) begin
      tracks_begin[i].slice_set($sformatf("0x%02x", mem_q[i]), .compress(1));
    end
    for (int i = 0; i < 16; i++) begin
      tracks_end[i].slice_set($sformatf("0x%02x", mem_q[255-i]), .compress(1));
    end

    if (read_i) track_ops.instant_evt($sformatf("Read @ 0x%02x: 0x%02x", addr_i, mem_q[addr_i]));
    if (write_i) track_ops.instant_evt($sformatf("Write @ 0x%02x: 0x%02x", addr_i, data_i));
  end
endmodule

//===----------------------------------------------------------------------===//
// Top-Level Testbench
//===----------------------------------------------------------------------===//
module top;

  // ==== Clock + Reset Gen  ===================================================
  logic clk;
  logic rst_n;

  // Generate a 100MHz clock (10ns period)
  initial begin
    clk = 0;
    forever #5 clk = ~clk;
  end

  // Generate reset pulse
  initial begin
    rst_n = 1'b0;
    #20;
    rst_n = 1'b1;
  end

  // ==== Interconnect =========================================================
  logic             [ 7:0] rom_addr;
  logic             [15:0] rom_data;
  logic             [ 7:0] ram_addr;
  logic             [ 7:0] ram_data_out;
  logic             [ 7:0] ram_data_in;
  logic                    ram_write;
  logic                    ram_read;
  logic                    halt;

  // ==== CircumSpect Setup ====================================================
  cspect_pkg::ctx          cspect_ctx;
  cspect_pkg::scope        cspect_scope_cpu;
  cspect_pkg::scope        cspect_scope_ram;
  cspect_pkg::track        track_sim;
  event                    cspect_ready;

  initial begin
    // Create cspect context:
    string fn = "trace_cpu.pftrace";
    cspect_ctx = new(fn);
    $display("Started cspect trace @ %s", fn);

    track_sim = cspect_ctx.new_track("Simulation");
    track_sim.instant_evt("Start");

    // Create scopes for child modules:
    cspect_scope_cpu = cspect_ctx.new_track("CPU");
    cspect_scope_ram = cspect_ctx.new_track("RAM");

    // Signal to child modules that cspect is ready:
    ->cspect_ready;
  end

  final begin
    cspect_ctx.finish();
  end

  // ==== Instances ============================================================
  cpu i_cpu (
      .clk_i         (clk),
      .rst_ni        (rst_n),
      .cspect_scope_i(cspect_scope_cpu),
      .cspect_ready_i(cspect_ready),
      .rom_addr_o    (rom_addr),
      .rom_data_i    (rom_data),
      .ram_addr_o    (ram_addr),
      .ram_data_o    (ram_data_out),
      .ram_write_o   (ram_write),
      .ram_read_o    (ram_read),
      .ram_data_i    (ram_data_in),
      .halt_o        (halt)
  );

  rom i_rom (
      .address_i(rom_addr),
      .data_o   (rom_data)
  );

  ram i_ram (
      .clk_i(clk),
      .rst_ni(rst_n),
      .cspect_scope_i(cspect_scope_ram),
      .cspect_ready_i(cspect_ready),
      .addr_i(ram_addr),
      .data_i(ram_data_out),
      .write_i(ram_write),
      .read_i(ram_read),
      .data_o(ram_data_in)
  );

  // ==== Simulation Control ===================================================
  initial begin
    $display("Starting simulation...");
    wait (halt);
    $display("CPU has halted at time %0t.", $time);

    // Run a few more clk cycles.
    #25;

    track_sim.instant_evt("End");

    // Check the iterative result stored in RAM[0]
    if (i_ram.mem_q[0] == 15) begin
      $display("SUCCESS: Iterative result stored in RAM[0] is 15.");
    end else begin
      $display("FAILURE: Iterative result in RAM[0] is %d, expected 15.", i_ram.mem_q[0]);
      $fatal(1);
    end

    // Check the recursive result stored in RAM[1]
    if (i_ram.mem_q[1] == 15) begin
      $display("SUCCESS: Recursive result stored in RAM[1] is 15.");
    end else begin
      $display("FAILURE: Recursive result in RAM[1] is %d, expected 15.", i_ram.mem_q[1]);
      $fatal(1);
    end

    #100;
    $finish;
  end
endmodule
