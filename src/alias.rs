use crate::constants::{
    MASK_124, MASK_62, TAG_ZLID_ALIAS_DEFAULT_CLAMPED, TAG_ZLID_ALIAS_DEFAULT_NORMAL,
    TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED, TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL,
    TAG_ZLID_DEFAULT_CLAMPED, TAG_ZLID_DEFAULT_NORMAL, TAG_ZLID_HIGH_THROUGHPUT_CLAMPED,
    TAG_ZLID_HIGH_THROUGHPUT_NORMAL,
};
use crate::crypto::HmacSha256;
use crate::error::{Error, Result};
use crate::ordered_types::ClockState;
use crate::profile::Profile;
use crate::ZLID;

pub(crate) fn alias_zlid(source: ZLID, key: &[u8], tweak: &[u8]) -> Result<ZLID> {
    let key = normalize_alias_key(key)?;
    let tweak = normalize_tweak(tweak)?;
    let alias_tag = source_tag_to_alias_tag(source.tag())?;
    let hmac = HmacSha256::new(key);
    let source_data = (u128::from_be_bytes(source.0) >> 4) & MASK_124;
    let alias_data = permute124(&hmac, tweak, source_data)?;
    Ok(ZLID(
        ((alias_data << 4) | u128::from(alias_tag)).to_be_bytes(),
    ))
}

pub(crate) fn unalias_zlid(alias: ZLID, key: &[u8], tweak: &[u8]) -> Result<ZLID> {
    let key = normalize_alias_key(key)?;
    let tweak = normalize_tweak(tweak)?;
    let source_tag = alias_tag_to_source_tag(alias.tag())?;
    let hmac = HmacSha256::new(key);
    let alias_data = (u128::from_be_bytes(alias.0) >> 4) & MASK_124;
    let source_data = inverse_permute124(&hmac, tweak, alias_data)?;
    Ok(ZLID(
        ((source_data << 4) | u128::from(source_tag)).to_be_bytes(),
    ))
}

pub(crate) fn alias_source_from_tag(tag: u8) -> Option<(Profile, ClockState)> {
    match tag {
        TAG_ZLID_ALIAS_DEFAULT_NORMAL => Some((Profile::Default, ClockState::Normal)),
        TAG_ZLID_ALIAS_DEFAULT_CLAMPED => Some((Profile::Default, ClockState::Clamped)),
        TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL => {
            Some((Profile::HighThroughput, ClockState::Normal))
        }
        TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED => {
            Some((Profile::HighThroughput, ClockState::Clamped))
        }
        _ => None,
    }
}

fn normalize_alias_key(key: &[u8]) -> Result<&[u8]> {
    if key.is_empty() {
        return Err(Error::InvalidFamily("alias key must not be empty"));
    }
    Ok(key)
}

fn normalize_tweak(tweak: &[u8]) -> Result<&[u8]> {
    if tweak.len() > 0xffff {
        return Err(Error::InvalidFamily("tweak must be at most 65535 bytes"));
    }
    Ok(tweak)
}

fn source_tag_to_alias_tag(tag: u8) -> Result<u8> {
    match tag {
        TAG_ZLID_DEFAULT_NORMAL => Ok(TAG_ZLID_ALIAS_DEFAULT_NORMAL),
        TAG_ZLID_DEFAULT_CLAMPED => Ok(TAG_ZLID_ALIAS_DEFAULT_CLAMPED),
        TAG_ZLID_HIGH_THROUGHPUT_NORMAL => Ok(TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL),
        TAG_ZLID_HIGH_THROUGHPUT_CLAMPED => Ok(TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED),
        _ => Err(Error::InvalidFamily("only ordered ZLIDs can be aliased")),
    }
}

fn alias_tag_to_source_tag(tag: u8) -> Result<u8> {
    match tag {
        TAG_ZLID_ALIAS_DEFAULT_NORMAL => Ok(TAG_ZLID_DEFAULT_NORMAL),
        TAG_ZLID_ALIAS_DEFAULT_CLAMPED => Ok(TAG_ZLID_DEFAULT_CLAMPED),
        TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL => Ok(TAG_ZLID_HIGH_THROUGHPUT_NORMAL),
        TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED => Ok(TAG_ZLID_HIGH_THROUGHPUT_CLAMPED),
        _ => Err(Error::InvalidFamily("only ZLID-A values can be unaliased")),
    }
}

fn permute124(hmac: &HmacSha256, tweak: &[u8], data124: u128) -> Result<u128> {
    if data124 > MASK_124 {
        return Err(Error::OutOfRange("124-bit value out of range"));
    }
    let mut left = ((data124 >> 62) as u64) & MASK_62;
    let mut right = (data124 as u64) & MASK_62;
    for round in 0..8u8 {
        let next = (left ^ round_function(hmac, tweak, round, right)?) & MASK_62;
        left = right;
        right = next;
    }
    Ok((u128::from(left) << 62) | u128::from(right))
}

fn inverse_permute124(hmac: &HmacSha256, tweak: &[u8], data124: u128) -> Result<u128> {
    if data124 > MASK_124 {
        return Err(Error::OutOfRange("124-bit value out of range"));
    }
    let mut left = ((data124 >> 62) as u64) & MASK_62;
    let mut right = (data124 as u64) & MASK_62;
    for round in (0..8u8).rev() {
        let next_left = (right ^ round_function(hmac, tweak, round, left)?) & MASK_62;
        right = left;
        left = next_left;
    }
    Ok((u128::from(left) << 62) | u128::from(right))
}

fn round_function(hmac: &HmacSha256, tweak: &[u8], round: u8, right: u64) -> Result<u64> {
    if right > MASK_62 {
        return Err(Error::OutOfRange("62-bit value out of range"));
    }
    let round_byte = [round];
    let tweak_len = (tweak.len() as u16).to_be_bytes();
    let right_bytes = right.to_be_bytes();
    let digest = hmac.digest(&[b"ZLID-A-F", &round_byte, &tweak_len, tweak, &right_bytes]);
    let mut low = [0u8; 8];
    low.copy_from_slice(&digest[24..32]);
    Ok(u64::from_be_bytes(low) & MASK_62)
}
