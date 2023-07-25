#![allow(unused)]

use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use super::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{alloc_frame, FrameTracker},
};

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

/// PageTable
/// frame: for RAII
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        // allocate a frame for root page table
        let frame = alloc_frame().expect("failed to allocate frame for root page table");

        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.fine_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn.0);

        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.fine_pte_create(vpn).unwrap();
        assert!(
            pte.is_valid(),
            "vpn {:?} is not mapped before unmapping",
            vpn.0
        );

        *pte = PageTableEntry::empty();
    }

    pub fn fine_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();

        // get root ppn
        let mut ppn = self.root_ppn;

        let mut res: Option<&mut PageTableEntry> = None;

        for i in 0..3 {
            let pte = ppn.get_pte_array().get_mut(idxs[i]).unwrap();

            // the last level
            if i == 2 {
                res = Some(pte);
                break;
            }
            // if is invalid, create a new page table
            if !pte.is_valid() {
                // allocate a frame for page table
                let frame = alloc_frame().expect("failed to allocate frame for page table");
                // set page table entry
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                // RAII
                self.frames.push(frame);
            }
        }

        res
    }

    // only for find page table entry
    // 用于手动查找页表项
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum(satp >> ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    // only for find page table entry
    // not create page table
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        // get root ppn
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTableEntry> = None;

        for i in 0..3 {
            let pte = ppn.get_pte_array().get_mut(idxs[i]).unwrap();

            // the last level
            if i == 2 {
                res = Some(pte);
                break;
            }
            // if is invalid, create a new page table
            if !pte.is_valid() {
                return None;
            }
        }

        res
    }

    // only for find page table entry
    // if found, return a cloned page table entry
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }
}
