//! Injectable clock, entropy, and deterministic generator-core seams.

pub use crate::clock::{Clock, SystemClock};
pub use crate::ordered::OrderedGeneratorCore;
pub use crate::ordered_types::OrderedEvent;
pub use crate::random::{EntropySource, SystemEntropy};
