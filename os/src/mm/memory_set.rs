#![allow(unused)]
use crate::mm::address::PAGE_SIZE;

use super::{
    address::{PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::{self, alloc_frame, FrameTracker},
    page_table::{self, PTEFlags, PageTable},
};
use alloc::{collections::BTreeMap, vec::Vec};
use bitflags::*;

pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_permission: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_permission: MapPermission,
    ) -> Self {
        // at least contains the start_va to end_va
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();

        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_permission,
        }
    }
    /// Map a single page
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;

        match self.map_type {
            MapType::Identical => {
                // 恒等映射 直接将虚拟地址转换为物理地址
                ppn = PhysPageNum::from(vpn.0);
            }
            MapType::Framed => {
                let frame = alloc_frame().expect("failed to allocate frame");
                ppn = frame.ppn;

                // add the frame to data_frames
                self.data_frames.insert(vpn, frame);
            }
        }

        let pte_flags = PTEFlags::from_bits(self.map_permission.bits()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }

        // unmap from page_table
        page_table.unmap(vpn);
    }

    // map area to a page table
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    // unmap area from a page table
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    // copy data to the area
    // if the area is not a framed area, panic
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        assert!(
            data.len() <= self.data_frames.len() * PAGE_SIZE,
            "data is too large"
        );

        self.vpn_range
            .into_iter()
            .enumerate()
            .for_each(|(index, vpn)| {
                let src = &data[index * PAGE_SIZE..index * PAGE_SIZE + PAGE_SIZE];
                let dst =
                    &mut page_table.translate(vpn).unwrap().ppn().get_bytes_array()[..src.len()];

                dst.copy_from_slice(src);
            });
    }
}

/// MapType
/// Identical: The virtual address is the same as the physical address.
/// Framed: The virtual address is different from the physical address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8{
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        MemorySet {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);

        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }

        self.areas.push(map_area);
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        let map_area = MapArea::new(start_va, end_va, MapType::Framed, permission);
        self.push(map_area, None);
    }

    pub fn new_kernel() -> Self {
        panic!("unimplemented");
        Self::new_bare()
    }

    pub fn from_elf() -> (Self, usize, usize) {
        panic!("unimplemented");
        (Self::new_bare(), 0, 0)
    }
}
