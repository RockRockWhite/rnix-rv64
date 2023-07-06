#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    for _ in 0..5 {
        println!("yield...");
        yield_();
    }
    println!("Hello, world!");
    0
}
