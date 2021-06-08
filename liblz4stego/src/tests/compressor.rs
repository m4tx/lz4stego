use lz4::Decoder;
use std::io::{Read, Write};

use crate::compressor::Compressor;

fn decode_lz4(data: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut decoder = Decoder::new(data.as_slice()).unwrap();
    decoder.read_to_end(&mut buffer).unwrap();

    buffer
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut compressor = Compressor::new(&mut output).unwrap();
    compressor.write(data).unwrap();
    compressor.finish().unwrap();

    output
}

#[test]
fn compress_empty() {
    let data = b"";
    let result = compress(data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_single_byte() {
    let data = b"a";
    let result = compress(data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_short_compressed() {
    let data = b"aaaaaaaaaaaaaaaaaaaa";
    let result = compress(data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_medium() {
    let data = include_bytes!("test_data/medium");
    let result = compress(data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_large_two_parts() {
    let data = include_bytes!("test_data/large_two_parts");
    let result = compress(data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_large_two_blocks() {
    let data = b"a".repeat(8_388_608);
    let result = compress(&data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}

#[test]
fn compress_long_match() {
    let mut data = b"a".repeat(100_000);
    data.extend_from_slice(&b"b".repeat(100_000));
    data.extend_from_slice(&b"a".repeat(100));
    let result = compress(&data);
    let decoded = decode_lz4(&result);

    assert_eq!(data, decoded.as_slice());
}
