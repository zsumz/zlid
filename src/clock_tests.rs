use super::checked_millis;
use crate::constants::MAX_TS;
use crate::Error;

#[test]
fn validates_the_complete_wire_timestamp_range() {
    assert_eq!(checked_millis(0).unwrap(), 0);
    assert_eq!(checked_millis(u128::from(MAX_TS)).unwrap(), MAX_TS);
    assert!(matches!(
        checked_millis(u128::from(MAX_TS) + 1),
        Err(Error::Clock(_))
    ));
}
