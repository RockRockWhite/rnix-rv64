#![allow(unused)]
use crate::{config, mm::address::PhysAddr, sync::UPSafeCell};

use super::address::PhysPageNum;
use alloc::vec::Vec;
use lazy_static::*;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        // first try to allocate from recycled
        if let Some(ppn) = self.recycled.pop() {
            return Some(ppn.into());
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        // check valid ppn
        if self.current <= ppn.0 && self.recycled.contains(&ppn.into()) {
            panic!("dealloc invalid ppn");
        }

        // push into recycled
        self.recycled.push(ppn.into());
    }
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

// RAII
impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes = ppn.get_bytes_array();
        for each in bytes.iter_mut() {
            *each = 0;
        }

        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc_frame(self.ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        UPSafeCell::new(StackFrameAllocator::new());
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(config::MEMORY_END).floor(),
    );
}

pub fn alloc_frame() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

pub fn dealloc_frame(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}
