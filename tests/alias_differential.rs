//! Differential checks against output fingerprints captured from published rc2.

#[path = "support/hex.rs"]
mod hex;
#[path = "support/legacy_alias.rs"]
mod legacy_alias;

use sha2::{Digest, Sha256};
use zlid::ZLID;

const SOURCE: [u8; 16] = [
    0x01, 0x98, 0xb0, 0x79, 0xbf, 0x8e, 0xab, 0xab, 0xc1, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xe1,
];
const TWEAK_LENGTHS: [usize; 10] = [0, 1, 55, 56, 63, 64, 65, 255, 1024, 65_535];
const KEY_ORACLES: [(usize, &str); 5] = [
    (
        1,
        "A3705D8918BE0986A4D47D1869258DACDA9612A24DF3EDDC92B0D32933770AD1",
    ),
    (
        63,
        "B9F4ADA7CB07304773BB26BF1872D110C033245AE6E55DE433D0004320EA5930",
    ),
    (
        64,
        "05BFC648C6101894F4AC4BB62ACE771160B9ADD0504A34AE499228E9964A01F0",
    ),
    (
        65,
        "2341EDFB1E34E1137AA6518F515C45C8B7A04E5BA7F0FE2C51806AEBD846BA93",
    ),
    (
        131,
        "A55767ED574C4A2F2D63E73E9D595620970D1955D73F45F0ABA1DFA5E80F2007",
    ),
];
const ORDERED_SOURCES: [(&str, u8, u8); 4] = [
    ("0198B079BF8E2A007000000000000001", 1, 6),
    ("0198B079BF8E2A009000000000000003", 3, 7),
    ("0198B079BF8E2ABEEF00000000000002", 2, 8),
    ("0198B079BF8E2A000500000000000004", 4, 9),
];

#[test]
fn optimized_alias_matches_rc2_boundary_oracle() {
    let source = ZLID::from_array(SOURCE);
    for (key_length, expected) in KEY_ORACLES {
        let key = generated_bytes(key_length, 37, 11);
        let mut fingerprint = Sha256::new();
        for tweak_length in TWEAK_LENGTHS {
            let tweak = generated_bytes(tweak_length, 29, 7);
            let alias = source.alias(&key, &tweak).unwrap();
            assert_eq!(alias.unalias(&key, &tweak).unwrap(), source);
            fingerprint.update(alias.as_bytes());
        }
        assert_eq!(hex::encode(&fingerprint.finalize()), expected);
    }
}

#[test]
fn optimized_alias_is_byte_exact_with_removed_implementation() {
    for (source_hex, source_tag, alias_tag) in ORDERED_SOURCES {
        let source_bytes = hex::decode(source_hex).unwrap();
        let source = ZLID::from_bytes(&source_bytes).unwrap();
        for key_length in [1, 16, 63, 64, 65, 131] {
            let key = generated_bytes(key_length, 37, 11);
            for tweak_length in TWEAK_LENGTHS {
                let tweak = generated_bytes(tweak_length, 29, 7);
                let optimized = source.alias(&key, &tweak).unwrap();
                let legacy = legacy_alias::alias(source.bytes(), alias_tag, &key, &tweak);
                assert_eq!(optimized.bytes(), legacy);
                assert_eq!(
                    legacy_alias::unalias(legacy, source_tag, &key, &tweak),
                    source.bytes()
                );
                assert_eq!(optimized.unalias(&key, &tweak).unwrap(), source);
            }
        }
    }
}

fn generated_bytes(length: usize, multiplier: usize, salt: usize) -> Vec<u8> {
    (0..length)
        .map(|index| ((index * multiplier + length * salt) & 0xff) as u8)
        .collect()
}
