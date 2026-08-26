use serde::de::value::{Error, SeqDeserializer};
use serde::de::Visitor;

use super::BinaryVisitor;
use crate::ZLID;

#[test]
fn binary_visitor_accepts_exactly_sixteen_tuple_elements() -> Result<(), String> {
    let bytes = [0x5a; ZLID::BYTE_LENGTH];
    let actual = visit(&bytes).map_err(|error| error.to_string())?;
    if actual.bytes() != bytes {
        return Err("binary visitor changed tuple bytes".to_string());
    }
    Ok(())
}

#[test]
fn binary_visitor_rejects_short_and_long_tuples() -> Result<(), String> {
    let short = [0u8; ZLID::BYTE_LENGTH - 1];
    let short_error = visit(&short)
        .err()
        .ok_or_else(|| "binary visitor accepted a short tuple".to_string())?
        .to_string();
    if !short_error.contains("invalid length 15") {
        return Err(short_error);
    }

    let long = [0u8; ZLID::BYTE_LENGTH + 1];
    let long_error = visit(&long)
        .err()
        .ok_or_else(|| "binary visitor accepted a long tuple".to_string())?
        .to_string();
    if !long_error.contains("invalid length 17") {
        return Err(long_error);
    }
    Ok(())
}

fn visit(bytes: &[u8]) -> Result<ZLID, Error> {
    let sequence = SeqDeserializer::<_, Error>::new(bytes.iter().copied());
    BinaryVisitor.visit_seq(sequence)
}
