use crate::constants::ZERO_PARTITION_KEY;
use crate::crypto::siphash24;
use crate::error::{Error, Result};

/// Computes SipHash-2-4(key16, input) & 0xff for byte input.
pub fn partition_bytes(input: &[u8], key: Option<&[u8]>) -> Result<u8> {
    let key = normalize_partition_key(key)?;
    Ok((siphash24(&key, input) & 0xff) as u8)
}

/// Computes SipHash-2-4(key16, utf8(input)) & 0xff.
pub fn partition_str(input: &str, key: Option<&[u8]>) -> Result<u8> {
    partition_bytes(input.as_bytes(), key)
}

fn normalize_partition_key(key: Option<&[u8]>) -> Result<[u8; 16]> {
    match key {
        None => Ok(ZERO_PARTITION_KEY),
        Some(key) if key.len() == 16 => {
            let mut out = [0u8; 16];
            out.copy_from_slice(key);
            Ok(out)
        }
        Some(key) => Err(Error::InvalidLength {
            what: "partition key",
            expected: 16,
            actual: key.len(),
        }),
    }
}
