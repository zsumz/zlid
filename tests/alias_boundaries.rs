//! Boundary and family-matrix coverage for reversible aliases.

use zlid::{pack_ordered, Error, Profile, ZLID};

const SOURCE_CASES: [(Profile, u8, u8); 4] = [
    (Profile::Default, 1, 6),
    (Profile::Default, 3, 7),
    (Profile::HighThroughput, 2, 8),
    (Profile::HighThroughput, 4, 9),
];

#[test]
fn every_ordered_tag_round_trips_at_hmac_key_and_tweak_boundaries() {
    let tweaks = [Vec::new(), vec![0xA5; 65_535]];

    for (profile, source_tag, alias_tag) in SOURCE_CASES {
        let source = ordered_source(profile, source_tag);
        for key_length in [1, 63, 64, 65] {
            let key = vec![key_length as u8; key_length];
            for tweak in &tweaks {
                let alias = source.alias(&key, tweak).expect("accepted alias inputs");
                assert_eq!(alias.tag(), alias_tag);
                assert_eq!(
                    alias.unalias(&key, tweak).expect("accepted unalias inputs"),
                    source
                );
            }
        }
    }
}

#[test]
fn tweak_limit_is_measured_in_bytes_including_for_utf8_helpers() {
    let source = ordered_source(Profile::Default, 1);
    let key = b"k";
    let accepted_text = format!("{}a", "é".repeat(32_767));
    let rejected_text = format!("{accepted_text}b");
    assert_eq!(accepted_text.len(), 65_535);
    assert_eq!(rejected_text.len(), 65_536);

    let alias = source
        .alias_str(key, &accepted_text)
        .expect("65,535 UTF-8 bytes are valid");
    assert_eq!(
        alias
            .unalias_str(key, &accepted_text)
            .expect("65,535 UTF-8 bytes are valid"),
        source
    );
    assert_invalid_family(source.alias_str(key, &rejected_text));
    assert_invalid_family(alias.unalias_str(key, &rejected_text));

    let oversized = vec![0; 65_536];
    for (profile, source_tag, _) in SOURCE_CASES {
        let source = ordered_source(profile, source_tag);
        let alias = source.alias(key, b"").expect("valid alias");
        assert_invalid_family(source.alias(key, &oversized));
        assert_invalid_family(alias.unalias(key, &oversized));
    }
}

#[test]
fn alias_and_unalias_accept_exactly_their_defined_tag_families() {
    let key = b"family-matrix-key";
    for tag in 0..=15u8 {
        let value = value_with_tag(tag);
        assert_eq!(
            value.alias(key, b"").is_ok(),
            [1, 2, 3, 4].contains(&tag),
            "alias acceptance for tag {tag}"
        );
        assert_eq!(
            value.unalias(key, b"").is_ok(),
            [6, 7, 8, 9].contains(&tag),
            "unalias acceptance for tag {tag}"
        );
    }

    for sentinel in [ZLID::NIL, ZLID::MAX] {
        assert_invalid_family(sentinel.alias(key, b""));
        assert_invalid_family(sentinel.unalias(key, b""));
    }
}

#[test]
fn empty_alias_keys_are_rejected_for_both_directions() {
    let source = ordered_source(Profile::Default, 1);
    let alias = source.alias(b"valid", b"").expect("valid alias");

    assert_invalid_family(source.alias(b"", b""));
    assert_invalid_family(alias.unalias(b"", b""));
}

#[test]
fn alias_domains_are_deterministic_and_keyed() {
    let source = ordered_source(Profile::Default, 1);
    let baseline = source.alias(b"primary-key", b"users|prod").unwrap();

    assert_eq!(
        source.alias(b"primary-key", b"users|prod").unwrap(),
        baseline
    );
    assert_ne!(
        source.alias(b"alternate-key", b"users|prod").unwrap(),
        baseline
    );
    assert_ne!(
        source.alias(b"primary-key", b"orders|prod").unwrap(),
        baseline
    );
}

fn ordered_source(profile: Profile, tag: u8) -> ZLID {
    ZLID::from_array(
        pack_ordered(profile, 0x1234_5678_9ABC, 0x5A, 7, 11, tag).expect("valid ordered source"),
    )
}

fn value_with_tag(tag: u8) -> ZLID {
    let mut bytes = [0x5A; 16];
    bytes[15] = (bytes[15] & 0xF0) | tag;
    ZLID::from_array(bytes)
}

fn assert_invalid_family(result: zlid::Result<ZLID>) {
    assert!(matches!(result, Err(Error::InvalidFamily(_))));
}
