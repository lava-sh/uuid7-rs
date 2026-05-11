#[expect(clippy::large_stack_arrays, clippy::large_stack_frames)]
const fn build_hex_words() -> [u32; 65536] {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut tmp = [0; 65536];
    let mut i = 0;

    while i < 65536 {
        tmp[i] = u32::from_ne_bytes([
            HEX_DIGITS[(i >> 12) & 0x0f],
            HEX_DIGITS[(i >> 8) & 0x0f],
            HEX_DIGITS[(i >> 4) & 0x0f],
            HEX_DIGITS[i & 0x0f],
        ]);
        i += 1;
    }

    tmp
}

const fn hex_nibble(c: u8) -> i8 {
    match c {
        b'0'..=b'9' => (c - b'0').cast_signed(),
        b'a'..=b'f' => (c - b'a' + 10).cast_signed(),
        b'A'..=b'F' => (c - b'A' + 10).cast_signed(),
        _ => -1,
    }
}

#[expect(clippy::large_stack_arrays)]
const fn build_hex_pairs() -> [i16; 65536] {
    let mut tmp = [-1_i16; 65536];
    let mut hi_c = 0_u16;

    while hi_c < 256 {
        let hn = hex_nibble(hi_c as u8);

        if hn >= 0 {
            let mut lo_c = 0_u16;

            while lo_c < 256 {
                let ln = hex_nibble(lo_c as u8);

                if ln >= 0 {
                    let idx = hi_c | (lo_c << 8);
                    tmp[idx as usize] = ((hn as i16) << 4) | ln as i16;
                }

                lo_c += 1;
            }
        }

        hi_c += 1;
    }

    tmp
}

pub static HEX_WORDS: [u32; 65536] = build_hex_words();
pub static HEX_PAIRS: [i16; 65536] = build_hex_pairs();
