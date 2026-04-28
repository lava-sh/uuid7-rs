use std::os::raw::c_int;

use crate::hex::table::{HEX_PAIR_TO_BYTE, HEX_PAIRS};

macro_rules! hex_byte {
    ($text:expr, $pos:expr) => {{
        let v = HEX_PAIR_TO_BYTE[($text[$pos] as usize) << 8 | $text[$pos + 1] as usize];
        if v < 0 {
            return -1;
        }
        u64::from(v.cast_unsigned())
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

#[inline]
pub fn hex_pair(buf: &mut [u8], pos: usize, byte: u8) {
    let i = byte as usize * 2;
    unsafe {
        *buf.get_unchecked_mut(pos) = *HEX_PAIRS.get_unchecked(i);
        *buf.get_unchecked_mut(pos + 1) = *HEX_PAIRS.get_unchecked(i + 1);
    }
}

#[inline]
pub fn fmt_dashed(high: u64, low: u64, buf: &mut [u8]) {
    let hi = high.to_be_bytes();
    let lo = low.to_be_bytes();
    hex_pair(buf, 0, hi[0]);
    hex_pair(buf, 2, hi[1]);
    hex_pair(buf, 4, hi[2]);
    hex_pair(buf, 6, hi[3]);
    buf[8] = b'-';
    hex_pair(buf, 9, hi[4]);
    hex_pair(buf, 11, hi[5]);
    buf[13] = b'-';
    hex_pair(buf, 14, hi[6]);
    hex_pair(buf, 16, hi[7]);
    buf[18] = b'-';
    hex_pair(buf, 19, lo[0]);
    hex_pair(buf, 21, lo[1]);
    buf[23] = b'-';
    hex_pair(buf, 24, lo[2]);
    hex_pair(buf, 26, lo[3]);
    hex_pair(buf, 28, lo[4]);
    hex_pair(buf, 30, lo[5]);
    hex_pair(buf, 32, lo[6]);
    hex_pair(buf, 34, lo[7]);
}

pub fn fmt_hex32(high: u64, low: u64, buf: &mut [u8]) {
    let hi = high.to_be_bytes();
    let lo = low.to_be_bytes();

    for (i, byte) in hi.iter().enumerate() {
        hex_pair(buf, i * 2, *byte);
    }

    for (i, byte) in lo.iter().enumerate() {
        hex_pair(buf, 16 + i * 2, *byte);
    }
}

pub fn parse_uuid_hex_str(mut py_str: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    if py_str.len() >= 9 && py_str[..9].eq_ignore_ascii_case(b"urn:uuid:") {
        py_str = &py_str[9..];
    }

    if py_str.len() >= 2 && py_str[0] == b'{' && py_str[py_str.len() - 1] == b'}' {
        py_str = &py_str[1..py_str.len() - 1];
    }

    match py_str.len() {
        32 => {
            *hi = hex_word!(py_str, 0, 2, 4, 6, 8, 10, 12, 14);
            *lo = hex_word!(py_str, 16, 18, 20, 22, 24, 26, 28, 30);
            0
        }
        36 => {
            if py_str[8] != b'-' || py_str[13] != b'-' || py_str[18] != b'-' || py_str[23] != b'-' {
                return -1;
            }
            *hi = hex_word!(py_str, 0, 2, 4, 6, 9, 11, 14, 16);
            *lo = hex_word!(py_str, 19, 21, 24, 26, 28, 30, 32, 34);
            0
        }
        _ => -1,
    }
}
