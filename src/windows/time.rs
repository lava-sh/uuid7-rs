use std::{
    ptr::addr_of_mut,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::windows::win_api::{FILETIME, GetSystemTimePreciseAsFileTime, QueryInterruptTime};

static EPOCH_OFFSET_MS: AtomicU64 = AtomicU64::new(0);

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
    let mut interrupt_time = 0_u64;
    unsafe {
        QueryInterruptTime(addr_of_mut!(interrupt_time));
    }
    interrupt_time / 10_000
}

#[inline]
pub fn now_ms() -> u64 {
    now() + EPOCH_OFFSET_MS.load(Ordering::Relaxed)
}

pub fn platform_seeded() {
    EPOCH_OFFSET_MS.store(system_ms() - now(), Ordering::Relaxed);
}
