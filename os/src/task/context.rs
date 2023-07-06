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
    pub fn goto_restore() -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            s: [0; 12],
        }
    }
}
