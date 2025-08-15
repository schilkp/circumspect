# THIS IS NOT A COMPLETE CRT0: JUST ENOUGH TO COMPILE, BUT WILL NOT WORK!
.section .text._start
.global _start
_start:
    call main

.global _exit
.align 4
_exit:
    j _exit
