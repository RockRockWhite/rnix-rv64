use crate::{println, syscall::syscall, task};
use context::TrapContext;
use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception},
    stval, stvec,
    utvec::TrapMode,
};

pub mod context;

global_asm!(include_str!("trap.s"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    unsafe { stvec::write(__alltraps as usize, TrapMode::Direct) }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        scause::Trap::Exception(Exception::UserEnvCall) => {
            // sret to the next instrustion
            ctx.sepc += 4;
            // distribute trap
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        scause::Trap::Exception(Exception::StoreFault)
        | scause::Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            task::exit_current_and_run_next();
        }
        scause::Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            task::exit_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }

    ctx
}
