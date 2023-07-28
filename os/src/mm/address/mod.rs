#![allow(unused)]

mod phys;
mod range;
mod virt;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub use phys::*;
pub use range::*;
pub use virt::*;
