use super::sha256;

pub(crate) fn hmac_sha256(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let digest = sha256(key);
        key_block[..32].copy_from_slice(&digest);
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
