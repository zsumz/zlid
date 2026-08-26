//! Integration tests for ZLID's optional Serde data-model contract.
#![cfg(feature = "serde")]

use zlid::ZLID;

const CANONICAL: &str = "01K2R7KFWE5807000000000001";

#[test]
fn json_uses_the_canonical_text_form() {
    let zlid = ZLID::parse_canonical(CANONICAL).unwrap();

    let encoded = serde_json::to_string(&zlid).unwrap();
    assert_eq!(encoded, format!("\"{CANONICAL}\""));
    assert_eq!(serde_json::from_str::<ZLID>(&encoded).unwrap(), zlid);
}

#[test]
fn json_deserialization_is_strictly_canonical() {
    let friendly_forms = [
        "01k2r7kfwe5807000000000001",
        "01K2R7-KFWE5807000000000001",
        "O1K2R7KFWE5807000000000001",
    ];
    for value in friendly_forms {
        let encoded = format!("\"{value}\"");
        let error = serde_json::from_str::<ZLID>(&encoded).unwrap_err();
        assert!(
            error.to_string().contains("invalid ZLID text"),
            "unexpected error for {value:?}: {error}"
        );
    }
}

#[test]
fn json_rejects_wrong_lengths_symbols_and_data_model_types() {
    let malformed_strings = [
        "",
        "01K2R7KFWE580700000000000",
        "01K2R7KFWE58070000000000010",
        "81K2R7KFWE5807000000000001",
        "01K2R7KFWE580700000000000U",
        "01K2R7KFWE580700000000000é",
    ];
    for value in malformed_strings {
        let encoded = format!("\"{value}\"");
        assert!(
            serde_json::from_str::<ZLID>(&encoded).is_err(),
            "accepted malformed string {value:?}"
        );
    }

    for encoded in ["null", "7", "{}", "[]", "[0, 1, 2]"] {
        let error = serde_json::from_str::<ZLID>(encoded).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("canonical 26-character ZLID string"),
            "unexpected error for {encoded}: {error}"
        );
    }
}

#[test]
fn postcard_uses_exactly_sixteen_wire_bytes() {
    let bytes = [
        0x00, 0x66, 0x2c, 0x79, 0xbf, 0x8e, 0x2a, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ];
    let zlid = ZLID::from_array(bytes);
    let mut buffer = [0u8; ZLID::BYTE_LENGTH];

    let encoded = postcard::to_slice(&zlid, &mut buffer).unwrap();
    assert_eq!(encoded, &bytes);
    assert_eq!(postcard::from_bytes::<ZLID>(encoded).unwrap(), zlid);
}

#[test]
fn postcard_preserves_nested_tuple_framing() {
    let zlid = ZLID::parse_canonical(CANONICAL).unwrap();
    let value = (0x12u8, zlid, 0x34u8);
    let mut buffer = [0u8; ZLID::BYTE_LENGTH + 2];

    let encoded = postcard::to_slice(&value, &mut buffer).unwrap();
    let mut expected = [0u8; ZLID::BYTE_LENGTH + 2];
    expected[0] = 0x12;
    expected[1..=ZLID::BYTE_LENGTH].copy_from_slice(zlid.as_bytes());
    expected[ZLID::BYTE_LENGTH + 1] = 0x34;
    assert_eq!(encoded, &expected);

    let decoded = postcard::from_bytes::<(u8, ZLID, u8)>(encoded).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn postcard_rejects_short_values_and_exposes_trailing_frame_data() {
    let short = [0u8; ZLID::BYTE_LENGTH - 1];
    assert!(decode_postcard_exact(&short).is_err());

    let long = [0u8; ZLID::BYTE_LENGTH + 1];
    let (_, remainder) = postcard::take_from_bytes::<ZLID>(&long).unwrap();
    assert_eq!(remainder, &[0]);
    assert_eq!(decode_postcard_exact(&long).unwrap_err(), "1 trailing byte");

    let zlid = ZLID::parse_canonical(CANONICAL).unwrap();
    let mut buffer = [0u8; ZLID::BYTE_LENGTH + 1];
    buffer[..ZLID::BYTE_LENGTH].copy_from_slice(zlid.as_bytes());
    buffer[ZLID::BYTE_LENGTH] = 0xff;
    assert_eq!(
        decode_postcard_exact(&buffer).unwrap_err(),
        "1 trailing byte"
    );
}

fn decode_postcard_exact(input: &[u8]) -> Result<ZLID, String> {
    let (zlid, remainder) = postcard::take_from_bytes(input).map_err(|error| error.to_string())?;
    if remainder.is_empty() {
        Ok(zlid)
    } else {
        Err(format!("{} trailing byte", remainder.len()))
    }
}
