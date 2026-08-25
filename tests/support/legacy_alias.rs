#[path = "legacy_sha256.rs"]
mod legacy_sha256;

use legacy_sha256::sha256;

const MASK_62: u64 = (1u64 << 62) - 1;

pub(super) fn alias(source: [u8; 16], alias_tag: u8, key: &[u8], tweak: &[u8]) -> [u8; 16] {
    let source_data = u128::from_be_bytes(source) >> 4;
    let alias_data = permute124(key, tweak, source_data);
    ((alias_data << 4) | u128::from(alias_tag)).to_be_bytes()
}

pub(super) fn unalias(alias: [u8; 16], source_tag: u8, key: &[u8], tweak: &[u8]) -> [u8; 16] {
    let alias_data = u128::from_be_bytes(alias) >> 4;
    let source_data = inverse_permute124(key, tweak, alias_data);
    ((source_data << 4) | u128::from(source_tag)).to_be_bytes()
}

fn permute124(key: &[u8], tweak: &[u8], data124: u128) -> u128 {
    let mut left = ((data124 >> 62) as u64) & MASK_62;
    let mut right = data124 as u64 & MASK_62;
    for round in 0..8u8 {
        let next = (left ^ round_function(key, tweak, round, right)) & MASK_62;
        left = right;
        right = next;
    }
    (u128::from(left) << 62) | u128::from(right)
}

fn inverse_permute124(key: &[u8], tweak: &[u8], data124: u128) -> u128 {
    let mut left = ((data124 >> 62) as u64) & MASK_62;
    let mut right = data124 as u64 & MASK_62;
    for round in (0..8u8).rev() {
        let next_left = (right ^ round_function(key, tweak, round, left)) & MASK_62;
        right = left;
        left = next_left;
    }
    (u128::from(left) << 62) | u128::from(right)
}

fn round_function(key: &[u8], tweak: &[u8], round: u8, right: u64) -> u64 {
    let round_byte = [round];
    let tweak_len = (tweak.len() as u16).to_be_bytes();
    let right_bytes = right.to_be_bytes();
    let digest = hmac_sha256(
        key,
        &[b"ZLID-A-F", &round_byte, &tweak_len, tweak, &right_bytes],
    );
    u64::from_be_bytes(digest[24..32].try_into().unwrap()) & MASK_62
}

fn hmac_sha256(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        key_block[..32].copy_from_slice(&sha256(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5cu8; 64];
    for index in 0..64 {
        inner_pad[index] ^= key_block[index];
        outer_pad[index] ^= key_block[index];
    }
    let mut inner = Vec::from(inner_pad);
    for part in parts {
        inner.extend_from_slice(part);
    }
    let inner_hash = sha256(&inner);
    let mut outer = Vec::from(outer_pad);
    outer.extend_from_slice(&inner_hash);
    sha256(&outer)
}
