.altmacro
.macro SAVE_GP n
    sd x\n, \n * 8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n * 8(sp)
.endm

.section .text.trampoline
.globl __alltraps
.globl __restore
# 2^2次方对齐，即4字节对齐
.align 2
__alltraps:
    # must confirm that sscratch: *TrapContext in user space(Constant); 
    # CSR read and write
    # sscratch -> sp
    # sp -> sscratch
    # switch(sp, sscratch)
    csrrw sp, sscratch, sp

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

    # load kernel satp
    ld t0, 34 * 8(sp)
    # load kernel_sp
    ld t1, 35 * 8(sp)
    # load trap_handler
    ld t2, 35 * 8(sp)

    # switch to kernel stack
    ld sp, t1
    # switch to kernel satp
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t2

__restore:
    # args:
    # a0: *TrapContext in user space(Constant); also the top of user stack
    # a1: user space token

    # switch to user space
    csrw satp, a1
    sfence.vma

    # save *TrapContext to sscratch
    csrw sscratch, a0
    # switch to stack that *TrapContext in
    ld sp, a0

    # recv CSR
    ld t0, 32 * 8(sp)
    ld t1, 33 * 8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # recv general-purpose reg
    ld x1, 1 * 8(sp)
    ld x3, 3 * 8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n + 1
    .endr

    # switch to user stack (recv x2)
    ld sp, 2 * 8(sp)

    sret


