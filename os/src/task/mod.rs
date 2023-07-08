#![allow(unused)]

use self::{switch::switch, task::TaskControlBlock};
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
    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].task_status = TaskStatus::Exited;
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].task_status = TaskStatus::Ready;
    }

    fn run_next_task(&self) {
        if let Some(next_task_id) = self.find_next_task_id() {
            let current_task_ctx_ptr2;
            let next_task_ctx_ptr2;
            {
                let mut inner = self.inner.borrow_mut();

                // get current task ctx
                let curr_task_id = inner.current_task;
                let mut current_task = inner.tasks.get_mut(curr_task_id).unwrap();
                current_task_ctx_ptr2 = current_task.get_task_ctx_ptr2();

                // mark task to run's status runnnig
                // get task to run ctx
                let mut task_to_run = inner.tasks.get_mut(next_task_id).unwrap();
                task_to_run.task_status = TaskStatus::Running;
                next_task_ctx_ptr2 = task_to_run.get_task_ctx_ptr2();

                // update current id
                inner.current_task = next_task_id;
            }

            // switch to next task
            unsafe {
                switch(current_task_ctx_ptr2, next_task_ctx_ptr2);
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
        let first_task_ctx_ptr2;
        {
            let mut inner = self.inner.borrow_mut();

            // mark running and get ctx ptr2
            let mut first_task = inner.tasks.get_mut(0).unwrap();
            first_task.task_status = TaskStatus::Running;
            first_task_ctx_ptr2 = first_task.get_task_ctx_ptr2();
        }

        let _unused: usize = 0;
        unsafe {
            switch(&_unused as *const _, first_task_ctx_ptr2);
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
