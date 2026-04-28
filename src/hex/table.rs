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
    match c {
        b'0'..=b'9' => (c - b'0').cast_signed(),
        b'a'..=b'f' => (c - b'a' + 10).cast_signed(),
        b'A'..=b'F' => (c - b'A' + 10).cast_signed(),
        _ => -1,
    }
}

#[expect(clippy::large_stack_arrays)]
const fn build_hex_pair_to_byte() -> [i16; 65536] {
    let mut tmp = [-1i16; 65536];
    let mut i = 0u16;

    while i < 256 {
        let hn = hex_nibble(i as u8);

        if hn >= 0 {
            let mut l = 0u16;

            while l < 256 {
                let ln = hex_nibble(l as u8);

                if ln >= 0 {
                    tmp[(i << 8 | l) as usize] = ((hn as i16) << 4) | ln as i16;
                }

                l += 1;
            }
        }

        i += 1;
    }

    tmp
}

pub static HEX_PAIRS: [u8; 512] = build_hex_pairs();
pub static HEX_PAIR_TO_BYTE: [i16; 65536] = build_hex_pair_to_byte();
