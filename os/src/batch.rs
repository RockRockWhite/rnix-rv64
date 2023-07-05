#![allow(unused)]

use crate::{loader, println, sbi};
use core::{arch::asm, cell::RefCell};
use lazy_static::*;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

struct AppManager {
    inner: RefCell<AppManagerInner>,
}

struct AppManagerInner {
    num_app: usize,
    current_app: usize,
}

unsafe impl Sync for AppManager {}

impl AppManager {
    pub fn get_current_app(&self) -> usize {
        self.inner.borrow().current_app
    }

    pub fn move_to_next_app(&self) {
        self.inner.borrow_mut().current_app += 1;
        if self.inner.borrow().current_app > self.inner.borrow().num_app {
            println!("All applications completed!");
            sbi::shutdown();
        }
    }
}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager {
        inner: RefCell::new({
            extern "C" {
                fn _num_app();
            }

            let num_app = unsafe { (_num_app as *const usize).read_volatile() };

            AppManagerInner {
                num_app,
                current_app: 0,
            }
        })
    };
}

pub fn run_next_app() -> ! {
    let curr_app = APP_MANAGER.get_current_app();
    APP_MANAGER.move_to_next_app();

    extern "C" {
        fn __restore(ctx_addr: usize);
    }

    let ctx = loader::init_app_ctx(curr_app);
    unsafe {
        __restore(ctx);
    }

    unreachable!()
}
