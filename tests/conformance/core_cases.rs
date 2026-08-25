use std::collections::BTreeMap;

use crate::hex;
use zlid::{wire::pack_ordered, Inspection, ZLID};

use crate::helpers::{parse_clock_state, parse_profile};
use crate::json::{get, number, object, string, Json};

pub(crate) fn assert_ordered_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let profile = parse_profile(string(get(entry, "profile")));
        let timestamp_ms = number(get(entry, "timestampMs")) as u64;
        let partition = number(get(entry, "partition")) as u8;
        let sequence = number(get(entry, "sequence")) as u32;
        let random_tail = u64::from_str_radix(string(get(entry, "randomHex")), 16).unwrap();
        let tag = number(get(entry, "tag")) as u8;
        let packed =
            pack_ordered(profile, timestamp_ms, partition, sequence, random_tail, tag).unwrap();

        assert_eq!(string(get(entry, "bytesHex")), hex::encode(&packed));

        let id = ZLID::parse(string(get(entry, "text"))).unwrap();
        assert_eq!(packed, id.bytes());
        assert_eq!(string(get(entry, "text")), id.text());

        match id.inspect() {
            Inspection::Ordered {
                profile: actual_profile,
                clock_state,
                timestamp_ms: actual_timestamp,
                partition: actual_partition,
                sequence: actual_sequence,
                random_hex,
                tag: actual_tag,
                ..
            } => {
                assert_eq!(profile, actual_profile);
                assert_eq!(
                    parse_clock_state(string(get(entry, "clockState"))),
                    clock_state
                );
                assert_eq!(timestamp_ms, actual_timestamp);
                assert_eq!(partition, actual_partition);
                assert_eq!(sequence, actual_sequence);
                assert_eq!(string(get(entry, "randomHex")), random_hex);
                assert_eq!(tag, actual_tag);
            }
            other => panic!("expected ordered inspection, got {other:?}"),
        }
    }
}

pub(crate) fn assert_random_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let entropy = hex::decode(string(get(entry, "inputEntropyHex"))).unwrap();
        let mut source = |size: usize| {
            assert_eq!(entropy.len(), size);
            entropy.clone()
        };
        let id = ZLID::random_with(&mut source).unwrap();

        assert_eq!(string(get(entry, "bytesHex")), id.bytes_hex());
        assert_eq!(string(get(entry, "text")), id.text());
        match id.inspect() {
            Inspection::Random {
                tag, random_hex, ..
            } => {
                assert_eq!(number(get(entry, "tag")) as u8, tag);
                assert_eq!(string(get(entry, "randomHex")), random_hex);
            }
            other => panic!("expected random inspection, got {other:?}"),
        }
    }
}

pub(crate) fn assert_alias_section(domain: &BTreeMap<String, Json>, entries: &[Json]) {
    let key = hex::decode(string(get(domain, "keyHex"))).unwrap();
    let tweak = string(get(domain, "tweakUtf8"));

    for entry in entries {
        let entry = object(entry);
        let source =
            ZLID::from_bytes(&hex::decode(string(get(entry, "sourceBytesHex"))).unwrap()).unwrap();
        assert_eq!(string(get(entry, "sourceText")), source.text());

        let tweak_hex = entry.get("tweakHex").map(string);
        let alias = if let Some(hex) = tweak_hex {
            source
                .alias(&key, &crate::hex::decode(hex).unwrap())
                .unwrap()
        } else {
            source.alias_str(&key, tweak).unwrap()
        };
        assert_eq!(string(get(entry, "bytesHex")), alias.bytes_hex());
        assert_eq!(string(get(entry, "text")), alias.text());
        let unaliased = if let Some(hex) = tweak_hex {
            alias
                .unalias(&key, &crate::hex::decode(hex).unwrap())
                .unwrap()
        } else {
            alias.unalias_str(&key, tweak).unwrap()
        };
        assert_eq!(source, unaliased);

        match alias.inspect() {
            Inspection::Alias {
                source_profile,
                source_clock_state,
                alias_data_hex,
                tag,
                ..
            } => {
                assert_eq!(
                    parse_profile(string(get(entry, "sourceProfile"))),
                    source_profile
                );
                assert_eq!(
                    parse_clock_state(string(get(entry, "sourceClockState"))),
                    source_clock_state
                );
                assert_eq!(string(get(entry, "aliasDataHex")), alias_data_hex);
                assert_eq!(number(get(entry, "tag")) as u8, tag);
            }
            other => panic!("expected alias inspection, got {other:?}"),
        }
    }

    let random = ZLID::parse("014D2PF2DBSQQG28T5CY4TQKF5").unwrap();
    assert!(random.alias_str(&key, tweak).is_err());
    assert!(random.unalias_str(&key, tweak).is_err());
    assert!(ZLID::NIL.alias_str(&key, tweak).is_err());
    assert!(ZLID::parse("01K2R7KFWE5807000000000001")
        .unwrap()
        .alias(&[], b"")
        .is_err());
}
