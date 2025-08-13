`ifndef CSPECT_SVH
`define CSPECT_SVH

import "DPI-C" function chandle cspect_new(
  input string trace_path,
  input real timescale,
  input int unsigned time_mult
);

import "DPI-C" function int cspect_finish(input chandle cspect_ctx);

import "DPI-C" function int cspect_flush(input chandle cspect_ctx);

import "DPI-C" function longint unsigned cspect_new_flow(input chandle cspect_ctx);

import "DPI-C" function longint unsigned cspect_new_track(
  input chandle cspect_ctx,
  input string name,
  input longint unsigned parent_uuid,
  input string description,
  input int child_ordering,
  input int child_order_rank
);

import "DPI-C" function int cspect_slice_begin(
  input chandle cspect_ctx,
  input longint unsigned parent_uuid,
  input real ts,
  input string name,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3,
  input bit replace_previous_slice
);

import "DPI-C" function int cspect_slice_end(
  input chandle cspect_ctx,
  input longint unsigned parent_uuid,
  input real ts,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3
);

import "DPI-C" function int cspect_instant_evt(
  input chandle cspect_ctx,
  input longint unsigned parent_uuid,
  input real ts,
  input string name,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3
);

import "DPI-C" function longint unsigned cspect_new_process(
  input chandle cspect_ctx,
  input int pid,
  input string process_name,
  input string cmdline,
  input int prio,
  input string description,
  input int child_ordering,
  input int child_order_rank
);

import "DPI-C" function longint unsigned cspect_new_thread(
  input chandle cspect_ctx,
  input int pid,
  input int tid,
  input string thread_name,
  input string description,
  input int child_ordering,
  input int child_order_rank
);

import "DPI-C" function longint unsigned cspect_new_counter(
  input chandle cspect_ctx,
  input string name,
  input string unit_name,
  input bit is_incremental,
  input longint unsigned parent_uuid,
  input string description,
  input int child_ordering,
  input int child_order_rank
);

import "DPI-C" function int cspect_int_counter_evt(
  input chandle cspect_ctx,
  input longint unsigned track_uuid,
  input real ts,
  input longint unsigned val
);

import "DPI-C" function int cspect_float_counter_evt(
  input chandle cspect_ctx,
  input longint unsigned track_uuid,
  input real ts,
  input real val
);

`endif  // CSPECT_SVH
