.altmacro
.macro SAVE_GP n
    sd x\n, \n * 8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n * 8(sp)
.endm

.section .text
.globl __alltraps
.globl __restore
# 2^2次方对齐，即4字节对齐
.align 2
__alltraps:
    # CSR read and write
    # sscratch -> sp
    # sp -> sscratch
    # switch(sp, sscratch)
    csrrw sp, sscratch, sp

    # allocate space for trap context
    addi sp, sp, -34 * 8

    # save registers
    # skip x0, x2, x4
    sd x1, 1 * 8(sp)
    sd x3, 3 * 8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # save csr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32 * 8(sp)
    sd t1, 33 * 8(sp)

    # save user stack sp
    csrr t2, sscratch
    sd t2, 2 * 8(sp)

    mv a0, sp
    call trap_handler

    # trap_handler ret
    # continue

__restore:
    # recv CSR
    ld t0, 32 * 8(sp)
    ld t1, 33 * 8(sp)
    ld t2, 2 * 8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # recv general-purpose reg
    ld x1, 1 * 8(sp)
    ld x3, 3 * 8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n + 1
    .endr

    # free stack
    addi sp, sp, 34 * 8

    # switch to user stack
    csrrw sp, sscratch, sp

    sret


