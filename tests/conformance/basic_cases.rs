use std::collections::BTreeMap;

use crate::hex;
use zlid::{Inspection, InspectionKind, ZLID};

use crate::helpers::{ordering_sign, parse_sentinel_name};
use crate::json::{array, get, number, object, string, Json};

pub(crate) fn assert_sentinel_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let id = ZLID::from_bytes(&hex::decode(string(get(entry, "bytesHex"))).unwrap()).unwrap();
        assert_eq!(string(get(entry, "text")), id.text());
        assert_eq!(
            id.bytes(),
            ZLID::parse(string(get(entry, "text"))).unwrap().bytes()
        );

        let inspection = id.inspect();
        assert_eq!(InspectionKind::Sentinel, inspection.kind());
        assert!(inspection.is_sentinel());
        assert!(!inspection.is_known_family());
        assert_eq!(number(get(entry, "tag")) as u8, inspection.tag());
        match inspection {
            Inspection::Sentinel { name, .. } => {
                assert_eq!(parse_sentinel_name(string(get(entry, "name"))), name);
            }
            other => panic!("expected sentinel inspection, got {other:?}"),
        }
    }
}

pub(crate) fn assert_opaque_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let id = ZLID::from_bytes(&hex::decode(string(get(entry, "bytesHex"))).unwrap()).unwrap();
        assert_eq!(string(get(entry, "text")), id.text());
        assert_eq!(
            id.bytes(),
            ZLID::parse(string(get(entry, "text"))).unwrap().bytes()
        );

        let inspection = id.inspect();
        assert_eq!(InspectionKind::Opaque, inspection.kind());
        assert!(!inspection.is_sentinel());
        assert!(!inspection.is_known_family());
        assert_eq!(number(get(entry, "tag")) as u8, inspection.tag());
    }
}

pub(crate) fn assert_friendly_parsing(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        assert_eq!(
            string(get(entry, "canonical")),
            ZLID::parse(string(get(entry, "input"))).unwrap().text()
        );
    }
}

pub(crate) fn assert_negative_parsing(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        assert!(
            ZLID::parse(string(get(entry, "input"))).is_err(),
            "expected parser rejection for {}",
            string(get(entry, "id"))
        );
    }
}

pub(crate) fn assert_invalid_operations(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let result = match string(get(entry, "operation")) {
            "alias" => {
                let source = ZLID::parse(string(get(entry, "sourceText"))).unwrap();
                let key = hex::decode(string(get(entry, "keyHex"))).unwrap();
                if let Some(tweak_hex) = entry.get("tweakHex") {
                    let tweak = hex::decode(string(tweak_hex)).unwrap();
                    source.alias(&key, &tweak).map(|_| ())
                } else {
                    source
                        .alias_str(&key, string(get(entry, "tweakUtf8")))
                        .map(|_| ())
                }
            }
            "unalias" => {
                let alias = ZLID::parse(string(get(entry, "sourceText"))).unwrap();
                let key = hex::decode(string(get(entry, "keyHex"))).unwrap();
                if let Some(tweak_hex) = entry.get("tweakHex") {
                    let tweak = hex::decode(string(tweak_hex)).unwrap();
                    alias.unalias(&key, &tweak).map(|_| ())
                } else {
                    alias
                        .unalias_str(&key, string(get(entry, "tweakUtf8")))
                        .map(|_| ())
                }
            }
            "partition" => {
                let key = hex::decode(string(get(entry, "keyHex"))).unwrap();
                if let Some(input) = entry.get("inputUtf8") {
                    ZLID::partition_str(string(input), Some(&key)).map(|_| ())
                } else {
                    let input = hex::decode(string(get(entry, "inputHex"))).unwrap();
                    ZLID::partition_bytes(&input, Some(&key)).map(|_| ())
                }
            }
            "fromBytes" => {
                let input = hex::decode(string(get(entry, "inputHex"))).unwrap();
                ZLID::from_bytes(&input).map(|_| ())
            }
            other => panic!("unknown invalid operation {other}"),
        };
        assert!(
            result.is_err(),
            "invalid operation {} ({}) succeeded",
            string(get(entry, "id")),
            string(get(entry, "operation"))
        );
    }
}

pub(crate) fn assert_partition_section(partition: &BTreeMap<String, Json>) {
    for entry in array(get(partition, "cases")) {
        let entry = object(entry);
        let uses_default_key = entry.contains_key("usesDefaultKey");
        let actual = if let Some(value) = entry.get("inputUtf8") {
            if uses_default_key {
                ZLID::partition_str(string(value), None).unwrap()
            } else {
                let key = hex::decode(string(get(entry, "keyHex"))).unwrap();
                ZLID::partition_str(string(value), Some(&key)).unwrap()
            }
        } else {
            let input = hex::decode(string(get(entry, "inputHex"))).unwrap();
            if uses_default_key {
                ZLID::partition_bytes(&input, None).unwrap()
            } else {
                let key = hex::decode(string(get(entry, "keyHex"))).unwrap();
                ZLID::partition_bytes(&input, Some(&key)).unwrap()
            }
        };
        assert_eq!(number(get(entry, "output")) as u8, actual);

        if entry.get("keyHex").map(string) == Some("00000000000000000000000000000000") {
            let input = hex::decode(string(get(entry, "inputHex"))).unwrap();
            assert_eq!(actual, ZLID::partition_bytes(&input, None).unwrap());
        }
    }
}

pub(crate) fn assert_compare_section(entries: &[Json]) {
    for entry in entries {
        let entry = object(entry);
        let left = ZLID::parse(string(get(entry, "leftText"))).unwrap();
        let right = ZLID::parse(string(get(entry, "rightText"))).unwrap();
        assert_eq!(
            number(get(entry, "expected")) as i8,
            ordering_sign(left.cmp(&right))
        );
        assert_eq!(
            -(number(get(entry, "expected")) as i8),
            ordering_sign(right.cmp(&left))
        );
    }
}
