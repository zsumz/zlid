use crate::alias::alias_source_from_tag;
use crate::bytes::bytes_to_hex;
use crate::classification::Family;
use crate::constants::{BYTE_LENGTH, TAG_ZLID_RANDOM};
use crate::ordered::{format_random_tail, unpack_ordered};
use crate::ordered_types::ClockState;
use crate::profile::Profile;
use crate::text::encode_text;

/// Semantic kind reported by inspection.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InspectionKind {
    /// A time-sortable ZLID.
    Ordered,
    /// A random ZLID-R.
    Random,
    /// A reversible ZLID-A alias.
    Alias,
    /// The reserved NIL or MAX value.
    Sentinel,
    /// A valid 128-bit payload whose tag is not a known family.
    Opaque,
}

impl InspectionKind {
    /// Returns the stable lowercase name used by the specification.
    pub fn wire_name(self) -> &'static str {
        match self {
            InspectionKind::Ordered => "ordered",
            InspectionKind::Random => "random",
            InspectionKind::Alias => "alias",
            InspectionKind::Sentinel => "sentinel",
            InspectionKind::Opaque => "opaque",
        }
    }
}

/// Sentinel name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SentinelName {
    /// The all-zero NIL sentinel.
    Nil,
    /// The all-one MAX sentinel.
    Max,
}

impl SentinelName {
    /// Returns the stable uppercase name used by the specification.
    pub fn wire_name(self) -> &'static str {
        match self {
            SentinelName::Nil => "NIL",
            SentinelName::Max => "MAX",
        }
    }
}

/// Structured inspection output.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Inspection {
    /// Decoded fields from a time-sortable ZLID.
    #[non_exhaustive]
    Ordered {
        /// Canonical 26-character representation.
        text: String,
        /// Uppercase hexadecimal representation of all 16 bytes.
        bytes_hex: String,
        /// Low wire tag nibble.
        tag: u8,
        /// Ordered layout profile.
        profile: Profile,
        /// Whether the source clock moved normally or was clamped.
        clock_state: ClockState,
        /// Milliseconds since the Unix epoch.
        timestamp_ms: u64,
        /// Application partition byte.
        partition: u8,
        /// Per-stream sequence value.
        sequence: u32,
        /// Random tail encoded as uppercase hexadecimal.
        random_hex: String,
    },
    /// Decoded fields from a random ZLID-R.
    #[non_exhaustive]
    Random {
        /// Canonical 26-character representation.
        text: String,
        /// Uppercase hexadecimal representation of all 16 bytes.
        bytes_hex: String,
        /// Low wire tag nibble.
        tag: u8,
        /// Random payload encoded as uppercase hexadecimal.
        random_hex: String,
    },
    /// Decoded fields from a reversible ZLID-A alias.
    #[non_exhaustive]
    Alias {
        /// Canonical 26-character representation.
        text: String,
        /// Uppercase hexadecimal representation of all 16 bytes.
        bytes_hex: String,
        /// Low wire tag nibble.
        tag: u8,
        /// Profile of the ordered source value.
        source_profile: Profile,
        /// Clock state of the ordered source value.
        source_clock_state: ClockState,
        /// Permuted alias payload encoded as uppercase hexadecimal.
        alias_data_hex: String,
    },
    /// A reserved boundary value.
    #[non_exhaustive]
    Sentinel {
        /// Canonical 26-character representation.
        text: String,
        /// Uppercase hexadecimal representation of all 16 bytes.
        bytes_hex: String,
        /// Low wire tag nibble.
        tag: u8,
        /// Which reserved boundary value was found.
        name: SentinelName,
    },
    /// A well-formed payload with an unknown family tag.
    #[non_exhaustive]
    Opaque {
        /// Canonical 26-character representation.
        text: String,
        /// Uppercase hexadecimal representation of all 16 bytes.
        bytes_hex: String,
        /// Low wire tag nibble.
        tag: u8,
    },
}

