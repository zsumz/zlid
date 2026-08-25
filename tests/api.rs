//! Public API behavior outside the shared wire-format fixture.

use std::hash::Hash;
use std::mem::size_of;

use zlid::{
    bytes_from_hex, partition_bytes, partition_str, ClockState, Error, Inspection,
    OrderedGenerator, Profile, SentinelName, Zlid, ZLID,
};

fn assert_value_traits<T: Copy + Eq + Ord + Hash + Send + Sync>() {}

#[test]
fn uppercase_name_is_primary_and_original_spelling_stays_compatible() {
    assert_value_traits::<ZLID>();
    assert_eq!(size_of::<ZLID>(), 16);
    assert_eq!(std::any::type_name::<ZLID>(), "zlid::ZLID");

    let primary: ZLID = Zlid::NIL;
    let compatible: Zlid = ZLID::MAX;
    assert_eq!(primary, ZLID::NIL);
    assert_eq!(compatible, Zlid::MAX);
    assert_eq!(
        format!("{primary:?}"),
        "ZLID(\"00000000000000000000000000\")"
    );
}

#[test]
fn owned_and_borrowed_bytes_preserve_the_exact_value() {
    let bytes = [0x5a; 16];
    let id = ZLID::from_array(bytes);
    assert_eq!(id.bytes(), bytes);
    assert_eq!(id.as_bytes(), &bytes);
    assert_eq!(ZLID::from_bytes(&bytes).unwrap(), id);
    assert!(matches!(
        ZLID::from_bytes(&bytes[..15]),
        Err(Error::InvalidLength {
            what: "ZLID",
            expected: 16,
            actual: 15
        })
    ));
}

#[test]
fn sentinel_text_round_trips() {
    assert_eq!(ZLID::NIL.text(), "00000000000000000000000000");
    assert_eq!(ZLID::MAX.text(), "7ZZZZZZZZZZZZZZZZZZZZZZZZZ");
    assert_eq!(ZLID::parse(&ZLID::MAX.text()).unwrap(), ZLID::MAX);
}

#[test]
fn display_and_from_str_share_canonical_text() {
    let parsed: ZLID = "01k2r7-kfwe58 07000000000001".parse().unwrap();
    assert_eq!(parsed.to_string(), "01K2R7KFWE5807000000000001");
}

#[test]
fn friendly_parsing_ignores_only_specified_ascii_separators() {
    let canonical = "01K2R7KFWE5807000000000001";
    let expected = ZLID::parse(canonical).unwrap();

    for separator in ["-", "_", " "] {
        let friendly = format!("{}{}{}", &canonical[..13], separator, &canonical[13..]);
        assert_eq!(ZLID::parse(&friendly).unwrap(), expected);
    }

    for whitespace in [
        "\t", "\n", "\r", "\u{000b}", "\u{000c}", "\u{0085}", "\u{00a0}", "\u{1680}", "\u{2003}",
        "\u{2028}", "\u{2029}", "\u{202f}", "\u{205f}", "\u{3000}",
    ] {
        let input = format!("{}{}{}", &canonical[..13], whitespace, &canonical[13..]);
        assert!(matches!(ZLID::parse(&input), Err(Error::InvalidText(_))));
    }
}

#[test]
fn stable_public_names_and_errors_are_explicit() {
    assert_eq!(Profile::Default.wire_name(), "default");
    assert_eq!(Profile::HighThroughput.wire_name(), "high-throughput");
    assert_eq!(
        Profile::from_wire_name("high-throughput").unwrap(),
        Profile::HighThroughput
    );
    assert!(matches!(
        Profile::from_wire_name("fast"),
        Err(Error::InvalidText(_))
    ));
    assert_eq!(ClockState::Normal.wire_name(), "normal");
    assert_eq!(ClockState::Clamped.wire_name(), "clamped");
    assert_eq!(SentinelName::Nil.wire_name(), "NIL");
    assert_eq!(SentinelName::Max.wire_name(), "MAX");

    assert!(bytes_from_hex("0").is_err());
    assert!(bytes_from_hex("GG").is_err());
    assert!(bytes_from_hex("0G").is_err());
    assert_eq!(
        partition_str("tenant", None),
        partition_bytes(b"tenant", None)
    );
    assert_eq!(
        Error::InvalidLength {
            what: "key",
            expected: 16,
            actual: 3,
        }
        .to_string(),
        "key must be exactly 16 bytes, got 3"
    );
    assert_eq!(
        Error::InvalidText("bad".to_string()).to_string(),
        "invalid ZLID text: bad"
    );
    assert_eq!(Error::OutOfRange("too large").to_string(), "too large");
    assert_eq!(
        Error::InvalidFamily("wrong family").to_string(),
        "wrong family"
    );
    assert_eq!(
        Error::Random("offline".to_string()).to_string(),
        "random source error: offline"
    );
    assert_eq!(
        Error::Clock("regressed".to_string()).to_string(),
        "clock error: regressed"
    );
}

#[test]
fn generator_convenience_constructors_preserve_profiles() {
    assert_eq!(ZLID::default_generator().profile(), Profile::Default);
    assert_eq!(
        ZLID::generator_for_profile(Profile::HighThroughput).profile(),
        Profile::HighThroughput
    );
    assert_eq!(
        ZLID::generator(Profile::Default, 17).profile(),
        Profile::Default
    );

    let mut generator =
        OrderedGenerator::with_sources(Profile::HighThroughput, 17, || 1_000, |size| vec![0; size]);
    let event = generator.next_event(None).unwrap();
    assert_eq!(
        (
            event.timestamp_ms,
            event.partition,
            event.sequence,
            event.tag
        ),
        (1_000, 17, 0, 2)
    );
}

#[test]
fn system_entropy_supplies_random_ids() -> zlid::Result<()> {
    assert!(matches!(
        ZLID::random()?.inspect(),
        Inspection::Random { .. }
    ));
    Ok(())
}

#[test]
fn shared_generator_supplies_ordered_ids() -> zlid::Result<()> {
    assert!(matches!(
        ZLID::next()?.inspect(),
        Inspection::Ordered { .. }
    ));
    Ok(())
}
