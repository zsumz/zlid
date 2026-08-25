use std::fmt;

/// Convenient result type for ZLID operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the ZLID SDK.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Text could not be decoded as a ZLID or related input.
    InvalidText(String),
    /// A numeric input exceeded the wire-format field width.
    OutOfRange(&'static str),
    /// An operation was not valid for the identifier family.
    InvalidFamily(&'static str),
    /// Operating-system or injected entropy was unavailable.
    Random(String),
    /// The clock or ordered-generator state could not produce a timestamp.
    Clock(String),
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
            Error::OutOfRange(message) => write!(f, "{message}"),
            Error::InvalidFamily(message) => write!(f, "{message}"),
            Error::Random(message) => write!(f, "random source error: {message}"),
            Error::Clock(message) => write!(f, "clock error: {message}"),
        }
    }
}

impl std::error::Error for Error {}
