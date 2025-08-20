`ifndef CSPECT_MACROS_SVH
`define CSPECT_MACROS_SVH


`ifndef CSPECT_MACROS_CLK
`define CSPECT_MACROS_CLK clk_i
`endif  // CSPECT_UTIL_CLK

`ifndef CSPECT_MACROS_NRST
`define CSPECT_MACROS_NRST rst_ni
`endif  // CSPECT_UTIL_NRST

// verilog_format: off

// Generate an instant event if a given condition is true at the rising clock
// edge.
//
// __track: Track on which to generate the slice
// __cond: Condition which triggers the event
// __name: Instant event message (optional.)
// __en: Enable condition (optional.)
// (__clk: clock input)
// (__arst_n: asynchronous reset, active-low)
`define CSPECT_EVT_FF(__track, __cond, __name = "", __en = 1, __clk = `CSPECT_MACROS_CLK,
                      __arst_n = `CSPECT_MACROS_NRST) \
  always_ff @(posedge (__clk)) begin                              \
    if (__arst_n) begin                                           \
      if (__en) begin                                             \
        if (__cond) begin                                         \
          __track.instant_evt(__name);                            \
        end                                                       \
      end                                                         \
    end                                                           \
  end

// Generates a slice if a given condition is true at the rising clock edge.
// Consecutive slice events are combined if their __name is identical.
//
// __track: Track on which to generate the slice.
// __cond: Condition which triggers the slice.
// __name: Slice event message (optional.)
// __en: Enable condition (optional.)
// (__clk: clock input)
// (__arst_n: asynchronous reset, active-low)
`define CSPECT_SLICE_FF(__track, __cond, __name = "", __en = 1, __clk = `CSPECT_MACROS_CLK,
                      __arst_n = `CSPECT_MACROS_NRST) \
  always_ff @(posedge (__clk)) begin                              \
    if (__arst_n) begin                                           \
      if (__en) begin                                             \
        if (__cond) begin                                         \
          __track.slice_set(__name, .compress(1));                \
        end else begin                                            \
          __track.slice_end();                                    \
        end                                                       \
      end                                                         \
    end                                                           \
  end

// verilog_format: on

`endif  // CSPECT_MACROS_SVH
