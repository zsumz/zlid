pub(crate) const BYTE_LENGTH: usize = 16;
pub(crate) const STRING_LENGTH: usize = 26;
pub(crate) const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
pub(crate) const MAX_TS: u64 = (1u64 << 48) - 1;
pub(crate) const MASK_62: u64 = (1u64 << 62) - 1;
pub(crate) const MASK_124: u128 = u128::MAX >> 4;
pub(crate) const ZERO_PARTITION_KEY: [u8; 16] = [0; 16];

pub(crate) const TAG_ZLID_DEFAULT_NORMAL: u8 = 0x1;
pub(crate) const TAG_ZLID_HIGH_THROUGHPUT_NORMAL: u8 = 0x2;
pub(crate) const TAG_ZLID_DEFAULT_CLAMPED: u8 = 0x3;
pub(crate) const TAG_ZLID_HIGH_THROUGHPUT_CLAMPED: u8 = 0x4;
pub(crate) const TAG_ZLID_RANDOM: u8 = 0x5;
pub(crate) const TAG_ZLID_ALIAS_DEFAULT_NORMAL: u8 = 0x6;
pub(crate) const TAG_ZLID_ALIAS_DEFAULT_CLAMPED: u8 = 0x7;
pub(crate) const TAG_ZLID_ALIAS_HIGH_THROUGHPUT_NORMAL: u8 = 0x8;
pub(crate) const TAG_ZLID_ALIAS_HIGH_THROUGHPUT_CLAMPED: u8 = 0x9;
