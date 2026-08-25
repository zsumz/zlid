use super::{decode_text, encode_text, encode_text_array, encoded_text_str};
use crate::constants::{ALPHABET, BYTE_LENGTH, STRING_LENGTH};
use crate::error::{Error, Result};
use crate::ZLID;

#[test]
fn stack_encoder_matches_the_original_algorithm() {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..2_048 {
        let mut bytes = [0u8; BYTE_LENGTH];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }

        let expected = legacy_encode(&bytes);
        let encoded = encode_text_array(&bytes);
        assert_eq!(encoded_text_str(&encoded), expected);
        assert_eq!(encode_text(&bytes), expected);
    }
}

#[test]
fn decoder_matches_original_behavior_for_ascii_mutations() {
    const CANONICAL: &str = "01K2R7KFWE5807000000000001";

    for byte in 0u8..=127 {
        let mut replacement = CANONICAL.as_bytes().to_vec();
        replacement[13] = byte;
        assert_matches_legacy(String::from_utf8(replacement).unwrap());

        let mut first = CANONICAL.as_bytes().to_vec();
        first[0] = byte;
        assert_matches_legacy(String::from_utf8(first).unwrap());

        let mut appended = CANONICAL.to_string();
        appended.push(byte as char);
        assert_matches_legacy(appended);
    }
}

#[test]
fn decoder_matches_original_behavior_for_unicode_and_friendly_text() {
    const CANONICAL: &str = "01K2R7KFWE5807000000000001";

    for character in ['é', '\u{00a0}', '\u{2003}', '中', '💾'] {
        let input = format!("{}{}{}", &CANONICAL[..13], character, &CANONICAL[13..]);
        assert_matches_legacy(input);
    }

    for input in [
        CANONICAL.to_lowercase(),
        format!("{}-{}", &CANONICAL[..13], &CANONICAL[13..]),
        format!("{}_{}", &CANONICAL[..13], &CANONICAL[13..]),
        format!("{} {}", &CANONICAL[..13], &CANONICAL[13..]),
        format!("8{}?", &CANONICAL[1..]),
        format!("8{}u", &CANONICAL[1..]),
    ] {
        assert_matches_legacy(input);
    }
}

fn assert_matches_legacy(input: String) {
    assert_eq!(
        decode_text(&input),
        legacy_decode(&input),
        "input {input:?}"
    );
}

fn legacy_encode(bytes: &[u8; BYTE_LENGTH]) -> String {
    let value = u128::from_be_bytes(*bytes);
    let mut out = String::with_capacity(STRING_LENGTH);
    for index in (0..STRING_LENGTH).rev() {
        let symbol = ((value >> (index * 5)) & 0x1f) as usize;
        out.push(ALPHABET[symbol] as char);
    }
    out
}

fn legacy_decode(input: &str) -> Result<ZLID> {
    let mut normalized = Vec::with_capacity(STRING_LENGTH);
    for ch in input.chars() {
        if matches!(ch, '-' | '_' | ' ') {
            continue;
        }
        let upper = ch.to_ascii_uppercase();
        match upper {
            'U' => return Err(Error::InvalidText("U is forbidden".to_string())),
            'I' | 'L' => normalized.push(1),
            'O' => normalized.push(0),
            _ => {
                if !upper.is_ascii() {
                    return Err(Error::InvalidText(format!(
                        "invalid non-ASCII alphabet character {upper:?}"
                    )));
                }
                let value = ALPHABET
                    .iter()
                    .position(|candidate| *candidate as char == upper)
                    .ok_or_else(|| {
                        Error::InvalidText(format!("invalid alphabet character {upper:?}"))
                    })?;
                normalized.push(value as u8);
            }
        }
    }

    if normalized.len() != STRING_LENGTH {
        return Err(Error::InvalidText(format!(
            "invalid normalized length ({})",
            normalized.len()
        )));
    }
    if normalized[0] > 7 {
        return Err(Error::InvalidText(
            "first character outside 0..7".to_string(),
        ));
    }

    let mut value = 0u128;
    for symbol in normalized {
        value = (value << 5) | u128::from(symbol);
    }
    Ok(ZLID(value.to_be_bytes()))
}
