#![allow(unused)]

use crate::trap::context::TrapContext;
use crate::{println, sbi};
use core::task::Context;
use core::{arch::asm, cell::RefCell};
use lazy_static::*;
use riscv::register::mcause::Trap;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.0.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    /// push context to kernel stack
    pub fn push_context(&self, ctx: TrapContext) -> &'static mut TrapContext {
        // 分配空间给 TrapContext
        let ctx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        // 填入context数据
        unsafe {
            *ctx_ptr = ctx;
        }

        unsafe { ctx_ptr.as_mut().unwrap() }
    }
}

#[repr(align(4096))]
struct UserStack([u8; USER_STACK_SIZE]);

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.0.as_ptr() as usize + USER_STACK_SIZE
    }
}

static KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);
static USER_STACK: UserStack = UserStack([0; USER_STACK_SIZE]);

struct AppManager {
    inner: RefCell<AppManagerInner>,
}

#[derive(Clone, Copy)]
struct AppInfo {
    pub start: usize,
    pub end: usize,
}

struct AppManagerInner {
    num_app: usize,
    current_app: usize,
    app_infos: [AppInfo; MAX_APP_NUM],
}

unsafe impl Sync for AppManager {}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.inner.borrow().num_app);

        for i in 0..self.inner.borrow().num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.inner.borrow().app_infos[i].start,
                self.inner.borrow().app_infos[i].end
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.inner.borrow().current_app
    }

    pub fn move_to_next_app(&self) {
        self.inner.borrow_mut().current_app += 1;
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.inner.borrow().num_app {
            println!("All applications completed!");
            sbi::shutdown();
        }

        println!(
            "[kernel] load app_{} [{:#x}, {:#x})",
            app_id,
            self.inner.borrow().app_infos[app_id].start,
            self.inner.borrow().app_infos[app_id].end
        );

        // clear app space
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0);
        });

        // load app
        let app_info = &self.inner.borrow().app_infos[app_id];
        let app_src =
            core::slice::from_raw_parts(app_info.start as *const u8, app_info.end - app_info.start);
        let app_dest = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());

        app_dest.copy_from_slice(&app_src);

        // clear icache
        unsafe { asm!("fence.i") };
    }
}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager {
        inner: RefCell::new({
            extern "C" {
                fn _num_app();
            }

            let num_app = unsafe { (_num_app as *const usize).read_volatile() };

            // read app_info_raw
            let app_info_raw: &[usize] = unsafe {
                core::slice::from_raw_parts(
                    (_num_app as *const usize).add(1),
                    num_app + 1,
                )
            };

            // generate app_infos
            let mut app_infos: [AppInfo; MAX_APP_NUM] = [AppInfo {
                start: 0,
                end: 0,
            }; MAX_APP_NUM];

            app_info_raw.windows(2).enumerate().for_each(|(i, window)| {
                app_infos[i].start = window[0];
                app_infos[i].end = window[1];
            });

            AppManagerInner {
                num_app,
                current_app: 0,
                app_infos,
            }
        })
    };
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.print_app_info();
}

pub fn run_next_app() -> ! {
    let curr_app = APP_MANAGER.get_current_app();
    unsafe { APP_MANAGER.load_app(curr_app) };
    APP_MANAGER.move_to_next_app();

    extern "C" {
        fn __restore(ctx_addr: usize);
    }

    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::from_app_info(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }

    unreachable!()
}
