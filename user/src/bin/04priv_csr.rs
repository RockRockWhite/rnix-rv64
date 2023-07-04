#![no_std]
#![no_main]

/// 修改CSR寄存器
use riscv::register::sstatus::{self, SPP};
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        sstatus::set_spp(SPP::User);
    }
    0
}
