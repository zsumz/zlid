use std::collections::{BTreeMap, BTreeSet};

use crate::json::{array, boolean, get, number, object as json_object, string, Json};

pub(crate) type Object = BTreeMap<String, Json>;

pub(crate) const MAX_TIMESTAMP: i64 = 281_474_976_710_655;

pub(crate) fn entries<'a>(root: &'a Object, key: &str) -> &'a [Json] {
    let values = array(get(root, key));
    assert!(
        !values.is_empty(),
        "fixture section {key} must not be empty"
    );
    values
}

pub(crate) fn exact_keys(object: &Object, required: &[&str], optional: &[&str]) {
    for key in required {
        assert!(
            object.contains_key(*key),
            "fixture object is missing key {key}"
        );
    }
    let allowed: BTreeSet<_> = required.iter().chain(optional).copied().collect();
    let actual: BTreeSet<_> = object.keys().map(String::as_str).collect();
    assert!(
        actual.is_subset(&allowed),
        "fixture object has unexpected keys: {:?}",
        actual.difference(&allowed).collect::<Vec<_>>()
    );
}

pub(crate) fn object_field<'a>(object: &'a Object, key: &str) -> &'a Object {
    json_object(get(object, key))
}

pub(crate) fn string_field<'a>(object: &'a Object, key: &str) -> &'a str {
    string(get(object, key))
}

pub(crate) fn optional_string<'a>(object: &'a Object, key: &str) -> Option<&'a str> {
    object.get(key).map(string)
}

pub(crate) fn nonempty_string(object: &Object, key: &str) {
    assert!(
        !string_field(object, key).is_empty(),
        "fixture field {key} must not be empty"
    );
}

pub(crate) fn exact_string(object: &Object, key: &str, expected: &str) {
    assert_eq!(
        expected,
        string_field(object, key),
        "fixture field {key} drifted"
    );
}

pub(crate) fn enum_string(object: &Object, key: &str, expected: &[&str]) {
    let value = string_field(object, key);
    assert!(
        expected.contains(&value),
        "invalid fixture {key} value {value:?}"
    );
}

pub(crate) fn number_range(object: &Object, key: &str, min: i64, max: i64) {
    let value = number(get(object, key));
    assert!(
        (min..=max).contains(&value),
        "fixture field {key}={value} is outside {min}..={max}"
    );
}

pub(crate) fn optional_number_range(object: &Object, key: &str, min: i64, max: i64) {
    if let Some(value) = object.get(key) {
        let value = number(value);
        assert!(
            (min..=max).contains(&value),
            "fixture field {key}={value} is outside {min}..={max}"
        );
    }
}

pub(crate) fn const_true(object: &Object, key: &str) {
    assert!(
        boolean(get(object, key)),
        "fixture field {key} must be true"
    );
}

pub(crate) fn case_id(object: &Object) {
    let value = string_field(object, "id");
    let split = value
        .find(|ch: char| ch.is_ascii_digit())
        .unwrap_or(value.len());
    let (letters, digits) = value.split_at(split);
    assert!(
        !letters.is_empty()
            && letters.bytes().all(|byte| byte.is_ascii_uppercase())
            && !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit()),
        "invalid fixture case id {value:?}"
    );
}

pub(crate) fn hex_field(object: &Object, key: &str, length: Option<usize>) {
    hex_value(string_field(object, key), key, length);
}

pub(crate) fn optional_hex(object: &Object, key: &str, length: Option<usize>) {
    if let Some(value) = optional_string(object, key) {
        hex_value(value, key, length);
    }
}

pub(crate) fn hex_value(value: &str, key: &str, length: Option<usize>) {
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte)),
        "fixture field {key} is not uppercase hexadecimal"
    );
    if let Some(length) = length {
        assert_eq!(length, value.len(), "fixture field {key} has wrong length");
    }
}

pub(crate) fn canonical_text(object: &Object, key: &str) {
    let value = string_field(object, key);
    assert_eq!(
        26,
        value.len(),
        "fixture field {key} must be 26 ASCII bytes"
    );
    let mut bytes = value.bytes();
    assert!(
        matches!(bytes.next(), Some(b'0'..=b'7')),
        "fixture field {key} has an invalid first character"
    );
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    assert!(
        bytes.all(|byte| ALPHABET.contains(&byte)),
        "fixture field {key} is not canonical Crockford Base32"
    );
}

pub(crate) fn profile(object: &Object) {
    enum_string(object, "profile", &["default", "high-throughput"]);
}

pub(crate) fn clock_state(object: &Object, key: &str) {
    enum_string(object, key, &["normal", "clamped"]);
}

pub(crate) fn number_array(object: &Object, key: &str, min: i64, max: i64) {
    let values = array(get(object, key));
    assert!(!values.is_empty(), "fixture array {key} must not be empty");
    for value in values {
        let value = number(value);
        assert!(
            (min..=max).contains(&value),
            "fixture array {key} is out of range"
        );
    }
}

pub(crate) fn object_array<'a>(object: &'a Object, key: &str) -> &'a [Json] {
    let values = array(get(object, key));
    assert!(!values.is_empty(), "fixture array {key} must not be empty");
    for value in values {
        json_object(value);
    }
    values
}
