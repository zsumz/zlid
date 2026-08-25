use crate::constants::{BYTE_LENGTH, TAG_ZLID_RANDOM};
use crate::error::{Error, Result};
use crate::profile::max_value_for_bits;

/// Source of random bytes for ZLID generation.
pub trait EntropySource {
    /// Fills the complete output slice with random bytes.
    fn fill_bytes(&mut self, out: &mut [u8]) -> Result<()>;
}

/// System entropy source.
#[derive(Debug, Default, Copy, Clone)]
pub struct SystemEntropy;

impl EntropySource for SystemEntropy {
    fn fill_bytes(&mut self, out: &mut [u8]) -> Result<()> {
        fill_system_random(out)
    }
}

impl<F> EntropySource for F
where
    F: FnMut(usize) -> Vec<u8>,
{
    fn fill_bytes(&mut self, out: &mut [u8]) -> Result<()> {
        let bytes = self(out.len());
        if bytes.len() != out.len() {
            return Err(Error::InvalidLength {
                what: "random source output",
                expected: out.len(),
                actual: bytes.len(),
            });
        }
        out.copy_from_slice(&bytes);
        Ok(())
    }
}

pub(crate) fn random_zlid<E: EntropySource>(source: &mut E) -> Result<[u8; BYTE_LENGTH]> {
    let mut bytes = [0u8; BYTE_LENGTH];
    source.fill_bytes(&mut bytes)?;
    bytes[15] = (bytes[15] & 0xf0) | TAG_ZLID_RANDOM;
    Ok(bytes)
}

pub(crate) fn random_value<E: EntropySource>(source: &mut E, bit_count: u8) -> Result<u64> {
    let byte_count = usize::from(bit_count.div_ceil(8));
    let mut buffer = [0u8; std::mem::size_of::<u64>()];
    let bytes = &mut buffer[..byte_count];
    source.fill_bytes(bytes)?;
    let mut value = 0u64;
    for &byte in bytes.iter() {
        value = (value << 8) | u64::from(byte);
    }
    let extra_bits = byte_count as u8 * 8 - bit_count;
    if extra_bits == 0 {
        Ok(value)
    } else {
        Ok(value & max_value_for_bits(bit_count))
    }
}

fn fill_system_random(out: &mut [u8]) -> Result<()> {
    getrandom::fill(out).map_err(|error| Error::EntropyUnavailable(error.to_string()))
}

#[cfg(test)]
#[path = "random_tests.rs"]
mod tests;
