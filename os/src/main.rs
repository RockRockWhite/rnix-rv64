#![no_std]
#![no_main]

use core::arch::global_asm;

mod console;
mod lang_items;
mod sbi;

global_asm!(include_str!("entry.s"));

#[no_mangle]
pub fn rust_main() -> ! {
    println!("Hello, world!");
    sbi::shutdown();
    loop {}
}
