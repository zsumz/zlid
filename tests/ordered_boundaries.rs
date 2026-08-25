//! Deterministic boundary tests for the ordered generator state machine.

use std::cell::Cell;
use std::rc::Rc;

use zlid::{
    unpack_ordered, Clock, EntropySource, Error, OrderedEvent, OrderedGenerator,
    OrderedGeneratorCore, Profile,
};

const MAX_TS: u64 = (1_u64 << 48) - 1;

struct CarryClock {
    calls: u32,
    sequence_max: u32,
    base_ms: u64,
}

impl Clock for CarryClock {
    fn now_ms(&mut self) -> zlid::Result<u64> {
        let now_ms = match self.calls {
            call if call <= self.sequence_max => self.base_ms,
            call if call == self.sequence_max + 1 => self.base_ms - 1,
            call if call == self.sequence_max + 2 => self.base_ms,
            call if call == self.sequence_max + 3 => self.base_ms + 1,
            _ => self.base_ms + 2,
        };
        self.calls += 1;
        Ok(now_ms)
    }
}

struct CountingEntropy {
    calls: Rc<Cell<usize>>,
}

impl EntropySource for CountingEntropy {
    fn fill_bytes(&mut self, out: &mut [u8]) -> zlid::Result<()> {
        self.calls.set(self.calls.get() + 1);
        out.fill(0);
        Ok(())
    }
}

#[test]
fn sequence_carry_preserves_clamped_tag_and_recovers_for_both_profiles() -> zlid::Result<()> {
    assert_carry_and_recovery(Profile::Default, 4_095, 1, 3)?;
    assert_carry_and_recovery(Profile::HighThroughput, 65_535, 2, 4)
}

#[test]
fn positive_partition_streams_advance_independently_when_interleaved() -> zlid::Result<()> {
    let times = [1_000, 1_000, 999, 1_000, 1_001];
    let mut index = 0;
    let clock = move || {
        let now_ms = times[index];
        index += 1;
        now_ms
    };
    let mut core = OrderedGeneratorCore::new(Profile::Default, 7, clock);

    assert_event(core.next(None)?, (1_000, 7, 0, 1));
    assert_event(core.next(Some(9))?, (1_000, 9, 0, 1));
    assert_event(core.next(None)?, (1_000, 7, 1, 3));
    assert_event(core.next(Some(9))?, (1_000, 9, 1, 1));
    assert_event(core.next(None)?, (1_001, 7, 0, 1));
    Ok(())
}

#[test]
fn max_timestamp_carry_is_repeatable_atomic_and_draws_no_entropy() -> zlid::Result<()> {
    assert_max_timestamp_exhaustion(Profile::Default, 4_095)?;
    assert_max_timestamp_exhaustion(Profile::HighThroughput, 65_535)?;

    let mut core = OrderedGeneratorCore::new(Profile::Default, 0, || MAX_TS + 1);
    assert_eq!(
        core.next(None),
        Err(Error::Clock(
            "clock produced timestamp outside 48-bit range".to_string()
        ))
    );
    Ok(())
}

fn assert_carry_and_recovery(
    profile: Profile,
    sequence_max: u32,
    normal_tag: u8,
    clamped_tag: u8,
) -> zlid::Result<()> {
    let base_ms = 50_000;
    let clock = CarryClock {
        calls: 0,
        sequence_max,
        base_ms,
    };
    let mut core = OrderedGeneratorCore::new(profile, 23, clock);

    for sequence in 0..=sequence_max {
        assert_event(core.next(None)?, (base_ms, 23, sequence, normal_tag));
    }

    assert_event(core.next(None)?, (base_ms + 1, 23, 0, clamped_tag));
    assert_event(core.next(None)?, (base_ms + 1, 23, 1, clamped_tag));
    assert_event(core.next(None)?, (base_ms + 1, 23, 2, normal_tag));
    assert_event(core.next(None)?, (base_ms + 2, 23, 0, normal_tag));
    Ok(())
}

fn assert_max_timestamp_exhaustion(profile: Profile, sequence_max: u32) -> zlid::Result<()> {
    let entropy_calls = Rc::new(Cell::new(0));
    let entropy = CountingEntropy {
        calls: Rc::clone(&entropy_calls),
    };
    let mut generator = OrderedGenerator::with_sources(profile, 41, || MAX_TS, entropy);

    let mut final_id = None;
    for _ in 0..=sequence_max {
        final_id = Some(generator.next()?);
    }
    let final_fields = unpack_ordered(final_id.expect("at least one ID").as_bytes())?;
    assert_eq!(
        (
            final_fields.timestamp_ms,
            final_fields.partition,
            final_fields.sequence
        ),
        (MAX_TS, 41, sequence_max)
    );
    let successful_draws = sequence_max as usize + 1;
    assert_eq!(entropy_calls.get(), successful_draws);

    assert_generated_range_error(generator.next());
    assert_eq!(entropy_calls.get(), successful_draws);
    assert_generated_range_error(generator.next());
    assert_eq!(entropy_calls.get(), successful_draws);

    let other_partition = generator.next_with_partition(42)?;
    let fields = unpack_ordered(other_partition.as_bytes())?;
    assert_eq!(
        (fields.timestamp_ms, fields.partition, fields.sequence),
        (MAX_TS, 42, 0)
    );
    assert_eq!(entropy_calls.get(), successful_draws + 1);

    assert_generated_range_error(generator.next());
    assert_eq!(entropy_calls.get(), successful_draws + 1);
    Ok(())
}

fn assert_generated_range_error(result: zlid::Result<zlid::ZLID>) {
    assert_eq!(
        result,
        Err(Error::Clock(
            "generated timestamp outside 48-bit range".to_string()
        ))
    );
}

fn assert_event(event: OrderedEvent, expected: (u64, u8, u32, u8)) {
    assert_eq!(
        (
            event.timestamp_ms,
            event.partition,
            event.sequence,
            event.tag
        ),
        expected
    );
}
