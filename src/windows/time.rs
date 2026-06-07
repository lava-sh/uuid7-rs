use std::{
    ptr::addr_of_mut,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::windows::win_api::{FILETIME, GetSystemTimePreciseAsFileTime, QueryInterruptTime};

static EPOCH_OFFSET_MS: AtomicU64 = AtomicU64::new(0);

// 369 years between 1601 and 1970, including 89 leap years
// = (369 * 365 + 89) days = 134_774 days
// * 86_400 secs = 11_644_473_600 secs
// * 10_000_000 ticks = 116_444_736_000_000_000
const UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
const TICKS_PER_MILLISECOND: u64 = 10_000;

#[inline]
fn system_ms() -> u64 {
    let mut file_time = FILETIME::default();
    unsafe {
        GetSystemTimePreciseAsFileTime(addr_of_mut!(file_time));
    }

    let ticks = unsafe { *(&raw const file_time).cast::<u64>() };

    (ticks - UNIX_EPOCH_TICKS) / TICKS_PER_MILLISECOND
}

#[inline]
fn now() -> u64 {
    let mut interrupt_time = 0;
    unsafe {
        QueryInterruptTime(addr_of_mut!(interrupt_time));
    }
    interrupt_time / TICKS_PER_MILLISECOND
}

#[inline]
pub fn now_ms() -> u64 {
    now() + EPOCH_OFFSET_MS.load(Ordering::Relaxed)
}

pub fn platform_seeded() {
    EPOCH_OFFSET_MS.store(system_ms() - now(), Ordering::Relaxed);
}
