ENTRY(_start)

MEMORY {
  MEM (rx)  : ORIGIN = 0x10000000, LENGTH = 16M
}

SECTIONS {
  /* Keep binaries lean */
  /DISCARD/ : { *(.riscv.attributes) *(.comment) }
}

SECTIONS {
  .text.fixed_main_func 0x20001230 : {
    *(.text.fixed_main_func)
  }

  .text.fixed_foobar_func 0x3000BEEF : {
    *(.text.fixed_foobar_func)
  }

  .text : {
    *(.text._start)
    *(.text)
    *(.text.*)
  } > MEM

  .misc : ALIGN(16) {
    *(.rodata)
    *(.rodata.*)
    *(.data)
    *(.data.*)
    *(.srodata)
    *(.srodata.*)
    *(.sdata)
    *(.sdata.*)
  } > MEM

  .bss : ALIGN(16) {
    __bss_start = .;
    *(.bss)
    *(.bss.*)
    *(.sbss)
    *(.sbss.*)
    __bss_end = .;
  } > MEM


  . = ALIGN(4);
  __stack_pointer$  = ORIGIN(MEM) + LENGTH(MEM) - 8;
  __global_pointer$ = ORIGIN(MEM) + LENGTH(MEM) / 2;
}
