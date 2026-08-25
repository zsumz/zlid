#![no_main]

use libfuzzer_sys::fuzz_target;
use zlid::ZLID;

fuzz_target!(|input: &str| {
    if let Ok(value) = ZLID::parse(input) {
        let canonical = value.text();
        assert_eq!(canonical.len(), ZLID::TEXT_LENGTH);
        assert_eq!(ZLID::parse_canonical(&canonical), Ok(value));
        assert_eq!(ZLID::parse(&canonical), Ok(value));
        assert_eq!(value.to_string(), canonical);
    }
});
