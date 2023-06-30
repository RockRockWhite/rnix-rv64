.section .text
.globl _start
_start:
    # init stack
    la sp, stack_top

    # init trap from U mode
    la t0, __alltraps
    csrw stvec, t0

    # 切换到用户态程序
    # 手动构造中断上下文
    # stack context
    # sp
    # sstatus
    # sepc
    addi sp, sp, -4 * 8

    csrr t0, sstatus
    la t1, entry
    la t2, user_stack_top

    sd t0, 2 * 8(sp)
    sd t1, 3 * 8(sp)
    sd t2, 1 * 8(sp)

    mv a0, sp

    call __restore

    # li x17, 0x123

entry:
    li x0, 0x123
    ecall



trap_handler:
    ret

# stack context
# sp
# sstatus
# sepc
.align 2
__alltraps:
    # 保存中断上下文

    # 切换到内核栈
    csrrw sp, sscratch, sp

    # 分配栈空间
    addi sp, sp, -4 * 8

    # 保存csr寄存器
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 2 * 8(sp)
    sd t1, 3 * 8(sp)

    # 保存sp
    csrr t2, sscratch
    sd t2, 1 * 8(sp)

    # args
    mv a0, sp
    call trap_handler

    # 函数执行完，会回来

# 恢复寄存器
# args：
#   a0: 指向中断上下文的指针
__restore:
    # 恢复保存完trap上下文时的sp
    mv sp, a0

    # 恢复trap上下文
    # 恢复csr寄存器
    ld t0, 2 * 8(sp)
    ld t1, 3 * 8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复sp
    ld t2, 1 * 8(sp)
    csrw sscratch, t2

    # 回收内核栈空间
    addi sp, sp, 4 * 8

    # 切换回用户栈
    csrrw sp, sscratch, sp

    # 回到U模式
    sret

# loop:
#     # stack frame
#     addi sp, sp , -8
#     sd fp, 0(sp)
#     mv fp, sp


#     addi sp, sp, -8
#     sd ra, 0(sp)

#     call func1

#     ld ra, 0(sp)

#     mv sp, fp
#     ld fp, 0(sp)
#     addi sp, sp, 8

#     nop


#     jal ra, loop

# func1:
#     addi sp, sp, -8
#     sd ra, 0(sp)

#     call func2

#     ld ra, 0(sp)
#     addi sp, sp, 8
#     ret

# func2:

#     ret

.section .bss.stack
stack_bottom:
    .space 4096 * 16
stack_top:

user_stack_bottom:
    .space 4096 * 16
user_stack_top:

