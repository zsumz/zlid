//! Deterministic whole-value, inspection, entropy, and concurrency properties.

use std::collections::HashSet;
use std::sync::{Arc, Barrier};

use zlid::{Error, Inspection, InspectionKind, SentinelName, ZLID};

const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[test]
fn arbitrary_payloads_round_trip_and_inspect_consistently() {
    let mut values = deterministic_values();
    values.push(ZLID::NIL);
    values.push(ZLID::MAX);

    for value in &values {
        let bytes = value.bytes();
        let text = value.text();
        let hex = value.bytes_hex();
        let inspection = value.inspect();

        assert_eq!(value.as_bytes(), &bytes);
        assert_eq!(ZLID::from_bytes(&bytes).unwrap(), *value);
        assert_eq!(ZLID::parse(&text).unwrap(), *value);
        assert_eq!(text.len(), 26);
        assert!(text.bytes().all(|byte| ALPHABET.contains(&byte)));
        assert_eq!(inspection.text(), text);
        assert_eq!(inspection.bytes_hex(), hex);
        assert_eq!(inspection.tag(), value.tag());
        assert_inspection_classification(*value, &inspection);
    }

    let mut by_bytes = values.clone();
    by_bytes.sort();
    let byte_order_text: Vec<_> = by_bytes.iter().map(ZLID::text).collect();
    let mut text_order = byte_order_text.clone();
    text_order.sort();
    assert_eq!(byte_order_text, text_order);
    for pair in by_bytes.windows(2) {
        assert_eq!(ZLID::compare(&pair[0], &pair[1]), pair[0].cmp(&pair[1]));
    }
}

#[test]
fn random_generation_rejects_every_wrong_output_length() {
    for actual in [0, 1, 15, 17, 32] {
        let mut source = |requested: usize| {
            assert_eq!(requested, 16);
            vec![0; actual]
        };
        assert!(matches!(
            ZLID::random_with(&mut source),
            Err(Error::InvalidLength {
                what: "random source output",
                expected: 16,
                actual: observed,
            }) if observed == actual
        ));
    }
}

#[test]
fn shared_generator_is_unique_and_partition_correct_across_threads() -> zlid::Result<()> {
    const THREADS: usize = 8;
    const IDS_PER_THREAD: usize = 64;
    const PARTITION: u8 = 73;
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();

    for _ in 0..THREADS {
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || -> zlid::Result<_> {
            barrier.wait();
            let mut ids = Vec::with_capacity(IDS_PER_THREAD);
            for _ in 0..IDS_PER_THREAD {
                ids.push(ZLID::next_with_partition(PARTITION)?);
            }
            Ok(ids)
        }));
    }

    let mut unique = HashSet::with_capacity(THREADS * IDS_PER_THREAD);
    for handle in handles {
        let ids = handle.join().expect("generator thread panicked")?;
        for id in ids {
            let Inspection::Ordered {
                partition: actual, ..
            } = id.inspect()
            else {
                panic!("shared generator emitted a non-ordered value");
            };
            assert_eq!(actual, PARTITION);
            assert!(unique.insert(id), "shared generator emitted a duplicate");
        }
    }
    assert_eq!(unique.len(), THREADS * IDS_PER_THREAD);
    Ok(())
}

fn deterministic_values() -> Vec<ZLID> {
    let mut state = 0xA076_1D64_78BD_642Fu64;
    let mut values = Vec::with_capacity(2_064);
    for _ in 0..2_048 {
        let mut bytes = [0; 16];
        for chunk in bytes.as_chunks_mut::<8>().0 {
            state = next_value(state);
            chunk.copy_from_slice(&state.to_be_bytes());
        }
        values.push(ZLID::from_array(bytes));
    }
    for tag in 0..=15u8 {
        let mut bytes = [0xA5; 16];
        bytes[15] = (bytes[15] & 0xF0) | tag;
        values.push(ZLID::from_array(bytes));
    }
    values
}

fn assert_inspection_classification(value: ZLID, inspection: &Inspection) {
    let (kind, family, sentinel, known) = if value == ZLID::NIL || value == ZLID::MAX {
        (InspectionKind::Sentinel, None, true, false)
    } else {
        match value.tag() {
            1..=4 => (InspectionKind::Ordered, Some("ZLID"), false, true),
            5 => (InspectionKind::Random, Some("ZLID-R"), false, true),
            6..=9 => (InspectionKind::Alias, Some("ZLID-A"), false, true),
            _ => (InspectionKind::Opaque, None, false, false),
        }
    };
    assert_eq!(inspection.kind(), kind);
    assert_eq!(inspection.family(), family);
    assert_eq!(inspection.is_sentinel(), sentinel);
    assert_eq!(inspection.is_known_family(), known);
    assert_eq!(
        kind.wire_name(),
        match kind {
            InspectionKind::Ordered => "ordered",
            InspectionKind::Random => "random",
            InspectionKind::Alias => "alias",
            InspectionKind::Sentinel => "sentinel",
            InspectionKind::Opaque => "opaque",
        }
    );
    match inspection {
        Inspection::Sentinel {
            name: SentinelName::Nil,
            ..
        } => assert_eq!(SentinelName::Nil.wire_name(), "NIL"),
        Inspection::Sentinel {
            name: SentinelName::Max,
            ..
        } => assert_eq!(SentinelName::Max.wire_name(), "MAX"),
        _ => {}
    }
}

fn next_value(mut value: u64) -> u64 {
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}
