#include <stdio.h>

// Include both headers to detect inconsistent prototypes.
// Multiple (compatible) prototypes are allowed in C, but mismatching
// prototypes cause GCC to act-up.
#include "dpi_hdr_cbindgen.h"
#include "dpi_hdr_verilator.h"

int main() { printf("OK!\r\n"); }
