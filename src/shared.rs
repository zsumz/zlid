use std::sync::Mutex;

use crate::clock::SystemClock;
use crate::error::{Error, Result};
use crate::ordered::OrderedGeneratorCore;
use crate::profile::Profile;
use crate::random::{random_value, EntropySource, SystemEntropy};
use crate::ZLID;

pub(crate) struct SharedOrderedGenerator(Mutex<OrderedGeneratorCore>);

impl SharedOrderedGenerator {
    pub(crate) fn new() -> Self {
        Self(Mutex::new(OrderedGeneratorCore::new(
            Profile::Default,
            0,
            SystemClock,
        )))
    }

    pub(crate) fn next_with_partition(&self, partition: u8) -> Result<ZLID> {
        let mut entropy = SystemEntropy;
        self.next_with_entropy(partition, &mut entropy)
    }

    fn next_with_entropy<E: EntropySource>(&self, partition: u8, entropy: &mut E) -> Result<ZLID> {
        let random_tail = random_value(entropy, Profile::Default.spec().rand_bits)?;
        let mut core = self
            .0
            .lock()
            .map_err(|_| Error::Clock("default generator lock is poisoned".to_string()))?;
        core.next_with_random_tail(Some(partition), random_tail)
    }
}

#[cfg(test)]
#[path = "shared_tests.rs"]
mod tests;
