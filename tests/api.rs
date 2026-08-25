//! Public API behavior outside the shared wire-format fixture.

use zlid::{Inspection, Zlid};

#[test]
fn sentinel_text_round_trips() {
    assert_eq!(Zlid::NIL.text(), "00000000000000000000000000");
    assert_eq!(Zlid::MAX.text(), "7ZZZZZZZZZZZZZZZZZZZZZZZZZ");
    assert_eq!(Zlid::parse(&Zlid::MAX.text()).unwrap(), Zlid::MAX);
}

#[test]
fn display_and_from_str_share_canonical_text() {
    let parsed: Zlid = "01k2r7-kfwe58 07000000000001".parse().unwrap();
    assert_eq!(parsed.to_string(), "01K2R7KFWE5807000000000001");
}

#[test]
fn system_entropy_supplies_random_ids() -> zlid::Result<()> {
    assert!(matches!(
        Zlid::random()?.inspect(),
        Inspection::Random { .. }
    ));
    Ok(())
}

#[test]
fn shared_generator_supplies_ordered_ids() -> zlid::Result<()> {
    assert!(matches!(
        Zlid::next()?.inspect(),
        Inspection::Ordered { .. }
    ));
    Ok(())
}
