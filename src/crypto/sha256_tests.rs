use super::sha256;
use crate::bytes_to_hex;

#[test]
fn sha256_known_vector() {
    assert_eq!(
        bytes_to_hex(&sha256(b"abc")),
        "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
    );
}
