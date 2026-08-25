//! Deterministic boundary and property coverage for ordered wire layouts.

use zlid::{
    wire::{pack_ordered, unpack_ordered, ClockState, OrderedFields},
    Error, Profile,
};

const MAX_TIMESTAMP: u64 = (1u64 << 48) - 1;

#[derive(Copy, Clone)]
struct Layout {
    profile: Profile,
    sequence_max: u32,
    random_max: u64,
    normal_tag: u8,
    clamped_tag: u8,
}

const LAYOUTS: [Layout; 2] = [
    Layout {
        profile: Profile::Default,
        sequence_max: (1u32 << 12) - 1,
        random_max: (1u64 << 56) - 1,
        normal_tag: 1,
        clamped_tag: 3,
    },
    Layout {
        profile: Profile::HighThroughput,
        sequence_max: (1u32 << 16) - 1,
        random_max: (1u64 << 52) - 1,
        normal_tag: 2,
        clamped_tag: 4,
    },
];

#[test]
fn ordered_minimum_and_maximum_fields_round_trip() {
    for layout in LAYOUTS {
        assert_round_trip(layout, 0, 0, 0, 0, layout.normal_tag);
        assert_round_trip(layout, 0, 0, 0, 0, layout.clamped_tag);
        assert_round_trip(
            layout,
            MAX_TIMESTAMP,
            u8::MAX,
            layout.sequence_max,
            layout.random_max,
            layout.normal_tag,
        );
        assert_round_trip(
            layout,
            MAX_TIMESTAMP,
            u8::MAX,
            layout.sequence_max,
            layout.random_max,
            layout.clamped_tag,
        );
    }
}

#[test]
fn deterministic_ordered_field_corpus_round_trips() {
    let mut state = 0xD6E8_FEB8_6659_FD93u64;
    for layout in LAYOUTS {
        for index in 0..1_024u32 {
            let timestamp = next_value(&mut state) & MAX_TIMESTAMP;
            let partition = next_value(&mut state) as u8;
            let sequence = (next_value(&mut state) as u32) & layout.sequence_max;
            let random_tail = next_value(&mut state) & layout.random_max;
            let tag = if index.is_multiple_of(2) {
                layout.normal_tag
            } else {
                layout.clamped_tag
            };
            assert_round_trip(layout, timestamp, partition, sequence, random_tail, tag);
        }
    }
}

#[test]
fn pack_rejects_each_out_of_range_field_and_wrong_tag() {
    for layout in LAYOUTS {
        for timestamp in [MAX_TIMESTAMP + 1, u64::MAX] {
            assert_out_of_range(pack_ordered(
                layout.profile,
                timestamp,
                0,
                0,
                0,
                layout.normal_tag,
            ));
        }
        for sequence in [layout.sequence_max + 1, u32::MAX] {
            assert_out_of_range(pack_ordered(
                layout.profile,
                0,
                0,
                sequence,
                0,
                layout.normal_tag,
            ));
        }
        for random_tail in [layout.random_max + 1, u64::MAX] {
            assert_out_of_range(pack_ordered(
                layout.profile,
                0,
                0,
                0,
                random_tail,
                layout.normal_tag,
            ));
        }

        for tag in 0..=15 {
            if tag != layout.normal_tag && tag != layout.clamped_tag {
                assert_invalid_tag(pack_ordered(layout.profile, 0, 0, 0, 0, tag));
            }
        }
    }
}

#[test]
fn unpack_rejects_every_nonordered_tag_and_both_sentinels() {
    for tag in 0..=15u8 {
        if ![1, 2, 3, 4].contains(&tag) {
            let mut bytes = [0xA5; 16];
            bytes[15] = (bytes[15] & 0xF0) | tag;
            assert_invalid_family(unpack_ordered(&bytes));
        }
    }

    assert_invalid_family(unpack_ordered(&[0; 16]));
    assert_invalid_family(unpack_ordered(&[u8::MAX; 16]));
}

fn assert_round_trip(
    layout: Layout,
    timestamp_ms: u64,
    partition: u8,
    sequence: u32,
    random_tail: u64,
    tag: u8,
) {
    let bytes = pack_ordered(
        layout.profile,
        timestamp_ms,
        partition,
        sequence,
        random_tail,
        tag,
    )
    .expect("valid ordered fields");
    let fields = unpack_ordered(&bytes).expect("packed ordered payload");

    assert_eq!(fields.profile, layout.profile);
    assert_eq!(fields.timestamp_ms, timestamp_ms);
    assert_eq!(fields.partition, partition);
    assert_eq!(fields.sequence, sequence);
    assert_eq!(fields.random_tail, random_tail);
    assert_eq!(fields.tag, tag);
    assert_eq!(
        fields.clock_state,
        if tag == layout.clamped_tag {
            ClockState::Clamped
        } else {
            ClockState::Normal
        }
    );
}

fn assert_out_of_range(result: zlid::Result<[u8; 16]>) {
    assert!(matches!(result, Err(Error::FieldOutOfRange { .. })));
}

fn assert_invalid_tag(result: zlid::Result<[u8; 16]>) {
    assert!(matches!(result, Err(Error::InvalidTag { .. })));
}

fn assert_invalid_family(result: zlid::Result<OrderedFields>) {
    assert!(matches!(result, Err(Error::InvalidTag { .. })));
}

fn next_value(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}
