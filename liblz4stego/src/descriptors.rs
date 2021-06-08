use bitfield::bitfield;

bitfield! {
    pub struct FlgByte(u8);
    impl Debug;

    pub get_version, set_version: 7, 6;
    pub is_block_independent, set_block_independent: 5;
    pub is_block_checksum_added, set_block_checksum_added: 4;
    pub is_content_size_added, set_content_size_added: 3;
    pub is_content_checksum_added, set_content_checksum_added: 2;
    pub is_dictionary_id_set, set_dictionary_id_set: 0;
}

bitfield! {
    pub struct BdByte(u8);
    impl Debug;

    pub get_block_max_size, set_block_max_size: 6, 4;
}

bitfield! {
    pub struct BlockSize(u32);
    impl Debug;

    pub get_block_size, set_block_size: 30, 0;
    pub is_uncompressed, set_block_uncompressed: 31;
}

bitfield! {
    pub struct Token(u8);
    impl Debug;

    pub get_literals_length, set_literals_length: 7, 4;
    pub get_match_length, set_match_length: 3, 0;
}
