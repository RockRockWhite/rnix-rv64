use crate::trap::trap_return;

#[allow(unused)]
#[repr(C)]
/// TaskContext
/// args:
///     ra  用于保存ret位置
///     s   s0-s11寄存器是callee-saved寄存器，由于switch相当于一个函数调用，因此只需保存callee-saved寄存器
pub struct TaskContext {
    ra: usize,
    s: [usize; 12],
}

impl TaskContext {
    /// init task context
    /// set Task Context{__restore ASM funciton: trap_return, sp: kstack_ptr, s: s_0..12}
    pub fn goto_restore() -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            s: [0; 12],
        }
    }

    pub fn goto_trap_return() -> Self {
        Self {
            ra: trap_return as usize,
            s: [0; 12],
        }
    }
}
