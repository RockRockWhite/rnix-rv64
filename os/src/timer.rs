#![allow(unused)]

use crate::{boards, sbi};
use riscv::register::time;

const SBI_SET_TIMER: usize = 0;
// 时间片数量
const TICKS_PER_SEC: usize = 100;

pub fn get_time() -> usize {
    time::read()
}

pub fn set_next_trigger() {
    sbi::set_timer(get_time() + boards::CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn get_time_ms() -> usize {
    get_time() / (boards::CLOCK_FREQ / 1000)
}
