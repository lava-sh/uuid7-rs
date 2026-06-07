use std::{ffi::c_int, ptr};

use crate::hex::table::{HEX_PAIRS, HEX_WORDS};

#[expect(clippy::inline_always)]
#[inline(always)]
fn hex_byte_le(text: *const u8, pos: usize) -> i16 {
    unsafe {
        let i = ptr::read_unaligned(text.add(pos).cast::<u16>()) as usize;
        *HEX_PAIRS.get_unchecked(i)
    }
}

macro_rules! hex_byte {
    ($text:expr, $pos:expr) => {{
        let x = hex_byte_le($text, $pos);
        if x < 0 {
            return -1;
        }
        u64::from(x.cast_unsigned())
    }};
}

macro_rules! hex_word {
    ($text:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr) => {
        (hex_byte!($text, $a) << 56)
            | (hex_byte!($text, $b) << 48)
            | (hex_byte!($text, $c) << 40)
            | (hex_byte!($text, $d) << 32)
            | (hex_byte!($text, $e) << 24)
            | (hex_byte!($text, $f) << 16)
            | (hex_byte!($text, $g) << 8)
            | hex_byte!($text, $h)
    };
}

#[expect(clippy::inline_always)]
#[inline(always)]
fn parse_hex32(py_str: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    let ptr = py_str.as_ptr();
    *hi = hex_word!(ptr, 0, 2, 4, 6, 8, 10, 12, 14);
    *lo = hex_word!(ptr, 16, 18, 20, 22, 24, 26, 28, 30);
    0
}

#[expect(clippy::inline_always)]
#[inline(always)]
fn parse_dashed(py_str: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    let ptr = py_str.as_ptr();
    let dashes = unsafe {
        (*ptr.add(8) ^ b'-') | (*ptr.add(13) ^ b'-') | (*ptr.add(18) ^ b'-') | (*ptr.add(23) ^ b'-')
    };
    if dashes != 0 {
        return -1;
    }
    *hi = hex_word!(ptr, 0, 2, 4, 6, 9, 11, 14, 16);
    *lo = hex_word!(ptr, 19, 21, 24, 26, 28, 30, 32, 34);
    0
}

#[inline(always)]
fn is_urn_uuid(py_str: &[u8]) -> bool {
    const URN_UUID: u64 = u64::from_le_bytes(*b"urn:uuid") | 0x2020_2020_2020_2020;

    let ptr = py_str.as_ptr();
    unsafe {
        (ptr::read_unaligned(ptr.cast::<u64>()) | 0x2020_2020_2020_2020) == URN_UUID
            && *ptr.add(8) == b':'
    }
}

#[expect(clippy::inline_always)]
#[inline(always)]
pub fn parse_uuid_hex_str(mut py_str: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    if py_str.len() >= 9 && is_urn_uuid(py_str) {
        py_str = &py_str[9..];
    }

    if py_str.len() >= 2 && py_str[0] == b'{' && py_str[py_str.len() - 1] == b'}' {
        py_str = &py_str[1..py_str.len() - 1];
    }

    match py_str.len() {
        32 => parse_hex32(py_str, hi, lo),
        36 => parse_dashed(py_str, hi, lo),
        _ => -1,
    }
}

#[expect(clippy::inline_always)]
#[inline(always)]
fn hex_word(bytes: u16) -> u64 {
    u64::from(unsafe { *HEX_WORDS.get_unchecked(bytes as usize) })
}

#[expect(clippy::inline_always)]
#[inline(always)]
fn hex_words(x: u64) -> [u64; 4] {
    [
        hex_word((x >> 48) as u16),
        hex_word((x >> 32) as u16),
        hex_word((x >> 16) as u16),
        hex_word(x as u16),
    ]
}

macro_rules! write_unaligned {
    ($buf:expr, $pos:expr, $val:expr) => {
        unsafe { ptr::write_unaligned($buf.add($pos).cast(), $val) }
    };
}

#[expect(clippy::inline_always)]
#[inline(always)]
pub fn fmt_hex32(high: u64, low: u64, buf: &mut [u8]) {
    let ptr = buf.as_mut_ptr();
    let hi = hex_words(high);
    let lo = hex_words(low);

    write_unaligned!(ptr, 0, hi[0] | (hi[1] << 32));
    write_unaligned!(ptr, 8, hi[2] | (hi[3] << 32));
    write_unaligned!(ptr, 16, lo[0] | (lo[1] << 32));
    write_unaligned!(ptr, 24, lo[2] | (lo[3] << 32));
}

#[inline(always)]
pub fn fmt_dashed(high: u64, low: u64, buf: &mut [u8]) {
    const DASH: u64 = b'-' as u64;

    let ptr = buf.as_mut_ptr();
    let hi = hex_words(high);
    let lo = hex_words(low);

    write_unaligned!(ptr, 0, hi[0] | (hi[1] << 32));
    write_unaligned!(
        ptr,
        8,
        DASH | (hi[2] << 8) | (DASH << 40) | ((hi[3] & 0xFFFF) << 48)
    );
    write_unaligned!(
        ptr,
        16,
        (hi[3] >> 16) | (DASH << 16) | (lo[0] << 24) | (DASH << 56)
    );
    write_unaligned!(ptr, 24, lo[1] | (lo[2] << 32));
    write_unaligned!(ptr, 32, lo[3] as u32);
}
