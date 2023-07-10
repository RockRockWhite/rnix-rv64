#![no_std]
#![no_main]

use user_lib::*;

const WIDTH: usize = 10;
const HEIGHT: usize = 20000;

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        for _ in 0..WIDTH {
            print!("A");
        }
        println!("[{}/{}]", i + 1, HEIGHT);
    }
    println!("Test write_a OK!");
    0
}
