use crate::error::{Error, Result};

/// Converts bytes to uppercase hexadecimal.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Parses an even-length hexadecimal string.
pub fn bytes_from_hex(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err(Error::InvalidText(
            "hex input must have an even number of characters".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_value(bytes[index]).ok_or_else(|| {
            Error::InvalidText(format!("invalid hex character {:?}", bytes[index] as char))
        })?;
        let low = hex_value(bytes[index + 1]).ok_or_else(|| {
            Error::InvalidText(format!(
                "invalid hex character {:?}",
                bytes[index + 1] as char
            ))
        })?;
        out.push((high << 4) | low);
        index += 2;
    }
    Ok(out)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
