#![no_main]

use libfuzzer_sys::fuzz_target;
use zlid::ZLID;

fuzz_target!(|bytes: [u8; 16]| {
    let value = ZLID::from_array(bytes);
    let text = value.text();
    let inspection = value.inspect();

    assert_eq!(value.bytes(), bytes);
    assert_eq!(ZLID::from_bytes(&bytes), Ok(value));
    assert_eq!(ZLID::parse_canonical(&text), Ok(value));
    assert_eq!(inspection.text(), text);
    assert_eq!(inspection.bytes_hex(), value.bytes_hex());
    assert_eq!(inspection.tag(), value.tag());
    assert_eq!(inspection.kind(), value.kind());
    assert_eq!(inspection.family(), value.family());
});
