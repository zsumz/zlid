use std::collections::{BTreeMap, BTreeSet};

use crate::json::{array, get, number, object, string, Json};

const REQUIRED_CASE_IDS: &[(&str, &[&str])] = &[
    (
        "ordered",
        &["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8", "O9", "O10"],
    ),
    ("random", &["R1", "R2", "R3"]),
    (
        "alias",
        &["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9"],
    ),
    ("sentinels", &["S1", "S2"]),
    ("opaque", &["X1", "X2", "X3"]),
    ("friendlyParsing", &["F1", "F2", "F3", "F4"]),
    (
        "negativeParsing",
        &["N1", "N2", "N3", "N4", "N5", "N6", "N7"],
    ),
    (
        "invalidOperations",
        &["IO1", "IO2", "IO3", "IO4", "IO5", "IO6"],
    ),
    ("partition", &["P1", "P2", "P3", "P4", "P5", "P6"]),
    ("compare", &["C1", "C2", "C3", "C4", "C5", "C6"]),
    ("generated", &["PG1", "PG2", "PG3", "PG4"]),
    ("generator", &["G1", "G2", "G3", "G4"]),
];

pub(crate) fn assert_conformance_baseline(
    coverage: &BTreeMap<String, Json>,
    golden: &BTreeMap<String, Json>,
) {
    assert_eq!(1, number(get(coverage, "version")));
    assert_eq!(
        "conformance/zlid-v0.1-golden.json",
        string(get(coverage, "fixture"))
    );
    let required = object(get(coverage, "requiredCaseIds"));
    let expected_sections: BTreeSet<_> = REQUIRED_CASE_IDS
        .iter()
        .map(|(section, _)| *section)
        .collect();
    assert_eq!(
        REQUIRED_CASE_IDS.len(),
        expected_sections.len(),
        "code-owned baseline contains duplicate sections"
    );
    let actual_sections: BTreeSet<_> = required.keys().map(String::as_str).collect();
    assert_eq!(
        expected_sections, actual_sections,
        "coverage section baseline drifted"
    );

    let mut all_expected_ids = BTreeSet::new();
    let mut all_fixture_ids = BTreeSet::new();
    for (section, expected) in REQUIRED_CASE_IDS {
        assert!(
            !expected.is_empty(),
            "code-owned baseline {section} is empty"
        );
        assert_exact_ids(
            "coverage manifest",
            section,
            array(get(required, section)).iter().map(string),
            expected,
        );
        let fixture_ids: Vec<_> = fixture_entries(golden, section)
            .iter()
            .map(|entry| string(get(object(entry), "id")))
            .collect();
        assert_exact_ids("fixture", section, fixture_ids.iter().copied(), expected);
        for id in *expected {
            assert!(
                all_expected_ids.insert(*id),
                "code-owned case id {id} is duplicated across sections"
            );
        }
        for id in fixture_ids {
            assert!(
                all_fixture_ids.insert(id),
                "fixture case id {id} is duplicated across sections"
            );
        }
    }
}

fn assert_exact_ids<'a>(
    source: &str,
    section: &str,
    actual: impl Iterator<Item = &'a str>,
    expected: &[&str],
) {
    let actual: Vec<_> = actual.collect();
    assert!(!actual.is_empty(), "{source} section {section} is empty");
    let unique: BTreeSet<_> = actual.iter().copied().collect();
    assert_eq!(
        actual.len(),
        unique.len(),
        "{source} section {section} contains duplicate case ids"
    );
    let expected_count = expected.len();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(
        expected_count,
        expected.len(),
        "code-owned baseline section {section} contains duplicate case ids"
    );
    assert_eq!(expected, unique, "{source} section {section} ids drifted");
}

fn fixture_entries<'a>(golden: &'a BTreeMap<String, Json>, section: &str) -> &'a [Json] {
    match section {
        "partition" => array(get(object(get(golden, "partition")), "cases")),
        "generator" => array(get(object(get(golden, "generator")), "cases")),
        other => array(get(golden, other)),
    }
}
