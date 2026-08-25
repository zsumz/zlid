use super::sha256;
use crate::bytes_to_hex;

const GOLDEN_FIXTURE: &[u8] = include_bytes!("../../conformance/zlid-v0.1-golden.json");
const GOLDEN_FIXTURE_SHA256: &str =
    "D36DE5FACB5D00A38FCC86930096DAB55198C8483CB5558F7C5E76628B1E9A96";

#[test]
fn sha256_known_vector() {
    assert_eq!(
        bytes_to_hex(&sha256(b"")),
        "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
    );
    assert_eq!(
        bytes_to_hex(&sha256(b"abc")),
        "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
    );
}

#[test]
fn golden_conformance_fixture_sha256_is_pinned_in_code() {
    assert_eq!(bytes_to_hex(&sha256(GOLDEN_FIXTURE)), GOLDEN_FIXTURE_SHA256);
}
