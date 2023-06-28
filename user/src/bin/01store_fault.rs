#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe {
        // 向不正确的位置写入数据
        core::ptr::null_mut::<u8>().write_volatile(0);
    }
    0
}
