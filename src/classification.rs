use crate::constants::{
    BYTE_LENGTH, TAG_ZLID_ALIAS_DEFAULT_CLAMPED, TAG_ZLID_ALIAS_DEFAULT_NORMAL,
    TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED, TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL,
    TAG_ZLID_DEFAULT_CLAMPED, TAG_ZLID_DEFAULT_NORMAL, TAG_ZLID_HIGH_THROUGHPUT_CLAMPED,
    TAG_ZLID_HIGH_THROUGHPUT_NORMAL, TAG_ZLID_RANDOM,
};
use crate::inspection::InspectionKind;

/// A recognized ZLID wire family.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Family {
    /// A time-sortable ZLID.
    Ordered,
    /// A random ZLID-R.
    Random,
    /// A reversible ZLID-A alias.
    Alias,
}

impl Family {
    /// Returns the stable family name used by the specification.
    pub fn wire_name(self) -> &'static str {
        match self {
            Family::Ordered => "ZLID",
            Family::Random => "ZLID-R",
            Family::Alias => "ZLID-A",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Classification {
    pub(crate) kind: InspectionKind,
    pub(crate) family: Option<Family>,
}

pub(crate) fn classify_bytes(bytes: &[u8; BYTE_LENGTH]) -> Classification {
    if bytes == &[0; BYTE_LENGTH] || bytes == &[u8::MAX; BYTE_LENGTH] {
        return Classification {
            kind: InspectionKind::Sentinel,
            family: None,
        };
    }

    match bytes[BYTE_LENGTH - 1] & 0x0f {
        TAG_ZLID_DEFAULT_NORMAL
        | TAG_ZLID_HIGH_THROUGHPUT_NORMAL
        | TAG_ZLID_DEFAULT_CLAMPED
        | TAG_ZLID_HIGH_THROUGHPUT_CLAMPED => Classification {
            kind: InspectionKind::Ordered,
            family: Some(Family::Ordered),
        },
        TAG_ZLID_RANDOM => Classification {
            kind: InspectionKind::Random,
            family: Some(Family::Random),
        },
        TAG_ZLID_ALIAS_DEFAULT_NORMAL
        | TAG_ZLID_ALIAS_DEFAULT_CLAMPED
        | TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL
        | TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED => Classification {
            kind: InspectionKind::Alias,
            family: Some(Family::Alias),
        },
        _ => Classification {
            kind: InspectionKind::Opaque,
            family: None,
        },
    }
}

#[cfg(test)]
#[path = "classification_tests.rs"]
mod tests;
