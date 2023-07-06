use core::arch::asm;

use crate::{
    println,
    task::{self, context::TaskContext},
    trap::context::TrapContext,
};

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 2;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.0.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    /// push context to kernel stack
    pub fn push_context(
        &self,
        trap_ctx: TrapContext,
        task_ctx: TaskContext,
    ) -> &'static mut TaskContext {
        unsafe {
            // 分配空间给 TrapContext
            let trap_ctx_ptr =
                (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
            // 填入数据
            *trap_ctx_ptr = trap_ctx;

            // 分配空间给 TaskContext
            let task_ctx_ptr =
                (trap_ctx_ptr as usize - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
            // 填入数据
            *task_ctx_ptr = task_ctx;

            task_ctx_ptr.as_mut().unwrap()
        }
    }
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack([u8; USER_STACK_SIZE]);

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.0.as_ptr() as usize + USER_STACK_SIZE
    }
}

static KERNEL_STACK: [KernelStack; task::MAX_APP_NUM] =
    [KernelStack([0; KERNEL_STACK_SIZE]); task::MAX_APP_NUM];
static USER_STACK: [UserStack; task::MAX_APP_NUM] =
    [UserStack([0; USER_STACK_SIZE]); task::MAX_APP_NUM];

/// load apps
/// load all user apps into memory
pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }

    // read app_num
    let num_app = unsafe { (_num_app as *const usize).read_volatile() };

    // read app_info_raw
    let app_info_raw: &[usize] =
        unsafe { core::slice::from_raw_parts((_num_app as *const usize).add(1), num_app + 1) };

    // load
    app_info_raw.windows(2).enumerate().for_each(|(i, window)| {
        let base_addr = get_base_address(i);

        // clear app space
        (base_addr..base_addr + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0);
        });

        // load current app
        let app_start = window[0];
        let app_end = window[1];

        let app_src =
            unsafe { core::slice::from_raw_parts(app_start as *const u8, app_end - app_start) };
        let app_dest =
            unsafe { core::slice::from_raw_parts_mut(base_addr as *mut u8, app_src.len()) };
        app_dest.copy_from_slice(&app_src);

        println!(
            "[kernel] loaded app_{} [{:#x}, {:#x}) to {:#x}",
            i, app_start, app_end, base_addr
        );
    });

    // clear i-cache
    unsafe { asm!("fence.i") };
}

fn get_base_address(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn init_app_ctx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(
        TrapContext::from_app_info(get_base_address(app_id), USER_STACK[app_id].get_sp()),
        TaskContext::goto_restore(),
    ) as *const _ as usize
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }

    // read app_num
    unsafe { (_num_app as *const usize).read_volatile() }
}
