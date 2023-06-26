#![no_std]
#![no_main]

use core::arch::global_asm;

mod console;
mod lang_items;
mod sbi;

global_asm!(include_str!("entry.s"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    sbi::shutdown();
    loop {}
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
