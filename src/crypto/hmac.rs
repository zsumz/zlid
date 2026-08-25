use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type RustCryptoHmacSha256 = Hmac<Sha256>;

pub(crate) struct HmacSha256 {
    keyed: RustCryptoHmacSha256,
}

impl HmacSha256 {
    pub(crate) fn new(key: &[u8]) -> Self {
        let keyed = match RustCryptoHmacSha256::new_from_slice(key) {
            Ok(keyed) => keyed,
            Err(_) => unreachable!("HMAC-SHA256 accepts keys of any length"),
        };
        HmacSha256 { keyed }
    }

    pub(crate) fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut mac = self.keyed.clone();
        for part in parts {
            mac.update(part);
        }
        let output = mac.finalize().into_bytes();
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&output);
        digest
    }
}

#[cfg(test)]
#[path = "hmac_tests.rs"]
mod tests;
