use core::arch::global_asm;

global_asm!(include_str!("switch.s"));

extern "C" {
    pub fn __switch(current_task_ctx_ptr2: *const usize, next_task_ctx_ptr2: *const usize);
}
