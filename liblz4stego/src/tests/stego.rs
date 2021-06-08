use std::io::{Read, Write};

use lz4::Decoder;

use crate::compressor::Compressor;
use crate::decompressor::Decompressor;
use crate::errors::DecompressResult;

fn decompress(data: &[u8]) -> DecompressResult<(Vec<u8>, Vec<u8>)> {
    let mut output = Vec::new();
    let mut decompressor = Decompressor::new(data, false);
    decompressor.read_to_end(&mut output).unwrap();

    Ok((output, decompressor.finish()))
}

fn decode_lz4(data: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut decoder = Decoder::new(data.as_slice()).unwrap();
    decoder.read_to_end(&mut buffer).unwrap();

    buffer
}

#[test]
fn test_single_byte() {
    let data = b"ala a ala b ala c ala d ala e ala f ala g ala h ala i ala j ala k ala l ala";
    let hidden_data = b"ab";
    let mut result = Vec::new();
    let mut compressor = Compressor::new_with_hidden_data(&mut result, hidden_data, true).unwrap();
    compressor.write(data).unwrap();
    compressor.finish().unwrap();

    let decoded_expected = decode_lz4(&result);
    let decoded_actual = decompress(&result);

    assert!(decoded_actual.is_ok(), "{}", decoded_actual.unwrap_err());
    let result = decoded_actual.unwrap();
    assert_eq!(result.0, decoded_expected);
    assert_eq!(result.1, hidden_data);
}
