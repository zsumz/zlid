use super::Inspection;
use crate::classification::Family;
use crate::ZLID;

#[test]
fn random_inspection_retains_complete_and_payload_hex() {
    let value = ZLID::from_array([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
        0xf5,
    ]);

    let Inspection::Random {
        bytes_hex,
        random_hex,
        ..
    } = value.inspect()
    else {
        panic!("expected random inspection");
    };
    assert_eq!(bytes_hex, "0123456789ABCDEF1032547698BADCF5");
    assert_eq!(random_hex, "0123456789ABCDEF1032547698BADCF");
}

#[test]
fn alias_inspection_retains_complete_and_payload_hex() {
    let value = ZLID::from_array([
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23,
        0x16,
    ]);

    let Inspection::Alias {
        bytes_hex,
        alias_data_hex,
        ..
    } = value.inspect()
    else {
        panic!("expected alias inspection");
    };
    assert_eq!(bytes_hex, "FEDCBA9876543210EFCDAB8967452316");
    assert_eq!(alias_data_hex, "FEDCBA9876543210EFCDAB896745231");
}

#[test]
fn inspection_reports_semantic_families() {
    let ordered = ZLID::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    let random = ZLID::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5]);
    let alias = ZLID::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6]);
    let opaque = ZLID::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10]);

    assert_eq!(ordered.inspect().family(), Some(Family::Ordered));
    assert_eq!(random.inspect().family(), Some(Family::Random));
    assert_eq!(alias.inspect().family(), Some(Family::Alias));
    assert_eq!(opaque.inspect().family(), None);
    assert_eq!(ZLID::NIL.inspect().family(), None);
    assert_eq!(ZLID::MAX.inspect().family(), None);
}
