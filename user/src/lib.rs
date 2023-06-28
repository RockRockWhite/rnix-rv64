#![no_std]
#![allow(unused)]
#![feature(linkage)]
#![feature(panic_info_message)]

pub mod console;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    unreachable!();
}

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    // clear bss segment
    (sbss as usize..ebss as usize)
        .into_iter()
        .for_each(|a| unsafe {
            (a as *mut u8).write_volatile(0);
        });
}

#[no_mangle]
#[linkage = "weak"]
// main 弱符号
fn main() -> i32 {
    panic!("Not main!");
}

pub fn write(fs: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fs, buf)
}

pub fn exit(code: i32) -> isize {
    syscall::sys_exit(code)
}
