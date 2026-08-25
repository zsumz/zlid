use crate::constants::{ALPHABET, BYTE_LENGTH, STRING_LENGTH};
use crate::error::{Error, Result};
use crate::ZLID;

const INVALID_SYMBOL: u8 = u8::MAX;
const DECODE_TABLE: [u8; 128] = make_decode_table();

pub(crate) fn encode_text(bytes: &[u8; BYTE_LENGTH]) -> String {
    let encoded = encode_text_array(bytes);
    encoded_text_str(&encoded).to_owned()
}

pub(crate) fn encode_text_array(bytes: &[u8; BYTE_LENGTH]) -> [u8; STRING_LENGTH] {
    let value = u128::from_be_bytes(*bytes);
    let mut out = [0u8; STRING_LENGTH];
    for (offset, index) in (0..STRING_LENGTH).rev().enumerate() {
        let symbol = ((value >> (index * 5)) & 0x1f) as usize;
        out[offset] = ALPHABET[symbol];
    }
    out
}

pub(crate) fn encoded_text_str(encoded: &[u8; STRING_LENGTH]) -> &str {
    std::str::from_utf8(encoded).expect("the ZLID alphabet contains only ASCII")
}

pub(crate) fn decode_text(input: &str) -> Result<ZLID> {
    let mut normalized_length = 0usize;
    let mut first_symbol = None;
    let mut value = 0u128;

    for (offset, byte) in input.bytes().enumerate() {
        if matches!(byte, b'-' | b'_' | b' ') {
            continue;
        }
        if matches!(byte, b'U' | b'u') {
            return Err(Error::InvalidText("U is forbidden".to_string()));
        }
        if !byte.is_ascii() {
            let character = input[offset..]
                .chars()
                .next()
                .expect("the byte belongs to the input string");
            return Err(Error::InvalidText(format!(
                "invalid non-ASCII alphabet character {character:?}"
            )));
        }

        let symbol = decode_symbol(byte).ok_or_else(|| {
            let character = (byte as char).to_ascii_uppercase();
            Error::InvalidText(format!("invalid alphabet character {character:?}"))
        })?;
        first_symbol.get_or_insert(symbol);
        value = (value << 5) | u128::from(symbol);
        normalized_length += 1;
    }

    if normalized_length != STRING_LENGTH {
        return Err(Error::InvalidText(format!(
            "invalid normalized length ({normalized_length})"
        )));
    }
    if first_symbol.is_some_and(|symbol| symbol > 7) {
        return Err(Error::InvalidText(
            "first character outside 0..7".to_string(),
        ));
    }

    Ok(ZLID(value.to_be_bytes()))
}

fn decode_symbol(byte: u8) -> Option<u8> {
    let symbol = DECODE_TABLE[usize::from(byte)];
    (symbol != INVALID_SYMBOL).then_some(symbol)
}

const fn make_decode_table() -> [u8; 128] {
    let mut table = [INVALID_SYMBOL; 128];
    let mut index = 0usize;
    while index < ALPHABET.len() {
        let uppercase = ALPHABET[index] as usize;
        table[uppercase] = index as u8;
        if uppercase >= b'A' as usize {
            table[uppercase + 32] = index as u8;
        }
        index += 1;
    }

    table[b'I' as usize] = 1;
    table[b'i' as usize] = 1;
    table[b'L' as usize] = 1;
    table[b'l' as usize] = 1;
    table[b'O' as usize] = 0;
    table[b'o' as usize] = 0;
    table
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
