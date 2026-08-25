use crate::json::{get, object, string, Json};
use crate::schema_support::{
    canonical_text, case_id, clock_state, const_true, enum_string, exact_keys, exact_string,
    hex_field, nonempty_string, number_range, optional_hex, optional_string, profile, string_field,
    MAX_TIMESTAMP,
};

pub(crate) fn assert_ordered_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &[
                "id",
                "family",
                "profile",
                "clockState",
                "timestampMs",
                "partition",
                "sequence",
                "randomHex",
                "tag",
                "bytesHex",
                "text",
            ],
            &[],
        );
        case_id(entry);
        exact_string(entry, "family", "ZLID");
        profile(entry);
        clock_state(entry, "clockState");
        number_range(entry, "timestampMs", 0, MAX_TIMESTAMP);
        number_range(entry, "partition", 0, 255);
        number_range(entry, "sequence", 0, 65_535);
        let random_length = string_field(entry, "randomHex").len();
        assert!([13, 14].contains(&random_length));
        hex_field(entry, "randomHex", Some(random_length));
        number_range(entry, "tag", 1, 4);
        hex_field(entry, "bytesHex", Some(32));
        canonical_text(entry, "text");
    }
}

pub(crate) fn assert_random_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &[
                "id",
                "inputEntropyHex",
                "tag",
                "bytesHex",
                "text",
                "randomHex",
            ],
            &[],
        );
        case_id(entry);
        hex_field(entry, "inputEntropyHex", Some(32));
        number_range(entry, "tag", 5, 5);
        hex_field(entry, "bytesHex", Some(32));
        canonical_text(entry, "text");
        hex_field(entry, "randomHex", Some(31));
    }
}

pub(crate) fn assert_alias_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &[
                "id",
                "sourceId",
                "sourceBytesHex",
                "sourceText",
                "sourceProfile",
                "sourceClockState",
                "tag",
                "bytesHex",
                "text",
                "aliasDataHex",
            ],
            &["tweakHex"],
        );
        case_id(entry);
        assert_case_id_value(entry, "sourceId");
        hex_field(entry, "sourceBytesHex", Some(32));
        canonical_text(entry, "sourceText");
        enum_string(entry, "sourceProfile", &["default", "high-throughput"]);
        clock_state(entry, "sourceClockState");
        optional_hex(entry, "tweakHex", None);
        number_range(entry, "tag", 6, 9);
        hex_field(entry, "bytesHex", Some(32));
        canonical_text(entry, "text");
        hex_field(entry, "aliasDataHex", Some(31));
    }
}

pub(crate) fn assert_sentinel_and_opaque_cases(sentinels: &[Json], opaque: &[Json]) {
    assert!(
        sentinels.len() >= 2,
        "fixture requires at least two sentinels"
    );
    for value in sentinels {
        let entry = object(value);
        exact_keys(entry, &["id", "name", "tag", "bytesHex", "text"], &[]);
        case_id(entry);
        enum_string(entry, "name", &["NIL", "MAX"]);
        let tag = crate::json::number(get(entry, "tag"));
        assert!([0, 15].contains(&tag), "invalid sentinel tag {tag}");
        hex_field(entry, "bytesHex", Some(32));
        canonical_text(entry, "text");
    }
    for value in opaque {
        let entry = object(value);
        exact_keys(entry, &["id", "tag", "bytesHex", "text"], &[]);
        case_id(entry);
        number_range(entry, "tag", 0, 15);
        hex_field(entry, "bytesHex", Some(32));
        canonical_text(entry, "text");
    }
}

pub(crate) fn assert_parse_cases(friendly: &[Json], negative: &[Json]) {
    for value in friendly {
        let entry = object(value);
        exact_keys(entry, &["id", "input", "canonical"], &[]);
        case_id(entry);
        nonempty_string(entry, "input");
        canonical_text(entry, "canonical");
    }
    for value in negative {
        let entry = object(value);
        exact_keys(entry, &["id", "input", "reason"], &[]);
        case_id(entry);
        string_field(entry, "input");
        nonempty_string(entry, "reason");
    }
}

pub(crate) fn assert_invalid_operation_cases(entries: &[Json]) {
    const REQUIRED: &[&str] = &["id", "operation", "reason"];
    const OPTIONAL: &[&str] = &[
        "sourceText",
        "inputHex",
        "inputUtf8",
        "keyHex",
        "tweakHex",
        "tweakUtf8",
    ];
    for value in entries {
        let entry = object(value);
        exact_keys(entry, REQUIRED, OPTIONAL);
        case_id(entry);
        enum_string(
            entry,
            "operation",
            &["alias", "unalias", "partition", "fromBytes"],
        );
        nonempty_string(entry, "reason");
        optional_hex(entry, "inputHex", None);
        optional_string(entry, "inputUtf8");
        optional_hex(entry, "keyHex", None);
        optional_hex(entry, "tweakHex", None);
        optional_string(entry, "tweakUtf8");
        if entry.contains_key("sourceText") {
            canonical_text(entry, "sourceText");
        }
        assert_invalid_operation_shape(entry);
    }
}

pub(crate) fn assert_partition_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &["id", "output"],
            &["inputHex", "inputUtf8", "keyHex", "usesDefaultKey"],
        );
        case_id(entry);
        assert_exactly_one(entry, "inputHex", "inputUtf8");
        assert_exactly_one(entry, "keyHex", "usesDefaultKey");
        optional_hex(entry, "inputHex", None);
        optional_string(entry, "inputUtf8");
        optional_hex(entry, "keyHex", Some(32));
        if entry.contains_key("usesDefaultKey") {
            const_true(entry, "usesDefaultKey");
        }
        number_range(entry, "output", 0, 255);
    }
}

pub(crate) fn assert_compare_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &["id", "leftText", "rightText", "expected", "note"],
            &[],
        );
        case_id(entry);
        canonical_text(entry, "leftText");
        canonical_text(entry, "rightText");
        number_range(entry, "expected", -1, 1);
        nonempty_string(entry, "note");
    }
}

fn assert_invalid_operation_shape(entry: &crate::schema_support::Object) {
    match string_field(entry, "operation") {
        "alias" | "unalias" => {
            assert!(entry.contains_key("sourceText") && entry.contains_key("keyHex"));
            assert_exactly_one(entry, "tweakHex", "tweakUtf8");
        }
        "partition" => {
            assert!(entry.contains_key("keyHex"));
            assert_exactly_one(entry, "inputHex", "inputUtf8");
        }
        "fromBytes" => assert!(entry.contains_key("inputHex")),
        _ => unreachable!(),
    }
}

fn assert_exactly_one(entry: &crate::schema_support::Object, left: &str, right: &str) {
    assert_ne!(
        entry.contains_key(left),
        entry.contains_key(right),
        "fixture must contain exactly one of {left} and {right}"
    );
}

fn assert_case_id_value(entry: &crate::schema_support::Object, key: &str) {
    let value = string(get(entry, key));
    let mut synthetic = crate::schema_support::Object::new();
    synthetic.insert("id".to_string(), Json::String(value.to_string()));
    case_id(&synthetic);
}
