use crate::decompressor::Decompressor;
use crate::errors::DecompressResult;
use std::io::Read;

fn decompress(data: &[u8]) -> DecompressResult<(Vec<u8>, Vec<u8>)> {
    let mut output = Vec::new();
    let mut decompressor = Decompressor::new(data, false);
    decompressor.read_to_end(&mut output).unwrap();

    Ok((output, decompressor.finish()))
}

#[test]
fn decompress_empty() {
    let bytes = include_bytes!("test_data/empty.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"");
}

#[test]
fn decompress_single_byte() {
    let bytes = include_bytes!("test_data/single_byte.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"a");
}

#[test]
fn decompress_short_uncompressed() {
    let bytes = include_bytes!("test_data/short_uncompressed.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"aaa");
}

#[test]
fn decompress_short_compressed() {
    let bytes = include_bytes!("test_data/short_compressed.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"aaaaaaaaaaaaaaaaaaaa");
}

#[test]
fn decompress_medium() {
    let bytes = include_bytes!("test_data/medium.lz4");
    let expected = include_bytes!("test_data/medium");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, expected);
}

#[test]
fn decompress_large_single_character() {
    let bytes = include_bytes!("test_data/large_single_character.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"a".repeat(1_000_000));
}

#[test]
fn decompress_large_two_parts() {
    let bytes = include_bytes!("test_data/large_two_parts.lz4");
    let expected = include_bytes!("test_data/large_two_parts");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, expected);
}

#[test]
fn decompress_large_two_blocks() {
    let bytes = include_bytes!("test_data/large_two_blocks.lz4");
    let result = decompress(bytes);

    assert!(result.is_ok(), "{}", result.unwrap_err());
    assert_eq!(result.unwrap().0, b"a".repeat(8_388_608));
}
