use super::{classify_bytes, Family};
use crate::inspection::InspectionKind;

#[test]
fn every_tag_has_the_expected_classification() {
    for tag in 0u8..=15 {
        let mut bytes = [0xa5; 16];
        bytes[15] = (bytes[15] & 0xf0) | tag;
        let expected = match tag {
            1..=4 => (InspectionKind::Ordered, Some(Family::Ordered)),
            5 => (InspectionKind::Random, Some(Family::Random)),
            6..=9 => (InspectionKind::Alias, Some(Family::Alias)),
            _ => (InspectionKind::Opaque, None),
        };
        let actual = classify_bytes(&bytes);
        assert_eq!((actual.kind, actual.family), expected, "tag {tag}");
    }
}

#[test]
fn sentinels_take_priority_over_their_tag_nibbles() {
    for bytes in [[0; 16], [u8::MAX; 16]] {
        let actual = classify_bytes(&bytes);
        assert_eq!(actual.kind, InspectionKind::Sentinel);
        assert_eq!(actual.family, None);
    }
}

#[test]
fn family_names_are_stable() {
    assert_eq!(Family::Ordered.wire_name(), "ZLID");
    assert_eq!(Family::Random.wire_name(), "ZLID-R");
    assert_eq!(Family::Alias.wire_name(), "ZLID-A");
}
