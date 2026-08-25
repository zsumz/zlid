use super::HmacSha256;
use crate::bytes_to_hex;

fn assert_vector(key: &[u8], parts: &[&[u8]], expected: &str) {
    assert_eq!(bytes_to_hex(&HmacSha256::new(key).digest(parts)), expected);
}

// RFC 4231, sections 4.2 through 4.8.
// https://www.rfc-editor.org/rfc/rfc4231.html#section-4.2

#[test]
fn rfc_4231_case_1() {
    let key = [0x0b; 20];
    assert_vector(
        &key,
        &[b"Hi ", b"There"],
        "B0344C61D8DB38535CA8AFCEAF0BF12B881DC200C9833DA726E9376C2E32CFF7",
    );
}

#[test]
fn rfc_4231_case_2() {
    assert_vector(
        b"Jefe",
        &[b"what do ya want ", b"for nothing?"],
        "5BDCC146BF60754E6A042426089575C75A003F089D2739839DEC58B964EC3843",
    );
}

#[test]
fn rfc_4231_case_3() {
    let key = [0xaa; 20];
    let data = [0xdd; 50];
    assert_vector(
        &key,
        &[&data],
        "773EA91E36800E46854DB8EBD09181A72959098B3EF8C122D9635514CED565FE",
    );
}

#[test]
fn rfc_4231_case_4() {
    let key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
    ];
    let data = [0xcd; 50];
    assert_vector(
        &key,
        &[&data],
        "82558A389A443C0EA4CC819899F2083A85F0FAA3E578F8077A2E3FF46729665B",
    );
}

#[test]
fn rfc_4231_case_5_truncated_to_128_bits() {
    let key = [0x0c; 20];
    let digest = HmacSha256::new(&key).digest(&[b"Test With ", b"Truncation"]);
    assert_eq!(
        bytes_to_hex(&digest[..16]),
        "A3B6167473100EE06E0C796C2955552B"
    );
}

#[test]
fn rfc_4231_case_6_hashes_a_key_larger_than_one_sha256_block() {
    let key = [0xaa; 131];
    assert_vector(
        &key,
        &[
            b"Test Using Larger Than ",
            b"Block-Size Key - ",
            b"Hash Key First",
        ],
        "60E431591EE0B67F0D8A26AACBF5B77F8E0BC6213728C5140546040F0EE37F54",
    );
}

#[test]
fn rfc_4231_case_7_hashes_a_long_key_and_multipart_message() {
    let key = [0xaa; 131];
    assert_vector(
        &key,
        &[
            b"This is a test using a larger than block-size key and a larger ",
            b"than block-size data. The key needs to be hashed before being ",
            b"used by the HMAC algorithm.",
        ],
        "9B09FFA71B942FCB27635FBCD5B0E944BFDC63644F0713938A7F51535C3A35E2",
    );
}

#[test]
fn prepared_key_state_is_reusable() {
    let hmac = HmacSha256::new(b"reused high-entropy key material");
    let parts: &[&[u8]] = &[b"domain", b"|", b"message"];
    assert_eq!(hmac.digest(parts), hmac.digest(parts));
    assert_ne!(hmac.digest(parts), hmac.digest(&[b"domain|other"]));
}
