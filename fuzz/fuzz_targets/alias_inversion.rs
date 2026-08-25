#![no_main]

use libfuzzer_sys::fuzz_target;
use zlid::{Family, ZLID};

const TAG_PAIRS: [(u8, u8); 4] = [(1, 6), (2, 8), (3, 7), (4, 9)];
const MAX_KEY_LENGTH: usize = 256;

fuzz_target!(|input: &[u8]| {
    if input.len() < ZLID::BYTE_LENGTH + 3 {
        return;
    }

    let mut source_bytes = [0u8; ZLID::BYTE_LENGTH];
    source_bytes.copy_from_slice(&input[..ZLID::BYTE_LENGTH]);
    let (source_tag, alias_tag) = TAG_PAIRS[input[ZLID::BYTE_LENGTH] as usize % TAG_PAIRS.len()];
    source_bytes[ZLID::BYTE_LENGTH - 1] = (source_bytes[ZLID::BYTE_LENGTH - 1] & 0xf0) | source_tag;

    let length_selector = input[ZLID::BYTE_LENGTH + 1];
    let material = &input[ZLID::BYTE_LENGTH + 2..];
    let maximum = material.len().min(MAX_KEY_LENGTH);
    if maximum == 0 {
        return;
    }
    let key_length = 1 + length_selector as usize % maximum;
    let (key, tweak) = material.split_at(key_length);
    let source = ZLID::from_array(source_bytes);

    let alias = source
        .alias(key, tweak)
        .expect("valid ordered source, nonempty key, and bounded tweak");
    assert_eq!(alias.tag(), alias_tag);
    assert_eq!(alias.family(), Some(Family::Alias));
    assert_eq!(source.alias(key, tweak), Ok(alias));
    assert_eq!(alias.unalias(key, tweak), Ok(source));
});
