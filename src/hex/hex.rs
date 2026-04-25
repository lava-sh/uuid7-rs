use std::os::raw::c_int;

use crate::{
    hex::table::{HEX_PAIR_TO_BYTE, HEX_PAIRS},
    parse::bytes_to_hilo,
};

#[inline]
pub fn hex_pair(buf: &mut [u8], pos: usize, byte: u8) {
    let i = byte as usize * 2;
    unsafe {
        *buf.get_unchecked_mut(pos) = *HEX_PAIRS.get_unchecked(i);
        *buf.get_unchecked_mut(pos + 1) = *HEX_PAIRS.get_unchecked(i + 1);
    }
}

#[inline]
pub fn fmt_dashed(hi: u64, lo: u64, buf: &mut [u8]) {
    let h = hi.to_be_bytes();
    let l = lo.to_be_bytes();
    hex_pair(buf, 0, h[0]);
    hex_pair(buf, 2, h[1]);
    hex_pair(buf, 4, h[2]);
    hex_pair(buf, 6, h[3]);
    buf[8] = b'-';
    hex_pair(buf, 9, h[4]);
    hex_pair(buf, 11, h[5]);
    buf[13] = b'-';
    hex_pair(buf, 14, h[6]);
    hex_pair(buf, 16, h[7]);
    buf[18] = b'-';
    hex_pair(buf, 19, l[0]);
    hex_pair(buf, 21, l[1]);
    buf[23] = b'-';
    hex_pair(buf, 24, l[2]);
    hex_pair(buf, 26, l[3]);
    hex_pair(buf, 28, l[4]);
    hex_pair(buf, 30, l[5]);
    hex_pair(buf, 32, l[6]);
    hex_pair(buf, 34, l[7]);
}

pub fn fmt_hex32(hi: u64, lo: u64, buf: &mut [u8]) {
    let h = hi.to_be_bytes();
    let l = lo.to_be_bytes();
    for (i, byte) in h.iter().enumerate() {
        hex_pair(buf, i * 2, *byte);
    }
    for (i, byte) in l.iter().enumerate() {
        hex_pair(buf, 16 + i * 2, *byte);
    }
}

pub fn parse_hex32(text: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    let mut bytes = [0u8; 16];
    let mut i = 0;
    while i < 16 {
        let v = HEX_PAIR_TO_BYTE[(text[i * 2] as usize) << 8 | text[i * 2 + 1] as usize];
        if v < 0 {
            return -1;
        }
        bytes[i] = u8::try_from(v).expect("hex table values are in byte range");
        i += 1;
    }
    bytes_to_hilo(bytes.as_ptr(), hi, lo);
    0
}

pub fn parse_uuid_hex_str(mut str: &[u8], hi: &mut u64, lo: &mut u64) -> c_int {
    if str.len() >= 9 && str[..9].eq_ignore_ascii_case(b"urn:uuid:") {
        str = &str[9..];
    }

    if str.len() >= 2 && str[0] == b'{' && str[str.len() - 1] == b'}' {
        str = &str[1..str.len() - 1];
    }

    match str.len() {
        32 => parse_hex32(str, hi, lo),
        36 => {
            if str[8] != b'-' || str[13] != b'-' || str[18] != b'-' || str[23] != b'-' {
                return -1;
            }
            let mut flat = [0u8; 32];
            unsafe {
                std::ptr::copy_nonoverlapping(str.as_ptr(), flat.as_mut_ptr(), 8);
                std::ptr::copy_nonoverlapping(str.as_ptr().add(9), flat.as_mut_ptr().add(8), 4);
                std::ptr::copy_nonoverlapping(str.as_ptr().add(14), flat.as_mut_ptr().add(12), 4);
                std::ptr::copy_nonoverlapping(str.as_ptr().add(19), flat.as_mut_ptr().add(16), 4);
                std::ptr::copy_nonoverlapping(str.as_ptr().add(24), flat.as_mut_ptr().add(20), 12);
            }
            parse_hex32(&flat, hi, lo)
        }
        _ => -1,
    }
}
