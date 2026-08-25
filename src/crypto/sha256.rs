use sha2::{Digest, Sha256};

pub(crate) fn sha256(input: &[u8]) -> [u8; 32] {
    let output = Sha256::digest(input);
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&output);
    digest
}

#[cfg(test)]
#[path = "sha256_tests.rs"]
mod tests;
