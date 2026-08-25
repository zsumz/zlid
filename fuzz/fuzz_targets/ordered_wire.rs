#![no_main]

use libfuzzer_sys::fuzz_target;
use zlid::wire::{pack_ordered, unpack_ordered};
use zlid::Profile;

const INPUT_LENGTH: usize = 39;

fuzz_target!(|input: &[u8]| {
    if input.len() < INPUT_LENGTH {
        return;
    }

    let selector = input[0];
    let profile = if selector & 1 == 0 {
        Profile::Default
    } else {
        Profile::HighThroughput
    };
    let timestamp_raw = read_u64(&input[1..9]);
    let timestamp_ms = if selector & 0x02 == 0 {
        timestamp_raw & MAX_TIMESTAMP
    } else {
        timestamp_raw
    };
    let partition = input[9];
    let sequence_raw = read_u32(&input[10..14]);
    let random_raw = read_u64(&input[14..22]);
    let (sequence_max, random_max, normal_tag, clamped_tag) = if selector & 1 == 0 {
        (0x0fff, (1u64 << 56) - 1, 1, 3)
    } else {
        (0xffff, (1u64 << 52) - 1, 2, 4)
    };
    let sequence = if selector & 0x04 == 0 {
        sequence_raw & sequence_max
    } else {
        sequence_raw
    };
    let random_tail = if selector & 0x08 == 0 {
        random_raw & random_max
    } else {
        random_raw
    };
    let tag = if selector & 0x10 == 0 {
        if input[22] & 1 == 0 {
            normal_tag
        } else {
            clamped_tag
        }
    } else {
        input[22]
    };

    if let Ok(packed) = pack_ordered(profile, timestamp_ms, partition, sequence, random_tail, tag) {
        let fields = unpack_ordered(&packed).expect("packed ordered fields must unpack");
        assert_eq!(fields.profile, profile);
        assert_eq!(fields.timestamp_ms, timestamp_ms);
        assert_eq!(fields.partition, partition);
        assert_eq!(fields.sequence, sequence);
        assert_eq!(fields.random_tail, random_tail);
        assert_eq!(fields.tag, tag);
        assert_eq!(
            pack_ordered(
                fields.profile,
                fields.timestamp_ms,
                fields.partition,
                fields.sequence,
                fields.random_tail,
                fields.tag,
            ),
            Ok(packed)
        );
    }

    let mut raw = [0u8; ZLID_BYTES];
    raw.copy_from_slice(&input[23..INPUT_LENGTH]);
    if let Ok(fields) = unpack_ordered(&raw) {
        assert_eq!(
            pack_ordered(
                fields.profile,
                fields.timestamp_ms,
                fields.partition,
                fields.sequence,
                fields.random_tail,
                fields.tag,
            ),
            Ok(raw)
        );
    }
});

const ZLID_BYTES: usize = 16;
const MAX_TIMESTAMP: u64 = (1u64 << 48) - 1;

fn read_u32(input: &[u8]) -> u32 {
    let mut bytes = [0; 4];
    bytes.copy_from_slice(input);
    u32::from_le_bytes(bytes)
}

fn read_u64(input: &[u8]) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(input);
    u64::from_le_bytes(bytes)
}
