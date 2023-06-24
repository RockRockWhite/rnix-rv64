.section .text.entry
.globl _start
_start:
    li x1, 100
    la sp, boot_stack_top
    call rust_main

.section .bss.stack
.globl boot_stack_bottom
.globl boot_stack_top

boot_stack_bottom:
    .space 4096 * 16
boot_stack_top: