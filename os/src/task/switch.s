.altmacro
.macro SAVE_SN n
    sd s\n, (\n + 1) * 8(sp)
.endm

.macro LOAD_SN n
    ld s\n, (\n + 1) * 8(sp)
.endm

.section .text
.global __switch

__switch:
    # push TaskContext to current sp and save its address to where a0 points to
    # args:
    # current_task_ctx_ptr: ptr<*const TaskContext>   &*const TaskContext,
    # next_task_ctx_ptr: &*const TaskContext
    #
    # pub struct TaskContext 
    #   ra: usize,
    #   s:  usize *  12
    # 

    # allocate space for current_task_ctx
    add sp, sp, -13 * 8

    # save sp to current_task_ctx_ptr
    sd sp, 0(a0)

    # save current task context
    # save ra
    sd ra, 0 * 8(sp)
    # save s0-s11
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr

    # switch to next task
    ld sp, 0(a1)

    # rec registers
    ld ra, 0 * 8(sp)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr

    # pop context
    add sp, sp, 13 * 8
    ret
