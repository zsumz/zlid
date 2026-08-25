use std::fmt;

/// Convenient result type for ZLID operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the ZLID SDK.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// An input or generated value had the wrong byte length.
    InvalidLength {
        /// Human-readable name of the value being measured.
        what: &'static str,
        /// Required length in bytes.
        expected: usize,
        /// Observed length in bytes.
        actual: usize,
    },
    /// Text could not be decoded as a ZLID.
    InvalidText(String),
    /// A profile name is not defined by this version of the specification.
    UnknownProfile(String),
    /// ZLID-A requires a non-empty secret key.
    EmptyAliasKey,
    /// A ZLID-A tweak exceeded the wire-format length limit.
    TweakTooLong {
        /// Maximum accepted length in bytes.
        maximum: usize,
        /// Observed length in bytes.
        actual: usize,
    },
    /// An operation rejected a low wire tag nibble.
    InvalidTag {
        /// Operation that rejected the tag.
        operation: &'static str,
        /// Human-readable description of accepted tags.
        expected: &'static str,
        /// Observed low wire tag nibble.
        actual: u8,
    },
    /// A numeric input exceeded its wire-format field width.
    FieldOutOfRange {
        /// Stable field name from the specification.
        field: &'static str,
        /// Maximum accepted value.
        maximum: u64,
        /// Observed value.
        actual: u64,
    },
    /// Operating-system or injected entropy was unavailable.
    EntropyUnavailable(String),
    /// The clock could not produce a valid timestamp.
    Clock(String),
    /// The shared generator mutex was poisoned by a panic.
    GeneratorPoisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidLength {
                what,
                expected,
                actual,
            } => write!(f, "{what} must be exactly {expected} bytes, got {actual}"),
            Error::InvalidText(message) => write!(f, "invalid ZLID text: {message}"),
            Error::UnknownProfile(profile) => write!(f, "unknown ZLID profile {profile:?}"),
            Error::EmptyAliasKey => write!(f, "alias key must not be empty"),
            Error::TweakTooLong { maximum, actual } => {
                write!(
                    f,
                    "alias tweak must be at most {maximum} bytes, got {actual}"
                )
            }
            Error::InvalidTag {
                operation,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "{operation} expected {expected}, got ZLID tag 0x{actual:X}"
                )
            }
            Error::FieldOutOfRange {
                field,
                maximum,
                actual,
            } => {
                write!(
                    f,
                    "ZLID field {field} must be at most {maximum}, got {actual}"
                )
            }
            Error::EntropyUnavailable(message) => write!(f, "entropy unavailable: {message}"),
            Error::Clock(message) => write!(f, "clock error: {message}"),
            Error::GeneratorPoisoned => write!(f, "shared ZLID generator mutex is poisoned"),
        }
    }
}

impl std::error::Error for Error {}
