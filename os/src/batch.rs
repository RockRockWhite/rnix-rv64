#![allow(unused)]

use crate::println;
use core::{arch::asm, cell::RefCell};
use lazy_static::*;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

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

    pub fn move_to_next_app(&mut self) {
        self.inner.borrow_mut().current_app += 1;
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= MAX_APP_NUM {
            panic!("illegal app_id");
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

pub fn test() {
    unsafe { APP_MANAGER.load_app(1) };
    println!("app_1 loaded");
}
