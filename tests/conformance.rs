//! ZLID v0.1 conformance suite backed by the packaged golden fixture.

use std::fs;
use std::path::PathBuf;

#[path = "conformance/basic_cases.rs"]
mod basic_cases;
#[path = "conformance/core_cases.rs"]
mod core_cases;
#[path = "conformance/coverage.rs"]
mod coverage;
#[path = "conformance/generated_cases.rs"]
mod generated_cases;
#[path = "conformance/hardening_tests.rs"]
mod hardening_tests;
#[path = "conformance/helpers.rs"]
mod helpers;
#[path = "conformance/json.rs"]
mod json;
#[path = "conformance/schema.rs"]
mod schema;
#[path = "conformance/schema_cases.rs"]
mod schema_cases;
#[path = "conformance/schema_generated.rs"]
mod schema_generated;
#[path = "conformance/schema_support.rs"]
mod schema_support;

use basic_cases::{
    assert_compare_section, assert_friendly_parsing, assert_invalid_operations,
    assert_negative_parsing, assert_opaque_section, assert_partition_section,
    assert_sentinel_section,
};
use core_cases::{assert_alias_section, assert_ordered_section, assert_random_section};
use coverage::assert_conformance_baseline;
use generated_cases::{assert_generated_section, assert_generator_section};
use json::{array, get, object, string, JsonParser};
use schema::{assert_coverage_schema, assert_fixture_schema};

#[test]
fn shared_conformance_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = root.join("conformance").join("zlid-v0.1-golden.json");
    let fixture = fs::read_to_string(&fixture_path).expect("read shared conformance fixture");
    let golden = JsonParser::parse(&fixture)
        .expect("parse shared conformance fixture")
        .into_object();
    let manifest_path = root.join("conformance").join("coverage.json");
    let manifest = fs::read_to_string(&manifest_path).expect("read coverage manifest");
    let coverage = JsonParser::parse(&manifest)
        .expect("parse coverage manifest")
        .into_object();

    assert_eq!(env!("CARGO_PKG_VERSION"), string(get(&golden, "release")));
    assert_fixture_schema(&golden);
    assert_coverage_schema(&coverage);
    assert_conformance_baseline(&coverage, &golden);

    assert_ordered_section(array(get(&golden, "ordered")));
    assert_random_section(array(get(&golden, "random")));
    assert_alias_section(
        object(get(&golden, "aliasDomain")),
        array(get(&golden, "alias")),
    );
    assert_sentinel_section(array(get(&golden, "sentinels")));
    assert_opaque_section(array(get(&golden, "opaque")));
    assert_friendly_parsing(array(get(&golden, "friendlyParsing")));
    assert_negative_parsing(array(get(&golden, "negativeParsing")));
    assert_invalid_operations(array(get(&golden, "invalidOperations")));
    assert_partition_section(object(get(&golden, "partition")));
    assert_compare_section(array(get(&golden, "compare")));
    assert_generated_section(array(get(&golden, "generated")));
    assert_generator_section(object(get(&golden, "generator")));
}
