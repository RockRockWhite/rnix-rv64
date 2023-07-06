#![allow(unused)]

use self::{switch::__switch, task::TaskControlBlock};
use crate::{loader, task::task::TaskStatus};
use core::{cell::RefCell, usize};
use lazy_static::*;

pub mod context;
mod switch;
mod task;

pub const MAX_APP_NUM: usize = 16;
struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

unsafe impl Sync for TaskManager {}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let current = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let current = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current].task_status = TaskStatus::Exited;
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            // mark ready
            inner.tasks[next].task_status = TaskStatus::Running;
            // mark current task
            inner.current_task = next;

            // get task context
            let current_task_cx_ptr2 = inner.tasks[current].get_task_ctx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_ctx_ptr2();

            core::mem::drop(inner);

            unsafe {
                __switch(current_task_cx_ptr2, next_task_cx_ptr2);
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;

        // 从当前任务往后，找到第一个ready的任务
        // current + 1 开始
        // current + 1 + self.num_app 结束
        (current + 1..self.num_app + current + 1)
            .map(|id| id % self.num_app)
            .find(|&id| inner.tasks[id].task_status == TaskStatus::Ready)
    }

    fn run_first_task(&self) {
        let mut inner = self.inner.borrow_mut();
        let mut first_task = inner.tasks[0];

        // set running
        first_task.task_status = TaskStatus::Running;

        core::mem::drop(inner);

        let _unused: usize = 0;
        unsafe {
            __switch(&_unused as *const _, first_task.get_task_ctx_ptr2());
        }
        unreachable!()
    }
}

lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        let num_app = loader::get_num_app();

        let mut tasks = [TaskControlBlock {
            task_ctx_ptr: 0,
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];

        (0..num_app).for_each(|id| {
            tasks[id].task_ctx_ptr = loader::init_app_ctx(id);
            tasks[id].task_status = TaskStatus::Ready;
        });

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

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
