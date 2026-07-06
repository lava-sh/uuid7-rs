use std::{
    cell::RefCell,
    ffi::c_int,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use rand::{
    Rng, TryRng,
    rngs::{StdRng as ChaCha12Rng, SysRng},
};

use crate::python::exceptions::{PyOSError, PyValueError};
#[cfg(unix)]
pub use crate::unix::time::{now_ms, platform_seeded};
#[cfg(windows)]
pub use crate::windows::time::{now_ms, platform_seeded};

const C: u64 = 0xd07e_bc63_2746_54c7;
const MASK42: u64 = (1 << 42) - 1;
const MASK30: u64 = (1 << 30) - 1;
const MASK62: u64 = (1 << 62) - 1;

static W1_STATE: AtomicU64 = AtomicU64::new(0);
static W1_SEEDED: AtomicBool = AtomicBool::new(false);
static LAST_MS: AtomicU64 = AtomicU64::new(0);
static COUNTER42: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static RNG: RefCell<ChaCha12Rng> = RefCell::new(rand::make_rng());
}

#[inline]
pub fn fill_random(buf: &mut [u8]) -> c_int {
    match SysRng.try_fill_bytes(buf) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[inline]
fn w1_mix(a: u64, b: u64) -> u64 {
    let t = u128::from(a) * u128::from(b);
    (t >> 64) as u64 ^ t as u64
}

#[cold]
#[inline(never)]
fn seed_rng() -> c_int {
    let mut buf = [0_u8; 16];
    if fill_random(&mut buf) != 0 {
        PyOSError::new_err(c"unable to generate random bytes");
        return -1;
    }
    let l = u64::from_ne_bytes(buf[..8].try_into().expect("8-byte seed chunk"));
    let r = u64::from_ne_bytes(buf[8..].try_into().expect("8-byte seed chunk"));
    W1_STATE.store(l ^ w1_mix(r, r ^ C), Ordering::Relaxed);
    W1_SEEDED.store(true, Ordering::Relaxed);
    platform_seeded();
    0
}

#[inline]
pub fn ensure_seeded() -> c_int {
    if W1_SEEDED.load(Ordering::Relaxed) {
        0
    } else {
        seed_rng()
    }
}

pub fn reseed() {
    W1_SEEDED.store(false, Ordering::Relaxed);
    LAST_MS.store(0, Ordering::Relaxed);
    COUNTER42.store(0, Ordering::Relaxed);
    RNG.with_borrow_mut(|rng| *rng = rand::make_rng());
}

#[inline]
fn advance_monotonic_with(
    observed_ms: u64,
    timestamp_ms: &mut u64,
    rand_a: &mut u16,
    tail62: &mut u64,
    rng: &mut impl WordRng,
) {
    let mut counter = COUNTER42.load(Ordering::Relaxed);
    let mut current_ms = LAST_MS.load(Ordering::Relaxed);

    let r = rng.next_word();
    let low32 = r as u32;

    if observed_ms > current_ms {
        current_ms = observed_ms;
        counter = rng.next_word() & MASK42;
    } else {
        let increment = 1 + ((r >> 32) & 0x0f);
        counter += increment;
        if counter > MASK42 {
            current_ms += 1;
            counter = rng.next_word() & MASK42;
        }
    }

    LAST_MS.store(current_ms, Ordering::Relaxed);
    COUNTER42.store(counter, Ordering::Relaxed);

    *timestamp_ms = current_ms;
    *rand_a = (counter >> 30) as u16;
    *tail62 = ((counter & MASK30) << 32) | u64::from(low32);
}

#[inline]
pub fn build_words(ts_ms: u64, rand_a: u16, tail62: u64) -> (u64, u64) {
    const V7_VERSION: u64 = 0x7000;
    const V7_VARIANT: u64 = 0x8000_0000_0000_0000;

    (
        (ts_ms << 16) | V7_VERSION | u64::from(rand_a),
        V7_VARIANT | tail62,
    )
}

pub struct Fast;
pub struct Secure;

trait WordRng {
    fn next_word(&mut self) -> u64;
}

struct W1Rng {
    state: u64,
}

impl W1Rng {
    #[inline]
    fn new() -> Self {
        Self {
            state: W1_STATE.load(Ordering::Relaxed),
        }
    }
}

impl Drop for W1Rng {
    #[inline]
    fn drop(&mut self) {
        W1_STATE.store(self.state, Ordering::Relaxed);
    }
}

impl WordRng for W1Rng {
    #[inline]
    fn next_word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(C);
        w1_mix(self.state, self.state ^ C)
    }
}

