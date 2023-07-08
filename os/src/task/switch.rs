use core::arch::global_asm;

global_asm!(include_str!("switch.s"));

extern "C" {
    fn __switch(current_task_ctx_ptr2: *const usize, next_task_ctx_ptr2: *const usize);
}

pub unsafe fn switch(current_task_ctx_ptr2: *const usize, next_task_ctx_ptr2: *const usize) {
    __switch(current_task_ctx_ptr2, next_task_ctx_ptr2)
}
