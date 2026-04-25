const fn build_hex_pairs() -> [u8; 512] {
    let hex_digits = b"0123456789abcdef";
    let mut tmp = [0u8; 512];
    let mut i = 0usize;

    while i < 256 {
        tmp[i * 2] = hex_digits[i >> 4];
        tmp[i * 2 + 1] = hex_digits[i & 0x0f];
        i += 1;
    }

    tmp
}

const fn hex_nibble(c: u8) -> i8 {
    if c >= b'0' && c <= b'9' {
        return (c - b'0').cast_signed();
    }
    if c >= b'a' && c <= b'f' {
        return (c - b'a' + 10).cast_signed();
    }
    if c >= b'A' && c <= b'F' {
        return (c - b'A' + 10).cast_signed();
    }
    -1
}

#[allow(clippy::large_stack_arrays)]
const fn build_hex_pair_to_byte() -> [i16; 65536] {
    let mut tmp = [-1i16; 65536];
    let mut h = 0u16;

    while h < 256 {
        let hn = hex_nibble(h as u8);

        if hn >= 0 {
            let mut l = 0u16;

            while l < 256 {
                let ln = hex_nibble(l as u8);

                if ln >= 0 {
                    tmp[(h << 8 | l) as usize] = ((hn as i16) << 4) | ln as i16;
                }

                l += 1;
            }
        }

        h += 1;
    }

    tmp
}

pub static HEX_PAIRS: [u8; 512] = build_hex_pairs();
pub static HEX_PAIR_TO_BYTE: [i16; 65536] = build_hex_pair_to_byte();
