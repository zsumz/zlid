use crate::profile::Profile;

/// Ordered clock state encoded in the tag.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ClockState {
    /// The observed clock did not move behind the stream state.
    Normal,
    /// The observed clock regressed and the prior timestamp was retained.
    Clamped,
}

impl ClockState {
    /// Returns the stable lowercase name used by the specification.
    pub fn wire_name(self) -> &'static str {
        match self {
            ClockState::Normal => "normal",
            ClockState::Clamped => "clamped",
        }
    }
}

/// A decoded ordered payload.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OrderedFields {
    /// Ordered layout profile.
    pub profile: Profile,
    /// Milliseconds since the Unix epoch.
    pub timestamp_ms: u64,
    /// Application partition byte.
    pub partition: u8,
    /// Per-stream sequence value.
    pub sequence: u32,
    /// Random tail value before tag packing.
    pub random_tail: u64,
    /// Low wire tag nibble.
    pub tag: u8,
    /// Whether the source clock moved normally or was clamped.
    pub clock_state: ClockState,
}

/// Generator event before the random tail is packed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OrderedEvent {
    /// Milliseconds since the Unix epoch.
    pub timestamp_ms: u64,
    /// Application partition byte.
    pub partition: u8,
    /// Per-stream sequence value.
    pub sequence: u32,
    /// Low wire tag nibble.
    pub tag: u8,
}
