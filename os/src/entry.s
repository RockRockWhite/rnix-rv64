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

.section .text
.globl test_call
test_call:
    addi sp, sp, -8
    sd ra, 0(sp)

    # li t0 , 0
    
    # auipc t0, %pcrel_hi(call_func)

    # jal ra, %pcrel_lo(call_func)

    # jalr ra, t0, %pcrel_lo(call_func)
    # jalr ra, t0, %pcrel_lo(call_func)
    # jalr ra, t0, %pcrel_lo(call_func)

    # jalr ra, call_func
    # call call_func
    la t0, call_func
    jalr ra, t0, 0


    ld ra, 0(sp)
    addi sp, sp, 8
    ret

call_func:
    li x0, 0x1234
    ret