pub(crate) fn siphash24(key: &[u8; 16], input: &[u8]) -> u64 {
    const V0_INIT: u64 = 0x736f6d6570736575;
    const V1_INIT: u64 = 0x646f72616e646f6d;
    const V2_INIT: u64 = 0x6c7967656e657261;
    const V3_INIT: u64 = 0x7465646279746573;

    let k0 = read_u64_le(&key[0..8]);
    let k1 = read_u64_le(&key[8..16]);
    let mut v0 = V0_INIT ^ k0;
    let mut v1 = V1_INIT ^ k1;
    let mut v2 = V2_INIT ^ k0;
    let mut v3 = V3_INIT ^ k1;

    let full_length = input.len() - (input.len() % 8);
    let mut offset = 0;
    while offset < full_length {
        let m = read_u64_le(&input[offset..offset + 8]);
        v3 ^= m;
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= m;
        offset += 8;
    }

    let mut final_block = (input.len() as u64) << 56;
    for (index, byte) in input[full_length..].iter().enumerate() {
        final_block |= u64::from(*byte) << (8 * index);
    }

    v3 ^= final_block;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= final_block;
    v2 ^= 0xff;
    for _ in 0..4 {
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    }

    v0 ^ v1 ^ v2 ^ v3
}

fn read_u64_le(bytes: &[u8]) -> u64 {
    let mut raw = [0u8; 8];
    raw.copy_from_slice(bytes);
    u64::from_le_bytes(raw)
}

fn sip_round(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
    *v0 = v0.wrapping_add(*v1);
    *v1 = v1.rotate_left(13);
    *v1 ^= *v0;
    *v0 = v0.rotate_left(32);

    *v2 = v2.wrapping_add(*v3);
    *v3 = v3.rotate_left(16);
    *v3 ^= *v2;

    *v0 = v0.wrapping_add(*v3);
    *v3 = v3.rotate_left(21);
    *v3 ^= *v0;

    *v2 = v2.wrapping_add(*v1);
    *v1 = v1.rotate_left(17);
    *v1 ^= *v2;
    *v2 = v2.rotate_left(32);
}

#[cfg(test)]
#[path = "siphash_tests.rs"]
mod tests;
