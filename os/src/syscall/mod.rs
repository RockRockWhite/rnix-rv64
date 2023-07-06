use self::{
    fs::sys_write,
    process::{sys_exit, sys_yield},
};

mod fs;
mod process;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_WRITE => sys_write(args[0] as usize, args[1] as *const u8, args[2] as usize),
        SYSCALL_YIELD => sys_yield(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
