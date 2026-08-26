//! Public human-readable text and ordering contracts.

use zlid::{Error, ZLID};

const CANONICAL: &str = "01K2R7KFWE5807000000000001";

#[test]
fn canonical_text_is_available_through_standard_rust_formatting() {
    let value = ZLID::parse_canonical(CANONICAL).unwrap();

    assert_eq!(value.text(), CANONICAL);
    assert_eq!(value.to_string(), CANONICAL);
    assert_eq!(format!("{value}"), CANONICAL);
}

#[test]
fn from_str_is_friendly_while_canonical_parsing_is_strict() {
    let value = ZLID::parse_canonical(CANONICAL).unwrap();
    let friendly = "O1k2r7-kfwe58_07OOOOOOOOOOO1";

    assert_eq!(friendly.parse::<ZLID>().unwrap(), value);
    assert!(matches!(
        ZLID::parse_canonical(friendly),
        Err(Error::InvalidText(_))
    ));

    for noncanonical in [
        "01k2r7kfwe5807000000000001",
        "01K2R7KF-WE5807000000000001",
        "O1K2R7KFWE5807000000000001",
    ] {
        assert!(matches!(
            ZLID::parse_canonical(noncanonical),
            Err(Error::InvalidText(_))
        ));
    }
}

#[test]
fn canonical_text_order_matches_zlid_and_byte_order() {
    let mut values = deterministic_values();
    values.sort_unstable();

    let text_in_zlid_order: Vec<_> = values.iter().map(ZLID::text).collect();
    let mut text_in_lexical_order = text_in_zlid_order.clone();
    text_in_lexical_order.sort_unstable();

    assert_eq!(text_in_zlid_order, text_in_lexical_order);
    for pair in values.windows(2) {
        assert_eq!(pair[0].cmp(&pair[1]), pair[0].bytes().cmp(&pair[1].bytes()));
    }
}

fn deterministic_values() -> Vec<ZLID> {
    let mut state = 0xA076_1D64_78BD_642Fu64;
    let mut values = vec![ZLID::NIL, ZLID::MAX];

    for _ in 0..1_024 {
        let mut bytes = [0u8; ZLID::BYTE_LENGTH];
        for chunk in bytes.as_chunks_mut::<8>().0 {
            state = next_value(state);
            chunk.copy_from_slice(&state.to_be_bytes());
        }
        values.push(ZLID::from_array(bytes));
    }

    values
}

fn next_value(mut value: u64) -> u64 {
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}
