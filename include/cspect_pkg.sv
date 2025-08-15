`include "cspect_dpi.svh"

`timescale 1ns / 1ns
package cspect_pkg;

  typedef longint unsigned uuid_t;

  typedef enum int {
    Unknown = 0,
    Lexicographic = 1,
    Chronological = 2,
    Explicit = 3
  } child_ordering_e;

  class cspect_ctx_chandle;
    chandle ctx_chandle;

    function new(chandle handle);
      ctx_chandle = handle;
    endfunction
  endclass

  class scope extends cspect_ctx_chandle;
    uuid_t scope_uuid;

    function new(chandle handle, uuid_t uuid);
      super.new(handle);
      scope_uuid = uuid;
    endfunction

    function uuid_t new_flow();
      uuid_t uuid = cspect_new_flow(ctx_chandle);
      if (uuid == 0) begin
        $error("cspect: cspect_new_flow failed");
      end
      return uuid;
    endfunction

    function track new_track(string name, string description = "",
                             child_ordering_e child_ordering = Unknown, int child_order_rank = 0);
      track new_track;
      uuid_t uuid = cspect_new_track(
          ctx_chandle, name, this.scope_uuid, description, child_ordering, child_order_rank
      );
      if (uuid == 0) begin
        $error("cspect: cspect_new_track failed for track '%s'", name);
        return null;
      end
      new_track = new(this.ctx_chandle, uuid);
      return new_track;
    endfunction

    function counter new_counter(string name, string unit_name = "", bit is_incremental = 0,
                                 string description = "", child_ordering_e child_ordering = Unknown,
                                 int child_order_rank = 0);
      counter new_counter;
      uuid_t uuid = cspect_new_counter(
          ctx_chandle,
          name,
          unit_name,
          is_incremental,
          this.scope_uuid,
          description,
          child_ordering,
          child_order_rank
      );
      if (uuid == 0) begin
        $error("cspect: cspect_new_counter failed for counter '%s'", name);
        return null;
      end
      new_counter = new(this.ctx_chandle, uuid);
      return new_counter;
    endfunction


  endclass

  class track extends scope;
    function new(chandle handle, uuid_t uuid);
      super.new(handle, uuid);
    endfunction

    function void slice_begin(string name, uuid_t flow1 = 0, uuid_t flow2 = 0, uuid_t flow3 = 0);
      int result = cspect_slice_begin(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          name,
          flow1,
          flow2,
          flow3,
          `CSPECT_REPLACE_OFF
      );
      if (result != 0) begin
        $error("cspect: cspect_slice_begin failed for slice '%s' with error code %0d", name,
               result);
      end
    endfunction

    function void slice_set(string name, uuid_t flow1 = 0, uuid_t flow2 = 0, uuid_t flow3 = 0,
                            bit compress = 0);
      int replacement_behaviour = compress ? `CSPECT_REPLACE_IF_DIFFERENT : `CSPECT_REPLACE;
      int result = cspect_slice_begin(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          name,
          flow1,
          flow2,
          flow3,
          replacement_behaviour
      );
      if (result != 0) begin
        $error("cspect: cspect_slice_begin failed for slice '%s' with error code %0d", name,
               result);
      end
    endfunction

    function void slice_end(uuid_t flow1 = 0, uuid_t flow2 = 0, uuid_t flow3 = 0);
      int result = cspect_slice_end(
          this.ctx_chandle, this.scope_uuid, $realtime, flow1, flow2, flow3
      );
      if (result != 0) begin
        $error("cspect: cspect_slice_end failed with error code %0d", result);
      end
    endfunction

    function void instant_evt(string name, uuid_t flow1 = 0, uuid_t flow2 = 0, uuid_t flow3 = 0);
      int result = cspect_instant_evt(
          this.ctx_chandle, this.scope_uuid, $realtime, name, flow1, flow2, flow3
      );
      if (result != 0) begin
        $error("cspect: cspect_instant_evt failed for event '%s' with error code %0d", name,
               result);
      end
    endfunction
  endclass

  class counter extends cspect_ctx_chandle;
    uuid_t counter_uuid;

    function new(chandle handle, uuid_t uuid);
      super.new(handle);
      counter_uuid = uuid;
    endfunction

    function void log_int(longint unsigned val, bit compress = 0);
      int result = cspect_int_counter_evt(
          this.ctx_chandle, this.counter_uuid, $realtime, val, compress
      );
      if (result != 0) begin
        $error("cspect: cspect_int_counter_evt failed with error code %0d", result);
      end
    endfunction

    function void log_float(real val, bit compress = 0);
      int result = cspect_float_counter_evt(
          this.ctx_chandle, this.counter_uuid, $realtime, val, compress
      );
      if (result != 0) begin
        $error("cspect: cspect_float_counter_evt failed with error code %0d", result);
      end
    endfunction
  endclass

  class process extends track;
    int pid;

    function new(chandle handle, uuid_t uuid, int process_id);
      super.new(handle, uuid);
      pid = process_id;
    endfunction

    function thread new_thread(int tid, string thread_name, string description = "",
                               child_ordering_e child_ordering = Unknown, int child_order_rank = 0);
      thread new_thread;
      uuid_t uuid = cspect_new_thread(
          ctx_chandle, pid, tid, thread_name, description, child_ordering, child_order_rank
      );
      if (uuid == 0) begin
        $error("cspect: cspect_new_thread failed for thread '%s'", thread_name);
        return null;
      end
      new_thread = new(this.ctx_chandle, uuid, this.pid, tid);
      return new_thread;
    endfunction
  endclass

  class thread extends track;
    int pid;
    int tid;
    function new(chandle handle, uuid_t uuid, int process_id, int thread_id);
      super.new(handle, uuid);
      pid = process_id;
      tid = thread_id;
    endfunction
  endclass

  class ctx extends scope;
    function new(string trace_path, int unsigned time_mult = 1);
      super.new(0, 0);
      this.ctx_chandle = cspect_new(trace_path, 0.000000001, time_mult);
      if (this.ctx_chandle == null) begin
        $error("cspect: Failed to create cspect context");
      end
    endfunction

    function void finish();
      int result;
      result = cspect_finish(this.ctx_chandle);
      if (result != 0) begin
        $error("cspect: cspect_finish failed with error code %0d", result);
      end
      this.ctx_chandle = null;
    endfunction

    function void flush();
      int result = cspect_flush(this.ctx_chandle);
      if (result != 0) begin
        $error("cspect: cspect_flush failed with error code %0d", result);
      end
    endfunction

    function process new_process(int pid, string process_name, string cmdline = "", int prio = 0,
                                 string description = "", child_ordering_e child_ordering = Unknown,
                                 int child_order_rank = 0);
      process new_process;
      uuid_t uuid = cspect_new_process(
          ctx_chandle,
          pid,
          process_name,
          cmdline,
          prio,
          description,
          child_ordering,
          child_order_rank
      );
      if (uuid == 0) begin
        $error("cspect: cspect_new_process failed for process '%s'", process_name);
        return null;
      end
      new_process = new(this.ctx_chandle, uuid, pid);
      return new_process;
    endfunction

  endclass
endpackage
