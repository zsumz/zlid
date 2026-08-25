//! Runtime qualification for the JavaScript clock and entropy adapters.

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use wasm_bindgen_test::wasm_bindgen_test;
use zlid::{InspectionKind, ZLID};

#[wasm_bindgen_test]
fn javascript_clock_and_entropy_support_public_generation() {
    assert_eq!(
        ZLID::next().unwrap().inspect().kind(),
        InspectionKind::Ordered
    );
    assert_eq!(
        ZLID::random().unwrap().inspect().kind(),
        InspectionKind::Random
    );
}
