use crate::constants::{
    TAG_ZLID_DEFAULT_CLAMPED, TAG_ZLID_DEFAULT_NORMAL, TAG_ZLID_HIGH_THROUGHPUT_CLAMPED,
    TAG_ZLID_HIGH_THROUGHPUT_NORMAL,
};
use crate::error::{Error, Result};

/// Ordered ZLID profile.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Profile {
    /// Twelve sequence bits and a 56-bit random tail.
    Default,
    /// Sixteen sequence bits and a 52-bit random tail.
    HighThroughput,
}

impl Profile {
    /// Returns the stable profile name used by the specification.
    pub fn wire_name(self) -> &'static str {
        match self {
            Profile::Default => "default",
            Profile::HighThroughput => "high-throughput",
        }
    }

    /// Parses a stable profile name from the specification.
    pub fn from_wire_name(value: &str) -> Result<Self> {
        match value {
            "default" => Ok(Profile::Default),
            "high-throughput" => Ok(Profile::HighThroughput),
            _ => Err(Error::UnknownProfile(value.to_string())),
        }
    }

    pub(crate) fn spec(self) -> ProfileSpec {
        match self {
            Profile::Default => ProfileSpec {
                seq_max: 4095,
                rand_bits: 56,
                normal_tag: TAG_ZLID_DEFAULT_NORMAL,
                clamped_tag: TAG_ZLID_DEFAULT_CLAMPED,
                random_hex_width: 14,
                sequence_shift: 60,
            },
            Profile::HighThroughput => ProfileSpec {
                seq_max: 65_535,
                rand_bits: 52,
                normal_tag: TAG_ZLID_HIGH_THROUGHPUT_NORMAL,
                clamped_tag: TAG_ZLID_HIGH_THROUGHPUT_CLAMPED,
                random_hex_width: 13,
                sequence_shift: 56,
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ProfileSpec {
    pub(crate) seq_max: u32,
    pub(crate) rand_bits: u8,
    pub(crate) normal_tag: u8,
    pub(crate) clamped_tag: u8,
    pub(crate) random_hex_width: usize,
    pub(crate) sequence_shift: u32,
}

pub(crate) fn sequence_bits(profile: Profile) -> u8 {
    match profile {
        Profile::Default => 12,
        Profile::HighThroughput => 16,
    }
}

pub(crate) fn max_value_for_bits(bits: u8) -> u64 {
    if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}
