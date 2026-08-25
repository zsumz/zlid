use std::collections::{BTreeMap, BTreeSet};

use crate::json::{array, get, object, string, Json};

pub(crate) fn assert_required_case_ids(
    required_case_ids: &BTreeMap<String, Json>,
    golden: &BTreeMap<String, Json>,
) {
    for (section, ids) in required_case_ids {
        let present = fixture_case_ids(golden, section);
        for id in array(ids) {
            let id = string(id);
            assert!(
                present.contains(id),
                "coverage manifest requires {section}.{id} but SDK fixture inputs do not include it"
            );
        }
    }
}

fn fixture_case_ids(golden: &BTreeMap<String, Json>, section: &str) -> BTreeSet<String> {
    let entries = match section {
        "partition" => array(get(object(get(golden, "partition")), "cases")),
        "generator" => array(get(object(get(golden, "generator")), "cases")),
        other => array(get(golden, other)),
    };
    entries
        .iter()
        .map(|entry| string(get(object(entry), "id")).to_string())
        .collect()
}
