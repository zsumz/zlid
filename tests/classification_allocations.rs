//! Allocation contracts for strict parsing and direct classification.

#![cfg(not(target_arch = "wasm32"))]

use std::hint::black_box;

use allocation_counter::measure;
use zlid::ZLID;

const CANONICAL: &str = "01K2R7KFWE5807000000000001";

#[test]
fn canonical_parsing_and_direct_classification_do_not_allocate() {
    let parse = measure(|| {
        black_box(ZLID::parse_canonical(black_box(CANONICAL)).unwrap());
    });
    assert_eq!(parse.count_total, 0, "canonical parse: {parse:?}");

    let id = ZLID::parse_canonical(CANONICAL).unwrap();
    let kind = measure(|| {
        black_box(black_box(id).kind());
    });
    assert_eq!(kind.count_total, 0, "kind: {kind:?}");

    let family = measure(|| {
        black_box(black_box(id).family());
    });
    assert_eq!(family.count_total, 0, "family: {family:?}");
}
