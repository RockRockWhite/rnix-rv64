.section .text
.globl _start
_start:
    # init stack
    la sp, stack_top

loop:
    # stack frame
    addi sp, sp , -8
    sd fp, 0(sp)
    mv fp, sp


    addi sp, sp, -8
    sd ra, 0(sp)

    call func1

    ld ra, 0(sp)

    mv sp, fp
    ld fp, 0(sp)
    addi sp, sp, 8

    nop


    jal ra, loop

func1:
    addi sp, sp, -8
    sd ra, 0(sp)

    call func2

    ld ra, 0(sp)
    addi sp, sp, 8
    ret

func2:

    ret

.section .bss.stack
stack_bottom:
    .space 4096 * 16
stack_top:

