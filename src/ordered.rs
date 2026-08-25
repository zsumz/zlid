use std::collections::HashMap;

use crate::clock::{Clock, SystemClock};
use crate::constants::{BYTE_LENGTH, MAX_TS};
use crate::error::{Error, Result};
use crate::ordered_types::{ClockState, OrderedEvent, OrderedFields};
use crate::profile::{max_value_for_bits, sequence_bits, Profile};
use crate::random::{random_value, EntropySource, SystemEntropy};
use crate::ZLID;

/// Explicit ordered generator.
pub struct OrderedGenerator<C = SystemClock, E = SystemEntropy> {
    core: OrderedGeneratorCore<C>,
    entropy: E,
}

impl OrderedGenerator<SystemClock, SystemEntropy> {
    /// Creates a generator for a profile and default partition.
    pub fn new(profile: Profile, default_partition: u8) -> Self {
        Self::with_sources(profile, default_partition, SystemClock, SystemEntropy)
    }

    /// Creates a generator for a profile with default partition `0`.
    pub fn with_profile(profile: Profile) -> Self {
        Self::new(profile, 0)
    }
}

impl Default for OrderedGenerator<SystemClock, SystemEntropy> {
    fn default() -> Self {
        Self::new(Profile::Default, 0)
    }
}

impl<C, E> OrderedGenerator<C, E>
where
    C: Clock,
    E: EntropySource,
{
    /// Creates a generator with injected clock and entropy sources.
    pub fn with_sources(profile: Profile, default_partition: u8, clock: C, entropy: E) -> Self {
        OrderedGenerator {
            core: OrderedGeneratorCore::new(profile, default_partition, clock),
            entropy,
        }
    }

    /// Returns the ordered layout profile.
    pub fn profile(&self) -> Profile {
        self.core.profile()
    }

    #[allow(clippy::should_implement_trait)]
    /// Emits the next ID for the generator's default partition.
    pub fn next(&mut self) -> Result<ZLID> {
        self.next_event_and_pack(None)
    }

    /// Emits the next ID for an explicit partition.
    pub fn next_with_partition(&mut self, partition: u8) -> Result<ZLID> {
        self.next_event_and_pack(Some(partition))
    }

    /// Emits the next deterministic event without drawing or packing entropy.
    pub fn next_event(&mut self, partition: Option<u8>) -> Result<OrderedEvent> {
        self.core.next(partition)
    }

    fn next_event_and_pack(&mut self, partition: Option<u8>) -> Result<ZLID> {
        let prepared = self.core.prepare_next(partition)?;
        let spec = self.core.profile().spec();
        let random_tail = random_value(&mut self.entropy, spec.rand_bits)?;
        self.core.pack_prepared(prepared, random_tail)
    }
}

/// Stateful ordered generation core. It emits deterministic events and does
/// not draw random bytes.
pub struct OrderedGeneratorCore<C = SystemClock> {
    profile: Profile,
    default_partition: u8,
    clock: C,
    state_by_partition: HashMap<u8, StreamState>,
}

#[derive(Debug, Copy, Clone)]
struct StreamState {
    last_ms: u64,
    sequence: u32,
}

struct PreparedEvent {
    event: OrderedEvent,
}

impl<C> OrderedGeneratorCore<C>
where
    C: Clock,
{
    /// Creates a deterministic generator core with an injected clock.
    pub fn new(profile: Profile, default_partition: u8, clock: C) -> Self {
        OrderedGeneratorCore {
            profile,
            default_partition,
            clock,
            state_by_partition: HashMap::new(),
        }
    }

    /// Returns the ordered layout profile.
    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// Emits the next deterministic event for an optional partition override.
    pub fn next(&mut self, partition: Option<u8>) -> Result<OrderedEvent> {
        let prepared = self.prepare_next(partition)?;
        Ok(self.commit(prepared))
    }

    pub(crate) fn next_with_random_tail(
        &mut self,
        partition: Option<u8>,
        random_tail: u64,
    ) -> Result<ZLID> {
        let prepared = self.prepare_next(partition)?;
        self.pack_prepared(prepared, random_tail)
    }

    fn prepare_next(&mut self, partition: Option<u8>) -> Result<PreparedEvent> {
        let partition = partition.unwrap_or(self.default_partition);
        let now_ms = self.clock.now_ms()?;
        if now_ms > MAX_TS {
            return Err(Error::Clock(
                "clock produced timestamp outside 48-bit range".to_string(),
            ));
        }

        let spec = self.profile.spec();
        let state = self.state_by_partition.get(&partition).copied();
        let (mut timestamp_ms, tag) = match state {
            None => (now_ms, spec.normal_tag),
            Some(previous) if now_ms < previous.last_ms => (previous.last_ms, spec.clamped_tag),
            Some(_) => (now_ms, spec.normal_tag),
        };

        let sequence = match state {
            Some(previous) if timestamp_ms == previous.last_ms => {
                let next_sequence = previous.sequence + 1;
                if next_sequence > spec.seq_max {
                    timestamp_ms = previous.last_ms.checked_add(1).ok_or_else(|| {
                        Error::Clock("generated timestamp outside 48-bit range".to_string())
                    })?;
                    0
                } else {
                    next_sequence
                }
            }
            _ => 0,
        };

        if timestamp_ms > MAX_TS {
            return Err(Error::Clock(
                "generated timestamp outside 48-bit range".to_string(),
            ));
        }

        Ok(PreparedEvent {
            event: OrderedEvent {
                timestamp_ms,
                partition,
                sequence,
                tag,
            },
        })
    }

    fn commit(&mut self, prepared: PreparedEvent) -> OrderedEvent {
        let event = prepared.event;
        self.state_by_partition.insert(
            event.partition,
            StreamState {
                last_ms: event.timestamp_ms,
                sequence: event.sequence,
            },
        );
        event
    }

    fn pack_prepared(&mut self, prepared: PreparedEvent, random_tail: u64) -> Result<ZLID> {
        let event = prepared.event;
        let bytes = pack_ordered(
            self.profile,
            event.timestamp_ms,
            event.partition,
            event.sequence,
            random_tail,
            event.tag,
        )?;
        self.commit(prepared);
        Ok(ZLID(bytes))
    }
}

/// Packs ordered fields into a 16-byte payload.
pub fn pack_ordered(
    profile: Profile,
    timestamp_ms: u64,
    partition: u8,
    sequence: u32,
    random_tail: u64,
    tag: u8,
) -> Result<[u8; BYTE_LENGTH]> {
    let spec = profile.spec();
    if timestamp_ms > MAX_TS {
        return Err(Error::OutOfRange("ts_ms must be in 0..2^48-1"));
    }
    if sequence > spec.seq_max {
        return Err(Error::OutOfRange("sequence exceeds selected profile limit"));
    }
    if random_tail > max_value_for_bits(spec.rand_bits) {
        return Err(Error::OutOfRange(
            "random tail exceeds selected profile bit width",
        ));
    }
    if tag != spec.normal_tag && tag != spec.clamped_tag {
        return Err(Error::OutOfRange(
            "ordered tag does not match selected profile",
        ));
    }

    let value = (u128::from(timestamp_ms) << 80)
        | (u128::from(partition) << 72)
        | (u128::from(sequence) << spec.sequence_shift)
        | (u128::from(random_tail) << 4)
        | u128::from(tag);
    Ok(value.to_be_bytes())
}

/// Unpacks an ordered payload.
pub fn unpack_ordered(bytes: &[u8; BYTE_LENGTH]) -> Result<OrderedFields> {
    let value = u128::from_be_bytes(*bytes);
    let tag = (value & 0x0f) as u8;
    let profile = ordered_profile_from_tag(tag)
        .ok_or(Error::InvalidFamily("input is not an ordered ZLID"))?;
    let spec = profile.spec();
    let sequence_mask = (1u128 << sequence_bits(profile)) - 1;
    let random_mask = (1u128 << spec.rand_bits) - 1;
    Ok(OrderedFields {
        profile,
        timestamp_ms: ((value >> 80) & u128::from(MAX_TS)) as u64,
        partition: ((value >> 72) & 0xff) as u8,
        sequence: ((value >> spec.sequence_shift) & sequence_mask) as u32,
        random_tail: ((value >> 4) & random_mask) as u64,
        tag,
        clock_state: if tag == spec.clamped_tag {
            ClockState::Clamped
        } else {
            ClockState::Normal
        },
    })
}

pub(crate) fn format_random_tail(profile: Profile, random_tail: u64) -> String {
    let width = profile.spec().random_hex_width;
    format!("{random_tail:0width$X}")
}

pub(crate) fn ordered_profile_from_tag(tag: u8) -> Option<Profile> {
    use crate::constants::{
        TAG_ZLID_DEFAULT_CLAMPED, TAG_ZLID_DEFAULT_NORMAL, TAG_ZLID_HIGH_THROUGHPUT_CLAMPED,
        TAG_ZLID_HIGH_THROUGHPUT_NORMAL,
    };

    match tag {
        TAG_ZLID_DEFAULT_NORMAL | TAG_ZLID_DEFAULT_CLAMPED => Some(Profile::Default),
        TAG_ZLID_HIGH_THROUGHPUT_NORMAL | TAG_ZLID_HIGH_THROUGHPUT_CLAMPED => {
            Some(Profile::HighThroughput)
        }
        _ => None,
    }
}