impl Inspection {
    /// Returns the semantic category of this inspection result.
    pub fn kind(&self) -> InspectionKind {
        match self {
            Inspection::Ordered { .. } => InspectionKind::Ordered,
            Inspection::Random { .. } => InspectionKind::Random,
            Inspection::Alias { .. } => InspectionKind::Alias,
            Inspection::Sentinel { .. } => InspectionKind::Sentinel,
            Inspection::Opaque { .. } => InspectionKind::Opaque,
        }
    }

    /// Returns the cached canonical text representation.
    pub fn text(&self) -> &str {
        match self {
            Inspection::Ordered { text, .. }
            | Inspection::Random { text, .. }
            | Inspection::Alias { text, .. }
            | Inspection::Sentinel { text, .. }
            | Inspection::Opaque { text, .. } => text,
        }
    }

    /// Returns the cached uppercase hexadecimal bytes.
    pub fn bytes_hex(&self) -> &str {
        match self {
            Inspection::Ordered { bytes_hex, .. }
            | Inspection::Random { bytes_hex, .. }
            | Inspection::Alias { bytes_hex, .. }
            | Inspection::Sentinel { bytes_hex, .. }
            | Inspection::Opaque { bytes_hex, .. } => bytes_hex,
        }
    }

    /// Returns the low wire tag nibble.
    pub fn tag(&self) -> u8 {
        match self {
            Inspection::Ordered { tag, .. }
            | Inspection::Random { tag, .. }
            | Inspection::Alias { tag, .. }
            | Inspection::Sentinel { tag, .. }
            | Inspection::Opaque { tag, .. } => *tag,
        }
    }

    /// Returns the specification family when the tag is known.
    pub fn family(&self) -> Option<Family> {
        match self {
            Inspection::Ordered { .. } => Some(Family::Ordered),
            Inspection::Random { .. } => Some(Family::Random),
            Inspection::Alias { .. } => Some(Family::Alias),
            Inspection::Sentinel { .. } | Inspection::Opaque { .. } => None,
        }
    }

    /// Returns whether this is the NIL or MAX sentinel.
    pub fn is_sentinel(&self) -> bool {
        matches!(self, Inspection::Sentinel { .. })
    }

    /// Returns whether this is an ordered, random, or alias family.
    pub fn is_known_family(&self) -> bool {
        matches!(
            self,
            Inspection::Ordered { .. } | Inspection::Random { .. } | Inspection::Alias { .. }
        )
    }
}

pub(crate) fn inspect_bytes(bytes: [u8; BYTE_LENGTH]) -> Inspection {
    let text = encode_text(&bytes);
    let bytes_hex = bytes_to_hex(&bytes);
    let tag = bytes[15] & 0x0f;

    if bytes == [0u8; BYTE_LENGTH] {
        return Inspection::Sentinel {
            text,
            bytes_hex,
            tag,
            name: SentinelName::Nil,
        };
    }
    if bytes == [0xff; BYTE_LENGTH] {
        return Inspection::Sentinel {
            text,
            bytes_hex,
            tag,
            name: SentinelName::Max,
        };
    }

    if tag == TAG_ZLID_RANDOM {
        let random_hex = bytes_hex[..31].to_string();
        return Inspection::Random {
            text,
            bytes_hex,
            tag,
            random_hex,
        };
    }

    if let Some((source_profile, source_clock_state)) = alias_source_from_tag(tag) {
        let alias_data_hex = bytes_hex[..31].to_string();
        return Inspection::Alias {
            text,
            bytes_hex,
            tag,
            source_profile,
            source_clock_state,
            alias_data_hex,
        };
    }

    match unpack_ordered(&bytes) {
        Ok(fields) => Inspection::Ordered {
            text,
            bytes_hex,
            tag: fields.tag,
            profile: fields.profile,
            clock_state: fields.clock_state,
            timestamp_ms: fields.timestamp_ms,
            partition: fields.partition,
            sequence: fields.sequence,
            random_hex: format_random_tail(fields.profile, fields.random_tail),
        },
        Err(_) => Inspection::Opaque {
            text,
            bytes_hex,
            tag,
        },
    }
}

#[cfg(test)]
#[path = "inspection_tests.rs"]
mod tests;
