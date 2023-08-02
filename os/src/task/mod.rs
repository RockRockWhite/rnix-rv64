#![allow(unused)]

use self::{switch::__switch, task::TaskControlBlock};
use crate::{
    loader, println,
    task::{context::TaskContext, task::TaskStatus},
    trap::context::TrapContext,
};
use alloc::vec::Vec;
use core::{cell::RefCell, usize};
use lazy_static::*;

pub mod context;
mod switch;
mod task;

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

unsafe impl Sync for TaskManager {}

impl TaskManager {
    fn get_current_token(&self) -> usize {
        let mut inner: core::cell::RefMut<'_, TaskManagerInner> = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].get_user_token()
    }

    fn get_current_ctx(&self) -> &mut TrapContext {
        let mut inner: core::cell::RefMut<'_, TaskManagerInner> = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].get_trap_ctx()
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].task_status = TaskStatus::Exited;
    }

    fn mark_current_suspended(&self) {
        let mut inner: core::cell::RefMut<'_, TaskManagerInner> = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].task_status = TaskStatus::Ready;
    }

    fn run_next_task(&self) {
        if let Some(next_task_id) = self.find_next_task_id() {
            let current_task_ctx_ptr;
            let next_task_ctx_ptr;
            {
                let mut inner = self.inner.borrow_mut();

                // get current task ctx
                let curr_task_id = inner.current_task;
                let mut current_task = inner.tasks.get_mut(curr_task_id).unwrap();
                current_task_ctx_ptr = &mut current_task.task_ctx as *mut TaskContext;

                // mark task to run's status runnnig
                // get task to run ctx
                let mut task_to_run = inner.tasks.get_mut(next_task_id).unwrap();
                task_to_run.task_status = TaskStatus::Running;
                next_task_ctx_ptr = &task_to_run.task_ctx as *const TaskContext;

                // update current id
                inner.current_task = next_task_id;
            }

            // switch to next task
            unsafe {
                __switch(current_task_ctx_ptr, next_task_ctx_ptr);
            }
        } else {
            panic!("[Kernel] All applications completed!");
        }
    }

    fn find_next_task_id(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current_task_id = inner.current_task;

        // 从当前任务往后，找到第一个ready的任务
        // current + 1 开始
        // current + 1 + self.num_app 结束
        (current_task_id + 1..self.num_app + current_task_id + 1)
            .map(|id| id % self.num_app)
            .find(|&id| inner.tasks[id].task_status == TaskStatus::Ready)
    }

    fn run_first_task(&self) {
        let first_task_ctx_ptr;
        {
            let mut inner = self.inner.borrow_mut();

            // mark running and get ctx ptr2
            let mut first_task = inner.tasks.get_mut(0).unwrap();
            first_task.task_status = TaskStatus::Running;
            first_task_ctx_ptr = &first_task.task_ctx as *const TaskContext;
        }

        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut _, first_task_ctx_ptr);
        }
        unreachable!()
    }
}

lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        println!("[kernel] init task manager");

        let num_app = loader::get_num_app();
        println!("num_app = {}", num_app);

        let mut tasks: Vec<TaskControlBlock> = Vec::new();

        (0..num_app).for_each(|id| {
            tasks.push(TaskControlBlock::new(loader::get_app_data(id), id));
        });

        println!("test point");

        TaskManager {
            num_app,
            inner: RefCell::new({
                TaskManagerInner {
                    tasks,
                    current_task: 0,
                }
            }),
        }
    };
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_ctx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_ctx()
}
