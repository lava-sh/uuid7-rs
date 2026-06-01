use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn now_ms() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
}

#[inline]
pub fn platform_seeded() {}
