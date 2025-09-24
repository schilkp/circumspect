`include "cspect_dpi.svh"

`timescale 1ns / 1ns
package cspect_pkg;

  // Forward typedefs:
  typedef class track;
  typedef class counter;
  typedef class thread;

  typedef longint unsigned uuid_t;

  typedef enum int {
    Unknown = 0,
    Lexicographic = 1,
    Chronological = 2,
    Explicit = 3
  } child_ordering_e;

  typedef struct {
    uuid_t  uuid0;
    uuid_t  uuid1;
    uuid_t  uuid2;
    uuid_t  uuid3;
    chandle others;
  } __dpi_uuid_array_t;

  function automatic __dpi_uuid_array_t __dpi_uuid_vec(uuid_t uuids[]);
    automatic __dpi_uuid_array_t result;
    automatic chandle array = 0;

    if (uuids.size() > 4) begin
      for (int i = 4; i < uuids.size(); i += 4) begin
        if (array == null) begin
          array = cspect_dpi_uuid_vec_new(
              uuids.size() > i + 0 ? uuids[i+0] : 0,
              uuids.size() > i + 1 ? uuids[i+1] : 0,
              uuids.size() > i + 2 ? uuids[i+2] : 0,
              uuids.size() > i + 3 ? uuids[i+3] : 0
          );
          if (array == null) begin
            $error("cspect: cspect_dpi_uuid_vec_new failed.");
          end
        end else begin
          automatic int err = 0;
          err = cspect_dpi_uuid_vec_append(
              array,
              uuids.size() > i + 0 ? uuids[i+0] : 0,
              uuids.size() > i + 1 ? uuids[i+1] : 0,
              uuids.size() > i + 2 ? uuids[i+2] : 0,
              uuids.size() > i + 3 ? uuids[i+3] : 0
          );
          if (err != 0) begin
            $error("cspect: cspect_dpi_uuid_vec_append failed with error code %0d.", err);
          end
        end
      end
    end

    result.uuid0  = uuids.size() > 0 ? uuids[0] : 0;
    result.uuid1  = uuids.size() > 1 ? uuids[1] : 0;
    result.uuid2  = uuids.size() > 2 ? uuids[2] : 0;
    result.uuid3  = uuids.size() > 3 ? uuids[3] : 0;
    result.others = array;
    return result;
  endfunction

  function automatic void __dpi_uuid_vec_delete(__dpi_uuid_array_t vec);
    if (vec.others != 0) begin
      automatic int result = cspect_dpi_uuid_vec_delete(vec.others);
      if (result != 0) begin
        $error("cspect: cspect_dpi_uuid_vec_delete failed with error code %0d.", result);
      end
    end
  endfunction

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

    function uuid_t new_uuid();
      automatic uuid_t uuid = cspect_dpi_new_uuid(ctx_chandle);
      if (uuid == null) begin
        $error("cspect: cspect_dpi_new_uuid failed.");
      end
      return uuid;
    endfunction

    function track new_track(string name, string description = "",
                             child_ordering_e child_ordering = Unknown, int child_order_rank = 0);
      track new_track;
      uuid_t uuid = cspect_dpi_new_track(
          ctx_chandle, name, this.scope_uuid, description, child_ordering, child_order_rank
      );
      if (uuid == null) begin
        $error("cspect: cspect_dpi_new_track failed for track '%s'.", name);
        return null;
      end
      new_track = new(this.ctx_chandle, uuid);
      return new_track;
    endfunction

    function counter new_counter(string name, string unit_name = "", bit is_incremental = 0,
                                 string description = "", child_ordering_e child_ordering = Unknown,
                                 int child_order_rank = 0);
      counter new_counter;
      uuid_t uuid = cspect_dpi_new_counter(
          ctx_chandle,
          name,
          unit_name,
          is_incremental,
          this.scope_uuid,
          description,
          child_ordering,
          child_order_rank
      );
      if (uuid == null) begin
        $error("cspect: cspect_dpi_new_counter failed for counter '%s'.", name);
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

    function void slice_begin(string name, uuid_t flows[] = {}, uuid_t flows_end[] = {},
                              uuid_t correlation_id = 0);
      automatic int result;
      automatic __dpi_uuid_array_t dpi_flows, dpi_flows_end;
      dpi_flows = __dpi_uuid_vec(flows);
      dpi_flows_end = __dpi_uuid_vec(flows_end);
      result = cspect_dpi_slice_begin(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          name,
          dpi_flows.uuid0,
          dpi_flows.uuid1,
          dpi_flows.uuid2,
          dpi_flows.uuid3,
          dpi_flows.others,
          dpi_flows_end.uuid0,
          dpi_flows_end.uuid1,
          dpi_flows_end.uuid2,
          dpi_flows_end.uuid3,
          dpi_flows_end.others,
          `CSPECT_REPLACE_OFF,
          correlation_id
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_slice_begin failed for slice '%s' with error code %0d.", name,
               result);
      end
      __dpi_uuid_vec_delete(dpi_flows);
      __dpi_uuid_vec_delete(dpi_flows_end);
    endfunction

    function void slice_set(string name, uuid_t flows[] = {}, uuid_t flows_end[] = {},
                            bit compress = 0, uuid_t correlation_id = 0);
      automatic int result;
      automatic int replacement_behaviour;
      automatic __dpi_uuid_array_t dpi_flows, dpi_flows_end;
      dpi_flows = __dpi_uuid_vec(flows);
      dpi_flows_end = __dpi_uuid_vec(flows_end);
      replacement_behaviour = compress ? `CSPECT_REPLACE_IF_DIFFERENT : `CSPECT_REPLACE;
      result = cspect_dpi_slice_begin(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          name,
          dpi_flows.uuid0,
          dpi_flows.uuid1,
          dpi_flows.uuid2,
          dpi_flows.uuid3,
          dpi_flows.others,
          dpi_flows_end.uuid0,
          dpi_flows_end.uuid1,
          dpi_flows_end.uuid2,
          dpi_flows_end.uuid3,
          dpi_flows_end.others,
          replacement_behaviour,
          correlation_id
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_slice_begin failed for slice '%s' with error code %0d.", name,
               result);
      end
      __dpi_uuid_vec_delete(dpi_flows);
      __dpi_uuid_vec_delete(dpi_flows_end);
    endfunction

    function void slice_end(uuid_t flows[] = {}, uuid_t flows_end[] = {}, bit force_end = 0,
                            uuid_t correlation_id = 0);
      automatic int result;
      automatic __dpi_uuid_array_t dpi_flows, dpi_flows_end;
      dpi_flows = __dpi_uuid_vec(flows);
      dpi_flows_end = __dpi_uuid_vec(flows_end);
      result = cspect_dpi_slice_end(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          dpi_flows.uuid0,
          dpi_flows.uuid1,
          dpi_flows.uuid2,
          dpi_flows.uuid3,
          dpi_flows.others,
          dpi_flows_end.uuid0,
          dpi_flows_end.uuid1,
          dpi_flows_end.uuid2,
          dpi_flows_end.uuid3,
          dpi_flows_end.others,
          force_end,
          correlation_id
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_slice_end failed with error code %0d.", result);
      end
      __dpi_uuid_vec_delete(dpi_flows);
      __dpi_uuid_vec_delete(dpi_flows_end);
    endfunction

    function void instant_evt(string name, uuid_t flows[] = {}, uuid_t flows_end[] = {},
                              uuid_t correlation_id = 0);
      automatic int result;
      automatic __dpi_uuid_array_t dpi_flows, dpi_flows_end;
      dpi_flows = __dpi_uuid_vec(flows);
      dpi_flows_end = __dpi_uuid_vec(flows_end);
      result = cspect_dpi_instant_evt(
          this.ctx_chandle,
          this.scope_uuid,
          $realtime,
          name,
          dpi_flows.uuid0,
          dpi_flows.uuid1,
          dpi_flows.uuid2,
          dpi_flows.uuid3,
          dpi_flows.others,
          dpi_flows_end.uuid0,
          dpi_flows_end.uuid1,
          dpi_flows_end.uuid2,
          dpi_flows_end.uuid3,
          dpi_flows_end.others,
          correlation_id
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_instant_evt failed for event '%s' with error code %0d.", name,
               result);
      end
      __dpi_uuid_vec_delete(dpi_flows);
      __dpi_uuid_vec_delete(dpi_flows_end);
    endfunction
  endclass

  class counter extends cspect_ctx_chandle;
    uuid_t counter_uuid;

    function new(chandle handle, uuid_t uuid);
      super.new(handle);
      counter_uuid = uuid;
    endfunction

    function void log_int(longint unsigned val, bit compress = 0);
      automatic
      int
      result = cspect_dpi_int_counter_evt(
          this.ctx_chandle, this.counter_uuid, $realtime, val, compress
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_int_counter_evt failed with error code %0d.", result);
      end
    endfunction

    function void log_float(real val, bit compress = 0);
      automatic
      int
      result = cspect_dpi_float_counter_evt(
          this.ctx_chandle, this.counter_uuid, $realtime, val, compress
      );
      if (result != 0) begin
        $error("cspect: cspect_dpi_float_counter_evt failed with error code %0d.", result);
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
      uuid_t uuid = cspect_dpi_new_thread(
          ctx_chandle, pid, tid, thread_name, description, child_ordering, child_order_rank
      );
      if (uuid == null) begin
        $error("cspect: cspect_dpi_new_thread failed for thread '%s'.", thread_name);
        return null;
      end
      new_thread = new(this.ctx_chandle, uuid);
      return new_thread;
    endfunction
  endclass

  class thread extends track;
    function new(chandle handle, uuid_t uuid);
      super.new(handle, uuid);
    endfunction
  endclass

  class ctx extends scope;
    function new(string trace_path, int unsigned time_mult = 1);
      super.new(0, 0);
      this.ctx_chandle = cspect_dpi_new(trace_path, 0.000000001, time_mult);
      if (this.ctx_chandle == null) begin
        $error("cspect:  cspect_dpi_new failed.");
      end
    endfunction

    function void finish();
      automatic int result;
      result = cspect_dpi_finish(this.ctx_chandle);
      if (result != 0) begin
        $error("cspect: cspect_dpi_finish failed with error code %0d.", result);
      end
      this.ctx_chandle = null;
    endfunction

    function void flush();
      automatic int result = cspect_dpi_flush(this.ctx_chandle);
      if (result != 0) begin
        $error("cspect: cspect_dpi_flush failed with error code %0d.", result);
      end
    endfunction

    function process new_process(int pid, string process_name, string cmdline = "", int prio = 0,
                                 string description = "", child_ordering_e child_ordering = Unknown,
                                 int child_order_rank = 0);
      process new_process;
      uuid_t uuid = cspect_dpi_new_process(
          ctx_chandle,
          pid,
          process_name,
          cmdline,
          prio,
          description,
          child_ordering,
          child_order_rank
      );
      if (uuid == null) begin
        $error("cspect: cspect_dpi_new_process failed for process '%s'.", process_name);
        return null;
      end
      new_process = new(this.ctx_chandle, uuid, pid);
      return new_process;
    endfunction

  endclass
endpackage
