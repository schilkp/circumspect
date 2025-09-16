`ifndef CSPECT_DPI_SVH
`define CSPECT_DPI_SVH

// Replacement behavior constants for cspect_slice_begin
`define CSPECT_REPLACE_OFF 0
`define CSPECT_REPLACE 1
`define CSPECT_REPLACE_IF_DIFFERENT 2

import "DPI-C" function chandle cspect_uuid_vec_new(
  input longint unsigned uuid0,
  input longint unsigned uuid1,
  input longint unsigned uuid2,
  input longint unsigned uuid3
);

import "DPI-C" function int cspect_uuid_vec_append(
  input chandle uuid_vec,
  input longint unsigned uuid0,
  input longint unsigned uuid1,
  input longint unsigned uuid2,
  input longint unsigned uuid3
);

import "DPI-C" function int cspect_uuid_vec_delete(input chandle uuid_vec);

import "DPI-C" function chandle cspect_new(
  input string trace_path,
  input real timescale,
  input int unsigned time_mult
);

import "DPI-C" function int cspect_finish(input chandle cspect_ctx);

import "DPI-C" function int cspect_flush(input chandle cspect_ctx);

import "DPI-C" function longint unsigned cspect_new_uuid(input chandle cspect_ctx);

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
  input longint unsigned flow0,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3,
  input chandle flow_others,
  input longint unsigned flow_end0,
  input longint unsigned flow_end1,
  input longint unsigned flow_end2,
  input longint unsigned flow_end3,
  input chandle flow_end_others,
  input int replacement_behaviour,
  input longint unsigned correlation_id
);

import "DPI-C" function int cspect_slice_end(
  input chandle cspect_ctx,
  input longint unsigned parent_uuid,
  input real ts,
  input longint unsigned flow0,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3,
  input chandle flow_others,
  input longint unsigned flow_end0,
  input longint unsigned flow_end1,
  input longint unsigned flow_end2,
  input longint unsigned flow_end3,
  input chandle flow_end_others,
  input bit force_end,
  input longint unsigned correlation_id
);

import "DPI-C" function int cspect_instant_evt(
  input chandle cspect_ctx,
  input longint unsigned parent_uuid,
  input real ts,
  input string name,
  input longint unsigned flow0,
  input longint unsigned flow1,
  input longint unsigned flow2,
  input longint unsigned flow3,
  input chandle flow_others,
  input longint unsigned flow_end0,
  input longint unsigned flow_end1,
  input longint unsigned flow_end2,
  input longint unsigned flow_end3,
  input chandle flow_end_others,
  input longint unsigned correlation_id
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
  input longint unsigned val,
  input bit compress
);

import "DPI-C" function int cspect_float_counter_evt(
  input chandle cspect_ctx,
  input longint unsigned track_uuid,
  input real ts,
  input real val,
  input bit compress
);

`endif  // CSPECT_DPI_SVH
