use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX_EPOCH")
        .as_millis() as u64
}

#[inline]
pub fn platform_seeded() {}
