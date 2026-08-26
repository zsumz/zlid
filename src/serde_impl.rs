use std::fmt;

use serde::de::{Error as _, IgnoredAny, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::constants::BYTE_LENGTH;
use crate::{text, ZLID};

impl Serialize for ZLID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let encoded = text::encode_text_array(&self.0);
            serializer.serialize_str(text::encoded_text_str(&encoded))
        } else {
            serialize_binary(self, serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ZLID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(CanonicalTextVisitor)
        } else {
            deserializer.deserialize_tuple(BYTE_LENGTH, BinaryVisitor)
        }
    }
}

fn serialize_binary<S>(zlid: &ZLID, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(BYTE_LENGTH)?;
    for byte in &zlid.0 {
        tuple.serialize_element(byte)?;
    }
    tuple.end()
}

struct CanonicalTextVisitor;

impl Visitor<'_> for CanonicalTextVisitor {
    type Value = ZLID;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical 26-character ZLID string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        ZLID::parse_canonical(value).map_err(E::custom)
    }
}

struct BinaryVisitor;

impl<'de> Visitor<'de> for BinaryVisitor {
    type Value = ZLID;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of exactly 16 bytes")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut bytes = [0u8; BYTE_LENGTH];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = sequence
                .next_element()?
                .ok_or_else(|| A::Error::invalid_length(index, &self))?;
        }
        if sequence.next_element::<IgnoredAny>()?.is_some() {
            return Err(A::Error::invalid_length(BYTE_LENGTH + 1, &self));
        }
        Ok(ZLID::from_array(bytes))
    }
}

#[cfg(test)]
#[path = "serde_tests.rs"]
mod tests;
