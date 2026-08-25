//! Regression tests for ordered generation failure atomicity.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use zlid::{
    advanced::{Clock, EntropySource},
    Error, Inspection, OrderedGenerator, Profile,
};

struct FailAtCall {
    calls: usize,
    fail_at: usize,
    trace: Option<Rc<RefCell<Vec<&'static str>>>>,
}

impl EntropySource for FailAtCall {
    fn fill_bytes(&mut self, out: &mut [u8]) -> zlid::Result<()> {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push("entropy");
        }
        let call = self.calls;
        self.calls += 1;
        if call == self.fail_at {
            return Err(Error::EntropyUnavailable(
                "injected entropy failure".to_string(),
            ));
        }
        out.fill(0);
        Ok(())
    }
}

struct PartialThenZero {
    failed: bool,
}

impl EntropySource for PartialThenZero {
    fn fill_bytes(&mut self, out: &mut [u8]) -> zlid::Result<()> {
        if !self.failed {
            self.failed = true;
            out.fill(0xff);
            return Err(Error::EntropyUnavailable(
                "partial entropy failure".to_string(),
            ));
        }
        out.fill(0);
        Ok(())
    }
}

struct FailOnceClock {
    failed: bool,
}

impl Clock for FailOnceClock {
    fn now_ms(&mut self) -> zlid::Result<u64> {
        if !self.failed {
            self.failed = true;
            return Err(Error::Clock("injected clock failure".to_string()));
        }
        Ok(1_000)
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
fn first_entropy_failure_does_not_initialize_stream() -> zlid::Result<()> {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let clock_trace = Rc::clone(&trace);
    let clock = move || {
        clock_trace.borrow_mut().push("clock");
        1_000
    };
    let entropy = FailAtCall {
        calls: 0,
        fail_at: 0,
        trace: Some(Rc::clone(&trace)),
    };
    let mut generator = OrderedGenerator::with_sources(Profile::Default, 7, clock, entropy);

    assert!(matches!(
        generator.next(),
        Err(Error::EntropyUnavailable(_))
    ));
    assert_eq!(*trace.borrow(), ["clock", "entropy"]);

    assert_ordered(generator.next()?, (1_000, 7, 0, 1));
    assert_eq!(*trace.borrow(), ["clock", "entropy", "clock", "entropy"]);
    Ok(())
}

#[test]
fn overflow_entropy_failure_retries_same_carried_event() -> zlid::Result<()> {
    assert_overflow_retry(Profile::Default, 4_095, 1)?;
    assert_overflow_retry(Profile::HighThroughput, 65_535, 2)
}

#[test]
fn failed_newer_clock_does_not_change_later_clamp_baseline() -> zlid::Result<()> {
    let times = Rc::new(RefCell::new(VecDeque::from([1_000, 2_000, 900])));
    let clock_times = Rc::clone(&times);
    let clock = move || clock_times.borrow_mut().pop_front().expect("clock value");
    let entropy = FailAtCall {
        calls: 0,
        fail_at: 1,
        trace: None,
    };
    let mut generator = OrderedGenerator::with_sources(Profile::Default, 3, clock, entropy);

    assert_ordered(generator.next()?, (1_000, 3, 0, 1));
    assert!(matches!(
        generator.next(),
        Err(Error::EntropyUnavailable(_))
    ));
    assert_ordered(generator.next()?, (1_000, 3, 1, 3));
    Ok(())
}

#[test]
fn partial_entropy_write_then_error_does_not_initialize_stream() -> zlid::Result<()> {
    let entropy = PartialThenZero { failed: false };
    let mut generator = OrderedGenerator::with_sources(Profile::Default, 4, || 1_000, entropy);

    assert!(matches!(
        generator.next(),
        Err(Error::EntropyUnavailable(_))
    ));
    let id = generator.next()?;
    assert_ordered(id, (1_000, 4, 0, 1));
    let Inspection::Ordered { random_hex, .. } = id.inspect() else {
        panic!("generator returned a non-ordered ID");
    };
    assert_eq!(random_hex, "00000000000000");
    Ok(())
}

#[test]
fn failed_partition_does_not_change_any_partition_state() -> zlid::Result<()> {
    let entropy = FailAtCall {
        calls: 0,
        fail_at: 0,
        trace: None,
    };
    let mut generator = OrderedGenerator::with_sources(Profile::Default, 0, || 1_000, entropy);

    assert!(matches!(
        generator.next_with_partition(7),
        Err(Error::EntropyUnavailable(_))
    ));
    assert_ordered(generator.next_with_partition(8)?, (1_000, 8, 0, 1));
    assert_ordered(generator.next_with_partition(7)?, (1_000, 7, 0, 1));
    Ok(())
}

#[test]
fn clock_error_draws_no_entropy_and_changes_no_state() -> zlid::Result<()> {
    let entropy_calls = Rc::new(Cell::new(0));
    let entropy = CountingEntropy {
        calls: Rc::clone(&entropy_calls),
    };
    let mut generator = OrderedGenerator::with_sources(
        Profile::Default,
        5,
        FailOnceClock { failed: false },
        entropy,
    );

    assert!(matches!(generator.next(), Err(Error::Clock(_))));
    assert_eq!(entropy_calls.get(), 0);
    assert_ordered(generator.next()?, (1_000, 5, 0, 1));
    assert_eq!(entropy_calls.get(), 1);
    Ok(())
}

fn assert_overflow_retry(profile: Profile, sequence_max: u32, normal_tag: u8) -> zlid::Result<()> {
    let entropy = FailAtCall {
        calls: 0,
        fail_at: sequence_max as usize + 1,
        trace: None,
    };
    let mut generator = OrderedGenerator::with_sources(profile, 9, || 5_000, entropy);

    for _ in 0..=sequence_max {
        generator.next()?;
    }
    assert!(matches!(
        generator.next(),
        Err(Error::EntropyUnavailable(_))
    ));

    assert_ordered(generator.next()?, (5_001, 9, 0, normal_tag));
    Ok(())
}

fn assert_ordered(id: zlid::ZLID, expected: (u64, u8, u32, u8)) {
    let Inspection::Ordered {
        timestamp_ms,
        partition,
        sequence,
        tag,
        ..
    } = id.inspect()
    else {
        panic!("generator returned a non-ordered ID");
    };
    assert_eq!((timestamp_ms, partition, sequence, tag), expected);
}
