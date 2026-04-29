#[inline]
pub fn now_ms() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_REALTIME, std::ptr::addr_of_mut!(ts));
    }
    #[cfg(target_pointer_width = "64")]
    {
        (ts.tv_sec.cast_unsigned() * 1000) + (ts.tv_nsec.cast_unsigned() / 1_000_000)
    }
    #[cfg(target_pointer_width = "32")]
    {
        (ts.tv_sec.cast_unsigned() as u64 * 1000) + (ts.tv_nsec.cast_unsigned() as u64 / 1_000_000)
    }
}

#[inline]
pub fn platform_seeded() {}
