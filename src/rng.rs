use std::os::raw::c_int;

use pyo3::ffi::{PyErr_SetString, PyExc_OSError, PyExc_ValueError};

#[cfg(unix)]
pub use crate::unix::time::now_ms;
#[cfg(windows)]
pub use crate::windows::time::{now_ms, platform_seeded};

#[cfg(unix)]
#[inline]
pub fn platform_seeded() {}

const C: u64 = 0xd07e_bc63_2746_54c7;
const MASK42: u64 = (1u64 << 42) - 1;
const MASK30: u64 = (1u64 << 30) - 1;
const MASK62: u64 = 0x3FFF_FFFF_FFFF_FFFF;

const V7_VERSION: u64 = 0x7000;
const V7_VARIANT: u64 = 0x8000_0000_0000_0000;

static mut W1_STATE: u64 = 0;
static mut W1_SEEDED: bool = false;
static mut LAST_MS: u64 = 0;
static mut COUNTER42: u64 = 0;

#[inline]
pub fn fill_random(buf: &mut [u8]) -> c_int {
    match getrandom::fill(buf) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[inline]
fn w1_mix(a: u64, b: u64) -> u64 {
    let t = u128::from(a) * u128::from(b);
    (t >> 64) as u64 ^ t as u64
}

#[inline]
fn w1rand() -> u64 {
    unsafe {
        W1_STATE = W1_STATE.wrapping_add(C);
        w1_mix(W1_STATE, W1_STATE ^ C)
    }
}

#[cold]
#[inline(never)]
fn seed_rng() -> c_int {
    let mut seed = [0u8; 16];
    if fill_random(&mut seed) != 0 {
        unsafe {
            PyErr_SetString(PyExc_OSError, c"unable to generate random bytes".as_ptr());
        }
        return -1;
    }
    let l = u64::from_ne_bytes(seed[..8].try_into().expect("8-byte seed chunk"));
    let r = u64::from_ne_bytes(seed[8..].try_into().expect("8-byte seed chunk"));
    unsafe {
        W1_STATE = l ^ w1_mix(r, r ^ C);
        W1_SEEDED = true;
    }
    platform_seeded();
    0
}

#[inline]
pub fn ensure_seeded() -> c_int {
    if unsafe { W1_SEEDED } { 0 } else { seed_rng() }
}

pub fn rnd_u64_secure() -> Result<u64, ()> {
    let mut buf = [0u8; 8];
    if fill_random(&mut buf) != 0 {
        unsafe {
            PyErr_SetString(PyExc_OSError, c"unable to generate random bytes".as_ptr());
        }
        return Err(());
    }
    Ok(u64::from_ne_bytes(buf))
}

pub fn reseed() {
    unsafe {
        W1_SEEDED = false;
        LAST_MS = 0;
        COUNTER42 = 0;
    }
}

#[inline]
fn advance_monotonic_with(
    observed_ms: u64,
    timestamp_ms: &mut u64,
    rand_a: &mut u16,
    tail62: &mut u64,
    mut rand: impl FnMut() -> Result<u64, ()>,
) -> c_int {
    let mut counter = unsafe { COUNTER42 };
    let mut current_ms = unsafe { LAST_MS };

    let r = match rand() {
        Ok(r) => r,
        Err(()) => return -1,
    };
    let low32 = r as u32;
    let increment = 1 + ((r >> 32) & 0x0f);

    if observed_ms > current_ms {
        current_ms = observed_ms;
        counter = match rand() {
            Ok(r) => r & MASK42,
            Err(()) => return -1,
        };
    } else {
        counter = counter.wrapping_add(increment);
        if counter > MASK42 {
            current_ms += 1;
            counter = match rand() {
                Ok(r) => r & MASK42,
                Err(()) => return -1,
            };
        }
    }

    unsafe {
        LAST_MS = current_ms;
        COUNTER42 = counter;
    }

    *timestamp_ms = current_ms;
    *rand_a = (counter >> 30) as u16;
    *tail62 = ((counter & MASK30) << 32) | u64::from(low32);
    0
}

#[inline]
pub fn advance_monotonic(
    observed_ms: u64,
    timestamp_ms: &mut u64,
    rand_a: &mut u16,
    tail62: &mut u64,
) {
    advance_monotonic_with(observed_ms, timestamp_ms, rand_a, tail62, || Ok(w1rand()));
}

pub fn advance_monotonic_secure(
    observed_ms: u64,
    timestamp_ms: &mut u64,
    rand_a: &mut u16,
    tail62: &mut u64,
) -> c_int {
    advance_monotonic_with(observed_ms, timestamp_ms, rand_a, tail62, rnd_u64_secure)
}

#[inline]
pub fn build_words(ts_ms: u64, rand_a: u16, tail62: u64) -> (u64, u64) {
    let hi = (ts_ms << 16) | V7_VERSION | u64::from(rand_a);
    let lo = V7_VARIANT | tail62;
    (hi, lo)
}

#[inline]
pub fn build_uuid7_default(hi: &mut u64, lo: &mut u64) -> c_int {
    if ensure_seeded() != 0 {
        return -1;
    }
    let (mut ts, mut ra, mut t62) = (0u64, 0u16, 0u64);
    advance_monotonic(now_ms(), &mut ts, &mut ra, &mut t62);
    let (h, l) = build_words(ts, ra, t62);
    *hi = h;
    *lo = l;
    0
}

pub fn build_uuid7_default_secure(hi: &mut u64, lo: &mut u64) -> c_int {
    if ensure_seeded() != 0 {
        return -1;
    }

    let (mut ts, mut ra, mut t62) = (0u64, 0u16, 0u64);
    if advance_monotonic_secure(now_ms(), &mut ts, &mut ra, &mut t62) != 0 {
        return -1;
    }

    let (h, l) = build_words(ts, ra, t62);
    *hi = h;
    *lo = l;
    0
}

#[inline]
fn extract_random_bits_with(
    has_ts: bool,
    has_nanos: bool,
    nanos: u64,
    rand_a: &mut u16,
    tail62: &mut u64,
    mut rand: impl FnMut() -> Result<u64, ()>,
) -> c_int {
    if has_ts && has_nanos {
        *rand_a = (nanos & 0x0FFF) as u16;
        return match rand() {
            Ok(r) => {
                *tail62 = r & MASK62;
                0
            }
            Err(()) => -1,
        };
    }
    if has_ts || has_nanos {
        let (c, r) = match (rand(), rand()) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return -1,
        };
        let counter = c & MASK42;
        *rand_a = (counter >> 30) as u16;
        *tail62 = ((counter & MASK30) << 32) | u64::from(r as u32);
        return 0;
    }
    1
}

