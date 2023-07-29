mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use self::memory_set::KERNEL_SPACE;
pub use heap_allocator::*;
pub use memory_set::remap_test;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    // 刷新 TLB
    KERNEL_SPACE.exclusive_access().activate();
}
