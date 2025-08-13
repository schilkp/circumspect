`timescale 1ns / 1ns

module top;
  import cspect_pkg::*;

  ctx cspect;
  process main_process;
  process worker_process;
  thread main_thread;
  thread worker_thread1;
  thread worker_thread2;

  counter main_process_counter;

  initial begin
    // Initialize the cspect context
    cspect = new("trace_processes.pftrace");

    // + "MainProcess"        (process - pid: 100)
    // |   + "MainThread"     (thread  - tid: 1001)
    // |
    // + "WorkerProcess"      (process - pid: 101)
    //     + "WorkerThread1"  (thread  - tid: 2001)
    //     + "WorkerThread2"  (thread  - tid: 2002)

    // Create processes
    main_process = cspect.new_process(
        /* pid: */
        100,
        /* name: */
        .process_name("MainProcess"),
        /* cmdline (optional): */
        .cmdline("/usr/bin/main"),
        /* priority (optional): */
        .prio(0),
        /* description (optional): */
        .description("Main application process")
    );

    worker_process = cspect.new_process(
        /* pid: */
        101,
        /* name: */
        .process_name("WorkerProcess"),
        /* cmdline (optional): */
        .cmdline("/usr/bin/worker"),
        /* priority (optional): */
        .prio(1),
        /* description (optional): */
        .description("Background worker process")
    );

    // Create threads within processes
    main_thread = main_process.new_thread(
        /* tid: */
        1001,
        /* name: */
        .thread_name("MainThread"),
        /* description (optional): */
        .description("Primary execution thread")
    );

    worker_thread1 = worker_process.new_thread(
        /* tid: */
        2001,
        /* name: */
        .thread_name("WorkerThread1"),
        /* description (optional): */
        .description("First worker thread")
    );

    worker_thread2 = worker_process.new_thread(
        /* tid: */
        2002,
        /* name: */
        .thread_name("WorkerThread2"),
        /* description (optional): */
        .description("Second worker thread")
    );

    // Create counter scoped to MainProcess
    main_process_counter = main_process.new_counter(
        /* name: */
        .name("tasks_processed"),
        /* unit_name (optional): */
        .unit_name("tasks"),
        /* is_incremental (optional): */
        .is_incremental(1),
        /* description (optional): */
        .description("Number of tasks processed")
    );

    // Generate process/thread trace events
    #5;
    main_thread.slice_begin("initialization");

    #10;
    worker_thread1.slice_begin("task_processing");
    worker_thread2.slice_begin("data_loading");
    main_process.slice_begin("startup_phase");

    #5 main_process.slice_end();

    #15;
    main_thread.instant_evt("config_loaded");
    main_thread.slice_end();  // end initialization

    #20;
    main_thread.slice_begin("main_loop");
    worker_thread1.instant_evt("task_received");
    main_process_counter.log_int(1);

    #25;
    worker_thread2.slice_end();  // end data_loading
    worker_thread2.slice_begin("data_validation");

    #30;
    worker_thread1.slice_end();  // end task_processing
    worker_thread1.slice_begin("result_reporting");
    main_process_counter.log_int(2);

    #15;
    main_thread.instant_evt("status_check");
    worker_thread2.instant_evt("validation_complete");

    #20;
    worker_thread1.slice_end();  // end result_reporting
    worker_thread2.slice_end();  // end data_validation

    #10;
    main_thread.slice_end();  // end main_loop
    main_thread.instant_evt("shutdown");

    // Finish
    cspect.finish();
    $finish;
  end

endmodule