#[inline]
fn extract_random_bits(
    has_ts: bool,
    has_nanos: bool,
    nanos: u64,
    rand_a: &mut u16,
    tail62: &mut u64,
) -> c_int {
    extract_random_bits_with(has_ts, has_nanos, nanos, rand_a, tail62, || Ok(w1rand()))
}

fn extract_random_bits_secure(
    has_ts: bool,
    has_nanos: bool,
    nanos: u64,
    rand_a: &mut u16,
    tail62: &mut u64,
) -> c_int {
    extract_random_bits_with(has_ts, has_nanos, nanos, rand_a, tail62, rnd_u64_secure)
}

#[inline]
pub fn build_uuid7_with_args(
    ts_ms: u64,
    has_ts: bool,
    nanos: u64,
    has_nanos: bool,
    hi: &mut u64,
    lo: &mut u64,
) -> c_int {
    if ensure_seeded() != 0 {
        return -1;
    }
    let (mut ra, mut t62) = (0u16, 0u64);
    let state = extract_random_bits(has_ts, has_nanos, nanos, &mut ra, &mut t62);

    let (h, l) = if state > 0 {
        let mut ms = ts_ms;
        advance_monotonic(ms, &mut ms, &mut ra, &mut t62);
        build_words(ms, ra, t62)
    } else {
        build_words(ts_ms, ra, t62)
    };
    *hi = h;
    *lo = l;
    0
}

pub fn build_uuid7_with_args_secure(
    ts_ms: u64,
    has_ts: bool,
    nanos: u64,
    has_nanos: bool,
    hi: &mut u64,
    lo: &mut u64,
) -> c_int {
    if ensure_seeded() != 0 {
        return -1;
    }

    let (mut ra, mut t62) = (0u16, 0u64);
    let state = extract_random_bits_secure(has_ts, has_nanos, nanos, &mut ra, &mut t62);

    if state < 0 {
        return -1;
    }

    let (h, l) = if state > 0 {
        let mut ms = ts_ms;
        if advance_monotonic_secure(ms, &mut ms, &mut ra, &mut t62) != 0 {
            return -1;
        }
        build_words(ms, ra, t62)
    } else {
        build_words(ts_ms, ra, t62)
    };

    *hi = h;
    *lo = l;
    0
}

#[inline]
pub fn build_timestamp_ms(ts_s: u64, has_ts: bool, nanos: u64, has_nanos: bool) -> Result<u64, ()> {
    const V7_MAX_TS_MS: u64 = 0xFFFF_FFFF_FFFF;
    const V7_MAX_TS_S: u64 = V7_MAX_TS_MS / 1000;

    if !has_ts {
        return Ok(now_ms());
    }

    if ts_s > V7_MAX_TS_S {
        unsafe {
            PyErr_SetString(PyExc_ValueError, c"timestamp is too large".as_ptr());
        }
        return Err(());
    }

    let mut ms = ts_s * 1000;

    if has_nanos {
        ms += nanos / 1_000_000;
    }

    if ms > V7_MAX_TS_MS {
        unsafe {
            PyErr_SetString(PyExc_ValueError, c"timestamp is too large".as_ptr());
        }
        return Err(());
    }
    Ok(ms)
}
