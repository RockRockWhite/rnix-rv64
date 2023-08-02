use crate::{
    config::{TRAMPOLINE, TRAP_CONTEXT},
    println,
    syscall::syscall,
    task::{self, current_trap_ctx, current_user_token},
    timer,
};
use core::arch::{asm, global_asm};
use riscv::register::{
    scause::{self, Exception},
    sie, stval, stvec,
    utvec::TrapMode,
};

pub mod context;

global_asm!(include_str!("trap.s"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let ctx = current_trap_ctx();
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        scause::Trap::Interrupt(scause::Interrupt::SupervisorTimer) => {
            // 设置下一次时钟中断
            timer::set_next_trigger();
            // 进行调度
            task::suspend_current_and_run_next();
        }

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

    trap_return();
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

/// enable_timer_interrupt
/// 初始化时钟中断
/// 设置屏蔽位， 允许supervisor timer中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
