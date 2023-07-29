#![allow(unused)]
use super::{
    address::{PhysAddr, PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::{self, alloc_frame, FrameTracker},
    page_table::{self, PTEFlags, PageTable},
};
use crate::{
    config::{MEMORY_END, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    mm::address::PAGE_SIZE,
    println,
    sync::UPSafeCell,
};
use _core::{arch::asm, mem};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use bitflags::*;
use lazy_static::*;
use riscv::register::satp;

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

    fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }

        // 将.text中的strampoline映射到虚拟地址开头处
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        )
    }

    pub fn new_kernel() -> Self {
        // 内核的地址空间
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn sdata();
            fn edata();
            fn sbss_with_stack();
            fn ebss();
            fn ekernel();
            fn strampoline();
        }
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        memory_set
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();

        // map trampoline
        memory_set.map_trampoline();

        // map program headers of elf
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;

        // check magic
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46]);

        let ph_cnt = elf_header.pt2.ph_count(); // program header count
        let mut max_end_vpn = VirtPageNum(0);

        (0..ph_cnt).for_each(|i| {
            let ph = elf.program_header(i).unwrap();

            // only need loadable segments
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = VirtAddr::from(ph.virtual_addr() as usize);
                let end_va = VirtAddr::from(ph.virtual_addr() as usize + ph.mem_size() as usize);

                // read permission
                let mut permission = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    permission |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    permission |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    permission |= MapPermission::X;
                }

                let area = MapArea::new(start_va, end_va, MapType::Framed, permission);
                max_end_vpn = area.vpn_range.get_end();

                memory_set.push(
                    area,
                    Some(
                        elf.input
                            .get(ph.offset() as usize..(ph.offset() + ph.file_size()) as usize)
                            .unwrap(),
                    ),
                );
            }
        });

        // map user stack
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // map trap context
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    // load satp csr
    // flush tlb
    pub fn activate(&self) {
        let satp = self.page_table.token();

        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(UPSafeCell::new(MemorySet::new_kernel()));
}

pub fn remap_test() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss_with_stack();
        fn ebss();
        fn ekernel();
        fn strampoline();
    }
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);
    println!("[kernel] remap_test passed!");
}
