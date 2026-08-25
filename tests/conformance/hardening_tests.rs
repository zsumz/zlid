use std::collections::BTreeMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use crate::coverage::assert_conformance_baseline;
use crate::json::{Json, JsonParser};
use crate::schema::assert_fixture_schema;

#[test]
fn baseline_rejects_fixture_and_manifest_that_shrink_together() {
    let mut golden = load_json("conformance/zlid-v0.1-golden.json");
    let mut coverage = load_json("conformance/coverage.json");
    let random_cases = array_mut(golden.get_mut("random").unwrap());
    random_cases.retain(|entry| {
        !matches!(
            entry,
            Json::Object(case)
                if matches!(case.get("id"), Some(Json::String(id)) if id == "R1")
        )
    });
    assert_eq!(random_cases.len(), 2);

    let required = object_mut(coverage.get_mut("requiredCaseIds").unwrap());
    let random_ids = array_mut(required.get_mut("random").unwrap());
    random_ids.retain(|entry| !matches!(entry, Json::String(id) if id == "R1"));
    assert_eq!(random_ids.len(), 2);

    assert_rejected(|| assert_conformance_baseline(&coverage, &golden));
}

#[test]
fn baseline_rejects_duplicate_case_ids() {
    let golden = load_json("conformance/zlid-v0.1-golden.json");
    let mut coverage = load_json("conformance/coverage.json");
    let required = object_mut(coverage.get_mut("requiredCaseIds").unwrap());
    array_mut(required.get_mut("random").unwrap()).push(Json::String("R1".to_string()));

    assert_rejected(|| assert_conformance_baseline(&coverage, &golden));
}

#[test]
fn schema_rejects_false_default_key_marker() {
    let mut golden = load_json("conformance/zlid-v0.1-golden.json");
    partition_case_mut(&mut golden, "P5").insert("usesDefaultKey".to_string(), Json::Bool(false));

    assert_rejected(|| assert_fixture_schema(&golden));
}

#[test]
fn schema_rejects_numeric_value_that_would_truncate() {
    let mut golden = load_json("conformance/zlid-v0.1-golden.json");
    partition_case_mut(&mut golden, "P5").insert("output".to_string(), Json::Number(329));

    assert_rejected(|| assert_fixture_schema(&golden));
}

#[test]
fn schema_rejects_unknown_root_fields() {
    let mut golden = load_json("conformance/zlid-v0.1-golden.json");
    golden.insert("unexpected".to_string(), Json::Null);

    assert_rejected(|| assert_fixture_schema(&golden));
}

#[test]
fn parser_rejects_duplicate_object_keys() {
    assert!(JsonParser::parse(r#"{"version":1,"version":1}"#).is_err());
}

#[test]
fn parser_rejects_all_unescaped_control_characters() {
    for code in 0..=0x1f {
        let control = char::from_u32(code).unwrap();
        let input = format!("{{\"value\":\"{control}\"}}");
        assert!(
            JsonParser::parse(&input).is_err(),
            "accepted raw U+{code:04X}"
        );
    }
}

fn load_json(relative: &str) -> BTreeMap<String, Json> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let input = fs::read_to_string(path).unwrap();
    JsonParser::parse(&input).unwrap().into_object()
}

fn partition_case_mut<'a>(
    root: &'a mut BTreeMap<String, Json>,
    id: &str,
) -> &'a mut BTreeMap<String, Json> {
    let partition = object_mut(root.get_mut("partition").unwrap());
    let cases = array_mut(partition.get_mut("cases").unwrap());
    let entry = cases
        .iter_mut()
        .find(|entry| match entry {
            Json::Object(entry) => {
                matches!(entry.get("id"), Some(Json::String(actual)) if actual == id)
            }
            _ => false,
        })
        .unwrap();
    object_mut(entry)
}

fn object_mut(value: &mut Json) -> &mut BTreeMap<String, Json> {
    match value {
        Json::Object(value) => value,
        other => panic!("expected object, got {other:?}"),
    }
}

fn array_mut(value: &mut Json) -> &mut Vec<Json> {
    match value {
        Json::Array(value) => value,
        other => panic!("expected array, got {other:?}"),
    }
}

fn assert_rejected(action: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(action)).is_err());
}
