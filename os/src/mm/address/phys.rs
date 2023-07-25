use riscv::addr::Page;

use crate::mm::page_table::PageTableEntry;

use super::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    /// page_offset
    /// 从物理地址获得页内偏移
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// floor
    /// 从物理地址获得页号
    /// 向下取整
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    /// ceil
    /// 从物理地址获得页号
    /// 向上取整
    /// 如果页内偏移不为0，页号加1
    /// 如果页内偏移为0，页号不变
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE + (self.page_offset() != 0) as usize)
    }
}

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }

}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysPageNum> for usize {
    fn from(value: PhysPageNum) -> Self {
        value.0
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        assert_eq!(value.page_offset(), 0);

        value.floor()
    }
}
