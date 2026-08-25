use std::collections::BTreeMap;

use crate::json::{get, object, string, Json};
use crate::schema_cases::{
    assert_alias_cases, assert_compare_cases, assert_invalid_operation_cases, assert_ordered_cases,
    assert_parse_cases, assert_partition_cases, assert_random_cases,
    assert_sentinel_and_opaque_cases,
};
use crate::schema_generated::{assert_generated_cases, assert_generator_cases};
use crate::schema_support::{entries, exact_keys, nonempty_string, object_field};

const ROOT_FIELDS: &[&str] = &[
    "dataset",
    "specVersion",
    "release",
    "ordered",
    "random",
    "aliasDomain",
    "alias",
    "sentinels",
    "opaque",
    "friendlyParsing",
    "negativeParsing",
    "invalidOperations",
    "partition",
    "compare",
    "generated",
    "generator",
];

pub(crate) fn assert_fixture_schema(golden: &BTreeMap<String, Json>) {
    exact_keys(golden, ROOT_FIELDS, &[]);
    nonempty_string(golden, "dataset");
    assert_eq!("v0.1", string(get(golden, "specVersion")));
    nonempty_string(golden, "release");

    assert_ordered_cases(entries(golden, "ordered"));
    assert_random_cases(entries(golden, "random"));
    assert_alias_domain(object_field(golden, "aliasDomain"));
    assert_alias_cases(entries(golden, "alias"));
    assert_sentinel_and_opaque_cases(entries(golden, "sentinels"), entries(golden, "opaque"));
    assert_parse_cases(
        entries(golden, "friendlyParsing"),
        entries(golden, "negativeParsing"),
    );
    assert_invalid_operation_cases(entries(golden, "invalidOperations"));
    assert_partition_wrapper(object_field(golden, "partition"));
    assert_compare_cases(entries(golden, "compare"));
    assert_generated_cases(entries(golden, "generated"));
    assert_generator_wrapper(object_field(golden, "generator"));
}

pub(crate) fn assert_coverage_schema(coverage: &BTreeMap<String, Json>) {
    exact_keys(coverage, &["version", "fixture", "requiredCaseIds"], &[]);
    object(get(coverage, "requiredCaseIds"));
}

fn assert_alias_domain(domain: &BTreeMap<String, Json>) {
    use crate::schema_support::{hex_field, string_field};

    exact_keys(domain, &["keyHex", "tweakUtf8"], &[]);
    hex_field(domain, "keyHex", Some(32));
    string_field(domain, "tweakUtf8");
}

fn assert_partition_wrapper(partition: &BTreeMap<String, Json>) {
    exact_keys(partition, &["cases"], &[]);
    assert_partition_cases(crate::schema_support::object_array(partition, "cases"));
}

fn assert_generator_wrapper(generator: &BTreeMap<String, Json>) {
    exact_keys(generator, &["cases"], &[]);
    assert_generator_cases(crate::schema_support::object_array(generator, "cases"));
}
