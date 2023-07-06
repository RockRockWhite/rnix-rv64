use crate::{println, task};

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    task::exit_current_and_run_next();
    unreachable!()
}

pub fn sys_yield() -> isize {
    task::suspend_current_and_run_next();
    0
}
