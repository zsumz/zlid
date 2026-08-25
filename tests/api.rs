//! Public API behavior outside the shared wire-format fixture.

use std::hash::Hash;
use std::mem::{align_of, size_of};

use zlid::{
    advanced::OrderedGeneratorCore, wire::ClockState, Error, Inspection, OrderedGenerator, Profile,
    SentinelName, ZLID,
};

fn assert_value_traits<T: Copy + Eq + Ord + Hash + Send + Sync>() {}

#[test]
fn uppercase_name_and_representation_are_explicit() {
    assert_value_traits::<ZLID>();
    assert_eq!(size_of::<ZLID>(), ZLID::BYTE_LENGTH);
    assert_eq!(align_of::<ZLID>(), align_of::<[u8; 16]>());
    assert_eq!(ZLID::BYTE_LENGTH, 16);
    assert_eq!(ZLID::TEXT_LENGTH, 26);
    assert_eq!(std::any::type_name::<ZLID>(), "zlid::ZLID");
    assert_eq!(
        format!("{:?}", ZLID::NIL),
        "ZLID(\"00000000000000000000000000\")"
    );
}

#[test]
fn owned_and_borrowed_bytes_preserve_the_exact_value() {
    let bytes = [0x5a; ZLID::BYTE_LENGTH];
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
fn canonical_and_friendly_parsing_have_distinct_contracts() {
    let canonical = "01K2R7KFWE5807000000000001";
    let expected = ZLID::parse_canonical(canonical).unwrap();
    assert_eq!(expected.text(), canonical);
    assert_eq!(canonical.parse::<ZLID>().unwrap(), expected);

    for friendly in [
        "01k2r7kfwe5807000000000001",
        "01K2R7KF-WE580_7000000000001",
        "O1K2R7KFWE58 07OOOOOOOOOOO1",
    ] {
        assert_eq!(ZLID::parse(friendly).unwrap(), expected);
        assert!(matches!(
            ZLID::parse_canonical(friendly),
            Err(Error::InvalidText(_))
        ));
    }

    for rejected in [
        "81K2R7KFWE5807000000000001",
        "01K2R7KFWE580700000000000U",
        "01K2R7KFWE580700000000000",
        "01K2R7KFWE58070000000000010",
    ] {
        assert!(matches!(
            ZLID::parse_canonical(rejected),
            Err(Error::InvalidText(_))
        ));
    }
}

#[test]
fn friendly_parsing_ignores_only_specified_ascii_separators() {
    let canonical = "01K2R7KFWE5807000000000001";
    let expected = ZLID::parse_canonical(canonical).unwrap();

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
fn semantic_errors_are_programmatically_distinct() {
    assert_eq!(
        Profile::from_wire_name("high-throughput").unwrap(),
        Profile::HighThroughput
    );
    assert!(matches!(
        Profile::from_wire_name("fast"),
        Err(Error::UnknownProfile(profile)) if profile == "fast"
    ));
    assert!(matches!(
        ZLID::NIL.alias(b"", b""),
        Err(Error::EmptyAliasKey)
    ));
    assert!(matches!(
        ZLID::NIL.alias(b"key", &vec![0; 65_536]),
        Err(Error::TweakTooLong {
            maximum: 65_535,
            actual: 65_536
        })
    ));
    assert!(matches!(
        ZLID::NIL.alias(b"key", b""),
        Err(Error::InvalidTag {
            operation: "alias",
            expected: "an ordered ZLID tag",
            actual: 0,
        })
    ));

    assert_eq!(
        Error::UnknownProfile("fast".to_string()).to_string(),
        "unknown ZLID profile \"fast\""
    );
    assert_eq!(
        Error::InvalidLength {
            what: "partition key",
            expected: 16,
            actual: 15,
        }
        .to_string(),
        "partition key must be exactly 16 bytes, got 15"
    );
    assert_eq!(
        Error::InvalidText("bad alphabet".to_string()).to_string(),
        "invalid ZLID text: bad alphabet"
    );
    assert_eq!(
        Error::EmptyAliasKey.to_string(),
        "alias key must not be empty"
    );
    assert_eq!(
        Error::TweakTooLong {
            maximum: 65_535,
            actual: 65_536,
        }
        .to_string(),
        "alias tweak must be at most 65535 bytes, got 65536"
    );
    assert_eq!(
        Error::InvalidTag {
            operation: "alias",
            expected: "an ordered ZLID tag",
            actual: 5,
        }
        .to_string(),
        "alias expected an ordered ZLID tag, got ZLID tag 0x5"
    );
    assert_eq!(
        Error::FieldOutOfRange {
            field: "sequence",
            maximum: 4_095,
            actual: 4_096,
        }
        .to_string(),
        "ZLID field sequence must be at most 4095, got 4096"
    );
    assert_eq!(
        Error::EntropyUnavailable("offline".to_string()).to_string(),
        "entropy unavailable: offline"
    );
    assert_eq!(
        Error::Clock("before epoch".to_string()).to_string(),
        "clock error: before epoch"
    );
    assert_eq!(
        Error::GeneratorPoisoned.to_string(),
        "shared ZLID generator mutex is poisoned"
    );
}

#[test]
fn primary_and_advanced_generator_paths_are_deliberate() {
    assert_eq!(OrderedGenerator::default().profile(), Profile::Default);
    assert_eq!(
        OrderedGenerator::with_profile(Profile::HighThroughput).profile(),
        Profile::HighThroughput
    );
    assert_eq!(
        OrderedGenerator::new(Profile::Default, 17).profile(),
        Profile::Default
    );

    let mut core = OrderedGeneratorCore::new(Profile::HighThroughput, 17, || 1_000);
    let event = core.next(None).unwrap();
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
fn stable_wire_names_are_explicit() {
    assert_eq!(Profile::Default.wire_name(), "default");
    assert_eq!(Profile::HighThroughput.wire_name(), "high-throughput");
    assert_eq!(ClockState::Normal.wire_name(), "normal");
    assert_eq!(ClockState::Clamped.wire_name(), "clamped");
    assert_eq!(SentinelName::Nil.wire_name(), "NIL");
    assert_eq!(SentinelName::Max.wire_name(), "MAX");
    assert_eq!(
        ZLID::partition_str("tenant", None),
        ZLID::partition_bytes(b"tenant", None)
    );
}

#[test]
fn system_sources_supply_public_identifier_families() -> zlid::Result<()> {
    assert!(matches!(
        ZLID::random()?.inspect(),
        Inspection::Random { .. }
    ));
    assert!(matches!(
        ZLID::next()?.inspect(),
        Inspection::Ordered { .. }
    ));
    Ok(())
}
