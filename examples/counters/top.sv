`timescale 1ns / 1ns

module top;
  import cspect_pkg::*;

  ctx     cspect;

  track   parent;

  counter basic_counter;
  counter basic_counter2;

  counter total_operations;
  counter cumulative_energy;

  counter execution_time;
  counter instruction_count;
  counter temperature_sensor;
  counter bandwidth_kbps;

  counter packet_count;
  counter throughput_mbps;
  counter error_rate;
  counter processing_time_us;

  initial begin
    // Initialize cspect context
    cspect = new("trace_counters.pftrace");

    // We will create all counters as children of a shared
    // parent track to specify an ordering - but that is not required.
    // You can also create counters without any parents (cspect.new_counter(..))
    // or as children to any track.
    parent = cspect.new_track("Counters Demo", .child_ordering(Chronological));

    // ========================================
    // 1. BASIC COUNTERS - Simple logging
    // ========================================

    // A basic counter can track numeric values over time:

    // Create a basic counter:
    basic_counter = parent.new_counter("My Counter");

    // Optional description:
    basic_counter2 = parent.new_counter(
        "My Other Counter",
        /* description: */
        .description("This is the other counter!")
    );

    basic_counter.log_int(0);
    basic_counter2.log_int(0);

    #10;
    basic_counter.log_int(42);
    basic_counter2.log_int(21);

    #10;
    basic_counter.log_float(3.14159);

    #10;
    basic_counter.log_int(42);
    basic_counter2.log_int(0);

    #30;

    // ========================================
    // 2. INCREMENTAL COUNTERS - Values accumulate
    // ========================================

    // Incremental counters track incremental changes to a running total:

    total_operations = parent.new_counter(
        "TotalOps",
        /* incremental counter: */
        .is_incremental(bit'(1)),
        /* description: */
        .description("Cumulative operation count")
    );
    cumulative_energy = parent.new_counter(
        "CumulativeEnergy",
        /* incremental counter: */
        .is_incremental(bit'(1)),
        /* description: */
        .description("Total energy consumed")
    );

    total_operations.log_int(5);  // Running total: 5
    cumulative_energy.log_float(1.2);  // Running total: 1.2

    #10;
    total_operations.log_int(3);  // Running total: 8 (5+3)
    cumulative_energy.log_int(3);  // Running total: 4.2 (1.2+3)

    #10;
    total_operations.log_int(-4);  // Running total: 4 (5+3-4)
    cumulative_energy.log_float(-0.2);  // Running total: 4 (1.2+3-0.2)

    #30;

    // ========================================
    // 3. UNIT EXAMPLES
    // ========================================

    // Counters can specify units.

    execution_time = parent.new_counter(
        "ExecTime",
        /* unit: */
        .unit_name("TimeNs"),
        /* description: */
        .description("Execution time in nanoseconds")
    );

    instruction_count = parent.new_counter(
        "Instructions",
        /* unit: */
        .unit_name("Count"),
        /* description: */
        .description("Number of instructions")
    );

    temperature_sensor = parent.new_counter(
        "Temperature",
        /* unit: */
        .unit_name("Celsius"),
        /* description: */
        .description("CPU Temperature")
    );

    execution_time.log_int(1500);  // 1500 ns
    instruction_count.log_int(250);  // 250 instructions
    temperature_sensor.log_float(45.5);  // 45.5°C

    #10;
    execution_time.log_int(2300);  // 2300 ns
    instruction_count.log_int(380);  // 380 instructions
    temperature_sensor.log_float(47.8);  // 47.8°C

    #30;

    // ========================================
    // 4 COMBINED EXAMPLE
    // ========================================

    packet_count = parent.new_counter(
        "PacketCount",
        /* unit: */
        .unit_name("Count"),
        /* incremental counter: */
        .is_incremental(bit'(1)),
        /* description: */
        .description("Total packets processed")
    );

    throughput_mbps = parent.new_counter(
        "Throughput",
        /* unit: */
        .unit_name("SizeBytes"),
        /* description: */
        .description("Throughput in MB/s")
    );

    error_rate = parent.new_counter(
        "ErrorRate",
        /* unit: */
        .unit_name("Percent"),
        /* description: */
        .description("Packet error rate")
    );

    processing_time_us = parent.new_counter(
        "ProcessingTime",
        /* unit: */
        .unit_name("TimeNs"),
        /* description: */
        .description("Processing time per packet (μs)")
    );

    for (int i = 0; i < 20; i++) begin
      #10;

      // Incremental packet count
      packet_count.log_int(longint'($urandom_range(10, 50)));

      // Varying throughput
      throughput_mbps.log_int(longint'($urandom_range(80, 120)));

      // Error rate as percentage
      error_rate.log_float($urandom_range(0, 500) / 100.0);

      // Processing latency
      processing_time_us.log_int(longint'($urandom_range(5, 25)));
    end

    $display("=== Demo Complete ===");

    // Clean up
    cspect.finish();
    $finish;
  end

endmodule
