#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;
use core::arch::global_asm;

mod boards;
pub mod config;
pub mod console;
mod lang_items;
mod loader;
pub mod mm;
mod sbi;
pub mod sync;
mod syscall;
mod task;
mod timer;
pub mod trap;

global_asm!(include_str!("entry.s"));
global_asm!(include_str!("link_app.s"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");

    extern "C" {
        fn add_two(a: usize, b: usize) -> usize;
    }
    println!("add_two(1, 2) = {}", unsafe { add_two(12, 13) });

    mm::init_heap();

    trap::init();
    loader::load_apps();

    // enable timer interrupt
    trap::enable_timer_interrupt();

    // init timer
    timer::set_next_trigger();
    task::run_first_task();
    sbi::shutdown()
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

    println!("the .bss segment inited.")
}
