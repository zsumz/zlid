use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use zlid::{
    bytes_from_hex, ClockState, Inspection, OrderedEvent, OrderedGenerator, OrderedGeneratorCore,
    Profile, ZLID,
};

use crate::helpers::parse_profile;
use crate::json::{array, get, number, object, string, Json};

pub(crate) fn assert_generated_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let profile = parse_profile(string(get(entry, "profile")));
        let default_partition = number(get(entry, "defaultPartition")) as u8;
        let warmup_calls = entry.get("warmupCalls").map(number).unwrap_or(0) as usize;
        let warmup_now_ms = entry.get("warmupNowMs").map(number).unwrap_or(0) as u64;
        let warmup_entropy = entry
            .get("warmupEntropyHex")
            .map(|value| bytes_from_hex(string(value)).unwrap());
        let times: Rc<RefCell<VecDeque<u64>>> = Rc::new(RefCell::new(
            array(get(entry, "nowMs"))
                .iter()
                .map(|value| number(value) as u64)
                .collect(),
        ));
        let entropy_chunks: Rc<RefCell<VecDeque<Vec<u8>>>> = Rc::new(RefCell::new(
            array(get(entry, "entropyHex"))
                .iter()
                .map(|value| bytes_from_hex(string(value)).unwrap())
                .collect(),
        ));

        let warmup_clock_remaining = Rc::new(RefCell::new(warmup_calls));
        let clock_warmups = Rc::clone(&warmup_clock_remaining);
        let clock_times = Rc::clone(&times);
        let clock = move || {
            let mut remaining = clock_warmups.borrow_mut();
            if *remaining > 0 {
                *remaining -= 1;
                return warmup_now_ms;
            }
            drop(remaining);
            clock_times
                .borrow_mut()
                .pop_front()
                .expect("generated test clock exhausted")
        };
        let warmup_entropy_remaining = Rc::new(RefCell::new(warmup_calls));
        let entropy_warmups = Rc::clone(&warmup_entropy_remaining);
        let entropy_values = Rc::clone(&entropy_chunks);
        let entropy = move |size: usize| {
            let mut remaining = entropy_warmups.borrow_mut();
            if *remaining > 0 {
                *remaining -= 1;
                let bytes = warmup_entropy
                    .as_ref()
                    .expect("generated test warmup entropy missing")
                    .clone();
                assert_eq!(size, bytes.len());
                return bytes;
            }
            drop(remaining);
            let bytes = entropy_values
                .borrow_mut()
                .pop_front()
                .expect("generated test entropy exhausted");
            assert_eq!(size, bytes.len());
            bytes
        };
        let mut generator =
            OrderedGenerator::with_sources(profile, default_partition, clock, entropy);

        let mut last_warmup = None;
        for _ in 0..warmup_calls {
            last_warmup = Some(generator.next().unwrap());
        }
        if let Some(expected) = entry.get("warmupLast") {
            assert_generated_id(
                last_warmup
                    .as_ref()
                    .expect("generated test warmup ID missing"),
                profile,
                object(expected),
            );
        }

        for call in array(get(entry, "calls")) {
            let call = object(call);
            let id = if let Some(partition) = call.get("partitionOverride") {
                generator
                    .next_with_partition(number(partition) as u8)
                    .unwrap()
            } else {
                generator.next().unwrap()
            };

            assert_generated_id(&id, profile, call);
        }

        assert_eq!(
            *warmup_clock_remaining.borrow(),
            0,
            "generated case {} left warmup clock values unused",
            string(get(entry, "id"))
        );
        assert_eq!(
            *warmup_entropy_remaining.borrow(),
            0,
            "generated case {} left warmup entropy chunks unused",
            string(get(entry, "id"))
        );
        assert!(
            times.borrow().is_empty(),
            "generated case {} left clock values unused",
            string(get(entry, "id"))
        );
        assert!(
            entropy_chunks.borrow().is_empty(),
            "generated case {} left entropy chunks unused",
            string(get(entry, "id"))
        );
    }
}

fn assert_generated_id(id: &ZLID, profile: Profile, expected: &BTreeMap<String, Json>) {
    assert_eq!(string(get(expected, "text")), id.text());
    assert_eq!(string(get(expected, "bytesHex")), id.bytes_hex());

    match id.inspect() {
        Inspection::Ordered {
            profile: actual_profile,
            clock_state,
            timestamp_ms,
            partition,
            sequence,
            random_hex,
            tag,
            ..
        } => {
            assert_eq!(profile, actual_profile);
            assert_eq!(
                generated_clock_state(number(get(expected, "tag")) as u8),
                clock_state
            );
            assert_eq!(number(get(expected, "timestampMs")) as u64, timestamp_ms);
            assert_eq!(number(get(expected, "partition")) as u8, partition);
            assert_eq!(number(get(expected, "sequence")) as u32, sequence);
            assert_eq!(string(get(expected, "randomHex")), random_hex);
            assert_eq!(number(get(expected, "tag")) as u8, tag);
        }
        other => panic!("expected generated ordered inspection, got {other:?}"),
    }
}

pub(crate) fn assert_generator_section(generator: &BTreeMap<String, Json>) {
    for entry in array(get(generator, "cases")) {
        let entry = object(entry);
        let profile = parse_profile(string(get(entry, "profile")));
        let partition = number(get(entry, "partition")) as u8;
        let mut remaining_times: Option<Rc<RefCell<VecDeque<u64>>>> = None;
        let clock: Box<dyn FnMut() -> u64> = if let Some(value) = entry.get("constantNowMs") {
            let constant = number(value) as u64;
            Box::new(move || constant)
        } else {
            let times: Rc<RefCell<VecDeque<u64>>> = Rc::new(RefCell::new(
                array(get(entry, "nowMs"))
                    .iter()
                    .map(|value| number(value) as u64)
                    .collect(),
            ));
            remaining_times = Some(Rc::clone(&times));
            Box::new(move || {
                times
                    .borrow_mut()
                    .pop_front()
                    .expect("test clock exhausted")
            })
        };
        let mut core = OrderedGeneratorCore::new(profile, 0, clock);

        let mut last_warmup = None;
        let warmup_calls = entry.get("warmupCalls").map(number).unwrap_or(0);
        for _ in 0..warmup_calls {
            last_warmup = Some(core.next(Some(partition)).unwrap());
        }
        if let Some(expected) = entry.get("warmupLast") {
            assert_event(object(expected), last_warmup.expect("warmup event"));
        }

        for expected in array(get(entry, "events")) {
            assert_event(object(expected), core.next(Some(partition)).unwrap());
        }
        if let Some(times) = remaining_times {
            assert!(
                times.borrow().is_empty(),
                "generator case {} left clock values unused",
                string(get(entry, "id"))
            );
        }
    }
}

fn generated_clock_state(tag: u8) -> ClockState {
    match tag {
        3 | 4 => ClockState::Clamped,
        _ => ClockState::Normal,
    }
}

fn assert_event(expected: &BTreeMap<String, Json>, actual: OrderedEvent) {
    assert_eq!(
        number(get(expected, "timestampMs")) as u64,
        actual.timestamp_ms
    );
    assert_eq!(number(get(expected, "partition")) as u8, actual.partition);
    assert_eq!(number(get(expected, "sequence")) as u32, actual.sequence);
    assert_eq!(number(get(expected, "tag")) as u8, actual.tag);
}
