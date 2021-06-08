use std::fs;
use std::io::{Read, Write};

pub fn compress(
    input_path: &str,
    output_path: &str,
    hidden_path_opt: Option<&str>,
    count: bool,
    prefer_hidden: bool,
) {
    let mut input_file = fs::File::open(input_path).unwrap();
    let output_file = fs::File::create(output_path).unwrap();
    let hidden_data = if let Some(hidden_path) = hidden_path_opt {
        fs::read(hidden_path).unwrap()
    } else {
        vec![]
    };
    let mut compressor = liblz4stego::compressor::Compressor::new_with_hidden_data(
        output_file,
        &hidden_data,
        prefer_hidden,
    )
    .unwrap();

    const BUFFER_SIZE: usize = 4 * 1024 * 1024;
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    unsafe {
        buffer.set_len(BUFFER_SIZE);
    }
    loop {
        let bytes_read = input_file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }

        compressor.write(&buffer[..bytes_read]).unwrap();
    }

    let available_bytes = compressor.finish().unwrap();
    if count {
        eprintln!("Available hidden data bytes: {}", available_bytes);
    }
}

pub fn decompress(
    input_path: &str,
    output_path: &str,
    hidden_path_opt: Option<&str>,
    prefer_hidden: bool,
) {
    let input_file = fs::File::open(input_path).unwrap();
    let mut output_file = fs::File::create(output_path).unwrap();
    let mut decompressor = liblz4stego::decompressor::Decompressor::new(input_file, prefer_hidden);

    const BUFFER_SIZE: usize = 4 * 1024 * 1024;
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    unsafe {
        buffer.set_len(BUFFER_SIZE);
    }
    loop {
        let bytes_read = decompressor.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }

        output_file.write(&buffer[..bytes_read]).unwrap();
    }

    let hidden_data = decompressor.finish();
    if let Some(hidden_path) = hidden_path_opt {
        fs::write(hidden_path, hidden_data).unwrap();
    }
}
