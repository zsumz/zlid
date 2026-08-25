#![doc = include_str!("../README.md")]

use std::cmp::Ordering;
use std::fmt;
use std::sync::OnceLock;

mod alias;
mod bytes;
mod clock;
mod constants;
mod crypto;
mod error;
mod inspection;
mod ordered;
mod ordered_types;
mod partition;
mod profile;
mod random;
mod shared;
mod text;

use constants::BYTE_LENGTH;
use shared::SharedOrderedGenerator;

pub use bytes::{bytes_from_hex, bytes_to_hex};
pub use clock::{Clock, SystemClock};
pub use error::{Error, Result};
pub use inspection::{Inspection, InspectionKind, SentinelName};
pub use ordered::{pack_ordered, unpack_ordered, OrderedGenerator, OrderedGeneratorCore};
pub use ordered_types::{ClockState, OrderedEvent, OrderedFields};
pub use partition::{partition_bytes, partition_str};
pub use profile::Profile;
pub use random::{EntropySource, SystemEntropy};

/// Immutable 16-byte ZLID value.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ZLID(pub(crate) [u8; BYTE_LENGTH]);

/// Compatibility alias for the original Rust-style spelling.
///
/// New code should use [`ZLID`].
pub type Zlid = ZLID;

impl ZLID {
    /// Reserved NIL sentinel.
    pub const NIL: ZLID = ZLID([0; BYTE_LENGTH]);

    /// Reserved MAX sentinel.
    pub const MAX: ZLID = ZLID([0xff; BYTE_LENGTH]);

    /// Parses canonical or friendly ZLID text.
    pub fn parse(text: &str) -> Result<Self> {
        text::decode_text(text)
    }

    /// Creates a ZLID from exactly 16 bytes. The input is copied.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != BYTE_LENGTH {
            return Err(Error::InvalidLength {
                what: "ZLID",
                expected: BYTE_LENGTH,
                actual: bytes.len(),
            });
        }
        let mut raw = [0u8; BYTE_LENGTH];
        raw.copy_from_slice(bytes);
        Ok(ZLID(raw))
    }

    /// Creates a ZLID from an owned byte array.
    pub fn from_array(bytes: [u8; BYTE_LENGTH]) -> Self {
        ZLID(bytes)
    }

    /// Emits the next ordered ZLID from the shared default generator.
    pub fn next() -> Result<Self> {
        Self::next_with_partition(0)
    }

    /// Emits the next ordered ZLID from the shared default generator with a partition.
    pub fn next_with_partition(partition: u8) -> Result<Self> {
        DEFAULT_GENERATOR
            .get_or_init(SharedOrderedGenerator::new)
            .next_with_partition(partition)
    }

    /// Creates a default-profile explicit ordered generator with partition `0`.
    pub fn default_generator() -> OrderedGenerator {
        OrderedGenerator::default()
    }

    /// Creates an explicit ordered generator for a profile with partition `0`.
    pub fn generator_for_profile(profile: Profile) -> OrderedGenerator {
        OrderedGenerator::with_profile(profile)
    }

    /// Creates a new explicit ordered generator.
    pub fn generator(profile: Profile, default_partition: u8) -> OrderedGenerator {
        OrderedGenerator::new(profile, default_partition)
    }

    /// Emits a ZLID-R using the system entropy source.
    pub fn random() -> Result<Self> {
        let mut source = SystemEntropy;
        Self::random_with(&mut source)
    }

    /// Emits a ZLID-R using an injected entropy source.
    pub fn random_with<E: EntropySource>(source: &mut E) -> Result<Self> {
        random::random_zlid(source).map(ZLID)
    }

    /// Encodes an ordered ZLID as a deterministic ZLID-A alias.
    ///
    /// The non-empty key must be a high-entropy application secret. ZLID-A
    /// does not authenticate the result or detect a wrong key during decoding.
    pub fn alias(&self, key: &[u8], tweak: &[u8]) -> Result<Self> {
        alias::alias_zlid(*self, key, tweak)
    }

    /// Encodes an ordered ZLID as a deterministic ZLID-A alias with UTF-8 tweak text.
    pub fn alias_str(&self, key: &[u8], tweak: &str) -> Result<Self> {
        self.alias(key, tweak.as_bytes())
    }

    /// Decodes a ZLID-A alias back to the ordered source.
    ///
    /// Applications are responsible for selecting the same versioned key and
    /// tweak that were used to create the alias.
    pub fn unalias(&self, key: &[u8], tweak: &[u8]) -> Result<Self> {
        alias::unalias_zlid(*self, key, tweak)
    }

    /// Decodes a ZLID-A alias back to the ordered source with UTF-8 tweak text.
    pub fn unalias_str(&self, key: &[u8], tweak: &str) -> Result<Self> {
        self.unalias(key, tweak.as_bytes())
    }

    /// Returns a defensive copy of the raw bytes.
    pub fn bytes(&self) -> [u8; BYTE_LENGTH] {
        self.0
    }

    /// Returns an immutable byte view.
    pub fn as_bytes(&self) -> &[u8; BYTE_LENGTH] {
        &self.0
    }

    /// Returns canonical uppercase 26-character ZLID text.
    pub fn text(&self) -> String {
        text::encode_text(&self.0)
    }

    /// Returns uppercase hexadecimal bytes.
    pub fn bytes_hex(&self) -> String {
        bytes_to_hex(&self.0)
    }

    /// Returns the low tag nibble.
    pub fn tag(&self) -> u8 {
        self.0[15] & 0x0f
    }

    /// Inspects this payload without changing it.
    pub fn inspect(&self) -> Inspection {
        inspection::inspect_bytes(self.0)
    }

    /// Computes unsigned lexicographic byte-order comparison.
    pub fn compare(a: &ZLID, b: &ZLID) -> Ordering {
        a.cmp(b)
    }

    /// Computes SipHash-2-4(key16, input) & 0xff for byte input.
    /// An omitted key uses the public all-zero key.
    pub fn partition_bytes(input: &[u8], key: Option<&[u8]>) -> Result<u8> {
        partition_bytes(input, key)
    }

    /// Computes SipHash-2-4(key16, utf8(input)) & 0xff.
    /// An omitted key uses the public all-zero key.
    pub fn partition_str(input: &str, key: Option<&[u8]>) -> Result<u8> {
        partition_bytes(input.as_bytes(), key)
    }
}

impl fmt::Debug for ZLID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = text::encode_text_array(&self.0);
        f.debug_tuple("ZLID")
            .field(&text::encoded_text_str(&encoded))
            .finish()
    }
}

impl fmt::Display for ZLID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = text::encode_text_array(&self.0);
        f.write_str(text::encoded_text_str(&encoded))
    }
}

impl std::str::FromStr for ZLID {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ZLID::parse(s)
    }
}

impl Ord for ZLID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for ZLID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

static DEFAULT_GENERATOR: OnceLock<SharedOrderedGenerator> = OnceLock::new();