impl WordRng for ChaCha12Rng {
    #[inline]
    fn next_word(&mut self) -> u64 {
        Rng::next_u64(self)
    }
}

impl Fast {
    #[inline]
    pub fn build_uuid7(high: &mut u64, low: &mut u64) -> c_int {
        if ensure_seeded() != 0 {
            return -1;
        }

        let mut rng = W1Rng::new();
        build_uuid7_monotonic(now_ms(), high, low, &mut rng)
    }

    #[inline]
    pub fn build_uuid7_with_args(
        ts_ms: u64,
        has_ts: bool,
        nanos: u64,
        has_nanos: bool,
        high: &mut u64,
        low: &mut u64,
    ) -> c_int {
        if ensure_seeded() != 0 {
            return -1;
        }

        let mut rng = W1Rng::new();
        build_uuid7_from_args(ts_ms, has_ts, nanos, has_nanos, high, low, &mut rng)
    }
}

impl Secure {
    #[inline]
    pub fn build_uuid7(high: &mut u64, low: &mut u64) -> c_int {
        if ensure_seeded() != 0 {
            return -1;
        }

        RNG.with_borrow_mut(|rng| build_uuid7_monotonic(now_ms(), high, low, rng))
    }

    #[inline]
    pub fn build_uuid7_with_args(
        ts_ms: u64,
        has_ts: bool,
        nanos: u64,
        has_nanos: bool,
        high: &mut u64,
        low: &mut u64,
    ) -> c_int {
        if ensure_seeded() != 0 {
            return -1;
        }

        RNG.with_borrow_mut(|rng| {
            build_uuid7_from_args(ts_ms, has_ts, nanos, has_nanos, high, low, rng)
        })
    }
}

#[inline]
fn build_uuid7_monotonic(
    observed_ms: u64,
    high: &mut u64,
    low: &mut u64,
    rng: &mut impl WordRng,
) -> c_int {
    let (mut ts, mut ra, mut t62) = (0_u64, 0_u16, 0_u64);
    advance_monotonic_with(observed_ms, &mut ts, &mut ra, &mut t62, rng);
    let (hi, lo) = build_words(ts, ra, t62);
    *high = hi;
    *low = lo;
    0
}

#[inline]
fn extract_random_bits_with(
    has_ts: bool,
    has_nanos: bool,
    nanos: u64,
    rand_a: &mut u16,
    tail62: &mut u64,
    rng: &mut impl WordRng,
) -> bool {
    if has_ts && has_nanos {
        *rand_a = (nanos & 0x0FFF) as u16;
        *tail62 = rng.next_word() & MASK62;
        return false;
    }
    if has_ts || has_nanos {
        let c = rng.next_word();
        let r = rng.next_word();
        let counter = c & MASK42;
        *rand_a = (counter >> 30) as u16;
        *tail62 = ((counter & MASK30) << 32) | u64::from(r as u32);
        return false;
    }
    true
}

#[inline]
fn build_uuid7_from_args(
    ts_ms: u64,
    has_ts: bool,
    nanos: u64,
    has_nanos: bool,
    high: &mut u64,
    low: &mut u64,
    rng: &mut impl WordRng,
) -> c_int {
    let (mut ra, mut t62) = (0_u16, 0_u64);

    let (hi, lo) = if extract_random_bits_with(has_ts, has_nanos, nanos, &mut ra, &mut t62, rng) {
        let mut ms = ts_ms;
        advance_monotonic_with(ms, &mut ms, &mut ra, &mut t62, rng);
        build_words(ms, ra, t62)
    } else {
        build_words(ts_ms, ra, t62)
    };
    *high = hi;
    *low = lo;
    0
}

#[inline]
pub fn build_timestamp_ms(ts_s: u64, has_ts: bool, nanos: u64, has_nanos: bool) -> Result<u64, ()> {
    const MAX_TS_MS: u64 = 0xFFFF_FFFF_FFFF;
    const MAX_TS_S: u64 = MAX_TS_MS / 1000;

    if !has_ts {
        return Ok(now_ms());
    }

    if ts_s > MAX_TS_S {
        PyValueError::new_err(c"timestamp is too large");
        return Err(());
    }

    let mut ms = ts_s * 1000;

    if has_nanos {
        ms += nanos / 1_000_000;
    }

    if ms > MAX_TS_MS {
        PyValueError::new_err(c"timestamp is too large");
        return Err(());
    }
    Ok(ms)
}
