use crate::constants::{ALPHABET, BYTE_LENGTH, STRING_LENGTH};
use crate::error::{Error, Result};
use crate::Zlid;

pub(crate) fn encode_text(bytes: &[u8; BYTE_LENGTH]) -> String {
    let value = u128::from_be_bytes(*bytes);
    let mut out = String::with_capacity(STRING_LENGTH);
    for index in (0..STRING_LENGTH).rev() {
        let symbol = ((value >> (index * 5)) & 0x1f) as usize;
        out.push(ALPHABET[symbol] as char);
    }
    out
}

pub(crate) fn decode_text(input: &str) -> Result<Zlid> {
    let mut normalized = Vec::with_capacity(STRING_LENGTH);
    for ch in input.chars() {
        if ch == '-' || ch == '_' || ch.is_whitespace() {
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
                let value = alphabet_value(upper).ok_or_else(|| {
                    Error::InvalidText(format!("invalid alphabet character {upper:?}"))
                })?;
                normalized.push(value);
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
    Ok(Zlid(value.to_be_bytes()))
}

fn alphabet_value(ch: char) -> Option<u8> {
    ALPHABET
        .iter()
        .position(|candidate| *candidate as char == ch)
        .map(|index| index as u8)
}
