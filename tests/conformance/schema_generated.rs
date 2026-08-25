use crate::json::{array, get, object, string, Json};
use crate::schema_support::{
    canonical_text, case_id, exact_keys, hex_field, hex_value, number_array, number_range,
    object_array, optional_number_range, profile, string_field, MAX_TIMESTAMP,
};

pub(crate) fn assert_generated_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &[
                "id",
                "profile",
                "defaultPartition",
                "nowMs",
                "entropyHex",
                "calls",
            ],
            &[
                "warmupCalls",
                "warmupNowMs",
                "warmupEntropyHex",
                "warmupLast",
            ],
        );
        case_id(entry);
        profile(entry);
        number_range(entry, "defaultPartition", 0, 255);
        number_array(entry, "nowMs", 0, MAX_TIMESTAMP);
        assert_entropy_array(entry);
        for call in object_array(entry, "calls") {
            assert_generated_call(object(call));
        }
        optional_number_range(entry, "warmupCalls", 0, i64::MAX);
        optional_number_range(entry, "warmupNowMs", 0, MAX_TIMESTAMP);
        if entry.contains_key("warmupEntropyHex") {
            hex_field(entry, "warmupEntropyHex", Some(14));
        }
        if let Some(value) = entry.get("warmupLast") {
            assert_generated_call(object(value));
        }
    }
}

pub(crate) fn assert_generator_cases(entries: &[Json]) {
    for value in entries {
        let entry = object(value);
        exact_keys(
            entry,
            &["id", "profile", "partition", "events"],
            &["nowMs", "constantNowMs", "warmupCalls", "warmupLast"],
        );
        case_id(entry);
        profile(entry);
        number_range(entry, "partition", 0, 255);
        assert_ne!(
            entry.contains_key("nowMs"),
            entry.contains_key("constantNowMs"),
            "generator case requires exactly one clock source"
        );
        if entry.contains_key("nowMs") {
            number_array(entry, "nowMs", 0, MAX_TIMESTAMP);
        }
        optional_number_range(entry, "constantNowMs", 0, MAX_TIMESTAMP);
        optional_number_range(entry, "warmupCalls", 0, i64::MAX);
        if let Some(value) = entry.get("warmupLast") {
            assert_event(object(value));
        }
        for event in object_array(entry, "events") {
            assert_event(object(event));
        }
    }
}

fn assert_generated_call(call: &crate::schema_support::Object) {
    exact_keys(
        call,
        &[
            "timestampMs",
            "partition",
            "sequence",
            "randomHex",
            "tag",
            "bytesHex",
            "text",
        ],
        &["partitionOverride"],
    );
    optional_number_range(call, "partitionOverride", 0, 255);
    number_range(call, "timestampMs", 0, MAX_TIMESTAMP);
    number_range(call, "partition", 0, 255);
    number_range(call, "sequence", 0, 65_535);
    let random_length = string_field(call, "randomHex").len();
    assert!([13, 14].contains(&random_length));
    hex_field(call, "randomHex", Some(random_length));
    number_range(call, "tag", 1, 4);
    hex_field(call, "bytesHex", Some(32));
    canonical_text(call, "text");
}

fn assert_event(event: &crate::schema_support::Object) {
    exact_keys(event, &["timestampMs", "partition", "sequence", "tag"], &[]);
    number_range(event, "timestampMs", 0, MAX_TIMESTAMP);
    number_range(event, "partition", 0, 255);
    number_range(event, "sequence", 0, 65_535);
    number_range(event, "tag", 1, 4);
}

fn assert_entropy_array(entry: &crate::schema_support::Object) {
    let values = array(get(entry, "entropyHex"));
    assert!(!values.is_empty(), "generated entropyHex must not be empty");
    for value in values {
        hex_value(string(value), "entropyHex", Some(14));
    }
}
