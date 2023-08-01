use crate::{
    config,
    mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE},
    trap::{context::TrapContext, trap_handler},
};

use super::context::TaskContext;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub task_ctx_ptr: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub trap_ctx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn get_trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_ctx_ppn = memory_set
            .translate(VirtAddr::from(config::TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // map a kernel-stack in kernel space

        let (kernel_stack_bottom, kernel_stack_top) = config::kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_ctx_ptr =
            (kernel_stack_top - core::mem::size_of::<TaskContext>()) as *mut TaskContext;

        unsafe {
            *task_ctx_ptr = TaskContext::goto_trap_return();
        }

        let tcb = TaskControlBlock {
            task_ctx_ptr: task_ctx_ptr as usize,
            task_status: TaskStatus::Ready,
            memory_set,
            trap_ctx_ppn,
            base_size: user_sp,
        };

        // prepare TrapContext in user space
        let trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );

        tcb
    }

    pub fn get_task_ctx_ptr2(&self) -> *const usize {
        &self.task_ctx_ptr as *const usize
    }
}
