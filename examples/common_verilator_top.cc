#include "Vtop.h"
#include "verilated.h"
#include "verilated_fst_c.h"
#include <csignal>
#include <cstdlib>
#include <iostream>
#include <ostream>

volatile bool interrupt_signal_received = false;
void signalHandler(int signum) { interrupt_signal_received = true; }

int main(int argc, char **argv) {

  signal(SIGINT, signalHandler);

  std::cout << "//===----------------------------------------------------------------------===//"
            << std::endl;
  std::cout << "//                           Verilator Runner" << std::endl;
  std::cout << "//===----------------------------------------------------------------------===//"
            << std::endl;

  // Setup context, defaults, and parse command line
  Verilated::debug(0);
  const std::unique_ptr<VerilatedContext> contextp{new VerilatedContext};
  contextp->commandArgs(argc, argv);
  contextp->assertOn(true);

  bool do_trace = std::getenv("VERILATOR_TRACE") != NULL;
  contextp->traceEverOn(do_trace);

  // Construct the Verilated model, from Vtop.h generated from Verilating
  const std::unique_ptr<Vtop> topp{new Vtop{contextp.get(), ""}};

  VerilatedFstC *tfp = nullptr;
  if (do_trace) {
    std::cout << "top: Tracing - ON" << std::endl;
    tfp = new VerilatedFstC;
    topp->trace(tfp, 99);  // Trace 99 levels of hierarchy
    tfp->open("dump.fst"); // Open fst file
  } else {
    std::cout << "top: Tracing - OFF" << std::endl;
  }

  // Simulate until $finish
  while (!contextp->gotFinish()) {
    topp->eval();
    if (do_trace) tfp->dump(contextp->time());

    if (topp->eventsPending()) {
      contextp->time(topp->nextTimeSlot());
    } else {
      contextp->timeInc(1);
    }

    if (interrupt_signal_received) {
      std::cout << std::endl;
      std::cout << std::endl;
      std::cout << "top: Received SIGINT!" << std::endl;
      break;
    }
  }

  // Execute 'final' processes
  topp->final();

  if (do_trace) tfp->close();

  std::cout << "//===----------------------------------------------------------------------===//"
            << std::endl;

  // Print statistical summary report
  contextp->statsPrintSummary();

  return (contextp->gotError() || interrupt_signal_received) ? 1 : 0;
}
