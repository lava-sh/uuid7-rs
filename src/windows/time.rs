use std::ptr::addr_of_mut;

use crate::windows::win_api::{FILETIME, GetSystemTimePreciseAsFileTime, QueryInterruptTime};

static mut EPOCH_BASE_MS: u64 = 0;
static mut TICK_BASE_MS: u64 = 0;

#[inline]
fn system_ms() -> u64 {
    let mut file_time = FILETIME::default();
    unsafe {
        GetSystemTimePreciseAsFileTime(addr_of_mut!(file_time));
    }
    let ticks = (u64::from(file_time.dwHighDateTime) << 32) | u64::from(file_time.dwLowDateTime);
    (ticks - 116_444_736_000_000_000) / 10_000
}

#[inline]
fn now() -> u64 {
    let mut interrupt_time = 0u64;
    unsafe {
        QueryInterruptTime(addr_of_mut!(interrupt_time));
    }
    interrupt_time / 10_000
}

#[inline]
pub fn now_ms() -> u64 {
    let now = now();
    unsafe { EPOCH_BASE_MS + now - TICK_BASE_MS }
}

pub fn platform_seeded() {
    unsafe {
        EPOCH_BASE_MS = system_ms();
        TICK_BASE_MS = now();
    }
}
