//! Shared-generator concurrency and stream-ordering regressions.

use std::collections::HashSet;
use std::sync::{Arc, Barrier};

use zlid::{Inspection, ZLID};

#[test]
fn one_partition_remains_strictly_ordered_under_contention() -> zlid::Result<()> {
    const THREADS: usize = 8;
    const IDS_PER_THREAD: usize = 128;
    const PARTITION: u8 = 197;
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || -> zlid::Result<Vec<ZLID>> {
            barrier.wait();
            let mut ids = Vec::with_capacity(IDS_PER_THREAD);
            for _ in 0..IDS_PER_THREAD {
                ids.push(ZLID::next_with_partition(PARTITION)?);
            }
            Ok(ids)
        }));
    }

    let mut ids = Vec::with_capacity(THREADS * IDS_PER_THREAD);
    for handle in handles {
        ids.extend(handle.join().expect("generation thread")?);
    }
    let unique: HashSet<_> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());

    ids.sort_unstable();
    let mut previous = None;
    for id in ids {
        let Inspection::Ordered {
            timestamp_ms,
            partition,
            sequence,
            ..
        } = id.inspect()
        else {
            panic!("shared generator returned a non-ordered ID");
        };
        assert_eq!(partition, PARTITION);
        let stream_position = (timestamp_ms, sequence);
        if let Some(previous) = previous {
            assert!(stream_position > previous);
        }
        previous = Some(stream_position);
    }
    Ok(())
}

#[test]
fn partitions_keep_independent_sequence_state() -> zlid::Result<()> {
    let first = ZLID::next_with_partition(211)?;
    let other = ZLID::next_with_partition(212)?;
    let second = ZLID::next_with_partition(211)?;

    let first_position = ordered_position(first, 211);
    let other_position = ordered_position(other, 212);
    let second_position = ordered_position(second, 211);
    assert_eq!(first_position.1, 0);
    assert_eq!(other_position.1, 0);
    assert!(second_position > first_position);
    Ok(())
}

fn ordered_position(id: ZLID, expected_partition: u8) -> (u64, u32) {
    let Inspection::Ordered {
        timestamp_ms,
        partition,
        sequence,
        ..
    } = id.inspect()
    else {
        panic!("shared generator returned a non-ordered ID");
    };
    assert_eq!(partition, expected_partition);
    (timestamp_ms, sequence)
}
