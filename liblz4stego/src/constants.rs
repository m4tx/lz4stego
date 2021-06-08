pub const LZ4_MAGIC_NUMBER: u32 = 0x184D2204;

pub const MIN_COMPRESS_LENGTH: usize = 13;
pub const END_LITERAL_NUM: usize = 5;
pub const MAP_PREF_SIZE: usize = 4;
pub const MAX_BLOCK_SIZE: usize = 4 * 1024 * 1024 - 12;
pub const MAX_OFFSET: usize = 65535;
pub const TOKEN_MAX_VAL: u8 = 15;
pub const MATCH_LENGTH_OFFSET: u32 = 4;
