#![allow(unused)]

use bitflags::*;

use super::address::PhysPageNum;

bitflags! {
    /// Page Table Entry Flags
    /// V: Valid
    /// R: Readable
    /// W: Writable
    /// X: Executable
    /// U: User 允许用户态访问
    /// G: Global
    /// A: Accessed 被访问过记录1
    /// D: Dirty 被修改过记录1
    pub struct PTEFlags: u8{
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

/// Page Table Entry
/// 页表项
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.bits >> 10)
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits as u8)
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}
