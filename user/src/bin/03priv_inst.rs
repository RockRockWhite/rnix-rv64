#![no_std]
#![no_main]

/// 执行特权指令
use core::arch::asm;

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");

    unsafe {
        asm!("mret");
    }
    0
}
