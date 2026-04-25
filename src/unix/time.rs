use std::ptr::addr_of_mut;

#[inline]
pub fn now_ms() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_REALTIME, addr_of_mut!(ts));
    }
    (ts.tv_sec.cast_unsigned() * 1000) + (ts.tv_nsec.cast_unsigned() / 1_000_000)
}
