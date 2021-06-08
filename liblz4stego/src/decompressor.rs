use xxhash_rust::xxh32::Xxh32;

use crate::constants::{LZ4_MAGIC_NUMBER, MATCH_LENGTH_OFFSET, TOKEN_MAX_VAL};
use crate::descriptors::{BdByte, BlockSize, FlgByte, Token};
use crate::errors::{DecompressResult, Lz4DecompressError};

use crate::numeral_coding;
use crate::occurrence_map::OccurrenceMap;
use byteorder::{ReadBytesExt, LE};
use std::cmp::min;
use std::io::Read;

pub struct Decompressor<R: Read> {
    input_read: R,
    input_buffer: Vec<u8>,
    buffer: Vec<u8>,
    hash: Xxh32,
    hidden_data_decoder: numeral_coding::Encoder,

    header_read: bool,
    content_checksum_added: bool,
    buffer_start: usize,
    prefer_hidden: bool,
}

impl<R: Read> Decompressor<R> {
    pub fn new(input_read: R, prefer_hidden: bool) -> Self {
        const INPUT_BUFFER_SIZE: usize = 4 * 1024 * 1024;

        let mut input_buffer = Vec::with_capacity(INPUT_BUFFER_SIZE);
        unsafe {
            input_buffer.set_len(INPUT_BUFFER_SIZE);
        }

        Self {
            input_read,
            input_buffer,
            buffer: Vec::new(),
            hash: Xxh32::new(0),
            hidden_data_decoder: numeral_coding::Encoder::new(),

            header_read: false,
            content_checksum_added: false,
            buffer_start: 0,
            prefer_hidden,
        }
    }

    fn read_header(&mut self) -> DecompressResult<()> {
        if self.input_read.read_u32::<LE>()? != LZ4_MAGIC_NUMBER {
            return Err(Lz4DecompressError::from_static_str("Invalid header"));
        }

        // TODO check all releveant flags
        let flg = FlgByte(self.input_read.read_u8()?);
        if flg.get_version() != 1 {
            return Err(Lz4DecompressError::from_static_str("Version is not 1"));
        }

        if flg.is_dictionary_id_set() {
            return Err(Lz4DecompressError::from_static_str(
                "Dictionary ID is not supported",
            ));
        }

        if !flg.is_block_independent() {
            return Err(Lz4DecompressError::from_static_str(
                "Blocks must be independent",
            ));
        }

        self.content_checksum_added = flg.is_content_checksum_added();

        let byte = BdByte(self.input_read.read_u8()?);
        let block_max_size = get_block_max_size(byte.get_block_max_size())?;
        self.buffer.reserve(block_max_size);

        // TODO check HC
        self.input_read.read_u8()?;

        self.header_read = true;

        Ok(())
    }

    fn read_block(&mut self) -> DecompressResult<usize> {
        let block_size_val = self.input_read.read_u32::<LE>()?;
        if block_size_val == 0 {
            self.check_checksum()?;
            return Ok(0);
        }

        let block_size_desc = BlockSize(block_size_val);
        let start_index = self.buffer.len();
        let block_size = block_size_desc.get_block_size() as usize;

        self.input_read.read(&mut self.input_buffer[..block_size])?;
        if block_size_desc.is_uncompressed() {
            let new_data = &self.input_buffer[..block_size];
            self.hash.update(new_data);
            self.buffer.extend_from_slice(new_data);

            Ok(block_size)
        } else {
            let (bytes_read, matches) = decompress_block_data(
                &mut self.buffer,
                &mut self.hash,
                &self.input_buffer[..block_size],
            )?;
            self.analyze_matches(start_index, matches);

            Ok(bytes_read)
        }
    }

    fn check_checksum(&mut self) -> DecompressResult<()> {
        if self.content_checksum_added {
            let file_checksum = self.input_read.read_u32::<LE>()?;
            let computed_checksum = self.hash.digest();
            if file_checksum != computed_checksum {
                return Err(Lz4DecompressError::from_static_str("Checksum is invalid"));
            }
        }

        Ok(())
    }

    pub fn finish(self) -> Vec<u8> {
        self.hidden_data_decoder.finish()
    }

    fn analyze_matches(&mut self, start_index: usize, matches: Vec<(u32, u32)>) {
        let data = &self.buffer[start_index..];
        let mut occur = OccurrenceMap::new(data, self.prefer_hidden);

        let mut last_index: u32 = 0;

        for (index, match_index) in matches {
            occur.add_occurrences(last_index as usize, index as usize - last_index as usize);
            last_index = index;

            let occurrences = occur.get_occurrences(index as usize);
            let max_val = occurrences.len();
            let val = occurrences.get_occurrence_index(match_index as usize);

            if let Some(value) = val {
                self.hidden_data_decoder
                    .add_value(value as u16, max_val as u16);
            }
        }
    }
}

fn decompress_block_data(
    buffer: &mut Vec<u8>,
    hash: &mut Xxh32,
    mut data: &[u8],
) -> DecompressResult<(usize, Vec<(u32, u32)>)> {
    let start_len = buffer.len();
    let mut matches: Vec<(u32, u32)> = Vec::new();
    let start_index = buffer.len();

    loop {
        let token = Token(data.read_u8()?);

        let literals_length_initial = token.get_literals_length();
        let literals_length =
            get_lsic_int(&mut data, literals_length_initial, TOKEN_MAX_VAL) as usize;
        buffer.extend_from_slice(&data[..literals_length]);
        data = &data[literals_length as usize..];

        if data.is_empty() {
            // End of block
            hash.update(&buffer[start_len..]);
            return Ok((buffer.len() - start_len, matches));
        }

        let offset = data.read_u16::<LE>()? as usize;
        if offset == 0 {
            return Err(Lz4DecompressError::from_static_str("Offset is 0"));
        }
        let match_length_initial = token.get_match_length();
        let match_length =
            get_lsic_int(&mut data, match_length_initial, TOKEN_MAX_VAL) + MATCH_LENGTH_OFFSET;
        let next_pos = buffer.len();
        let next_pos_block = next_pos - start_index;
        matches.push((next_pos_block as u32, next_pos_block as u32 - offset as u32));

        let match_pos = next_pos - offset;
        if offset == 1 {
            buffer.resize(buffer.len() + match_length as usize, buffer[match_pos]);
        } else {
            let mut to_copy = match_length as usize;

            while to_copy > 0 {
                let current_to_copy = min(to_copy, buffer.len() - match_pos);
                buffer.extend_from_within(match_pos..match_pos + current_to_copy);
                to_copy -= current_to_copy;
            }
        }
    }
}

impl<R: Read> Read for Decompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self.header_read {
            self.read_header()?;
        }

        if self.buffer.is_empty() || self.buffer_start >= self.buffer.len() {
            self.buffer.clear();
            self.buffer_start = 0;
            let bytes_read = self.read_block()?;

            if bytes_read == 0 {
                return Ok(0);
            }
        }

        let to_return = min(buf.len(), self.buffer.len() - self.buffer_start);
        buf[..to_return]
            .copy_from_slice(&self.buffer[self.buffer_start..self.buffer_start + to_return]);
        self.buffer_start += to_return;
        Ok(to_return)
    }
}

fn get_block_max_size(index: u8) -> DecompressResult<usize> {
    if index < 3 || index > 7 {
        return Err(Lz4DecompressError::from_static_str(
            "Block max size is invalid",
        ));
    }

    Ok((index as usize - 4).pow(4) * 65536)
}

fn get_lsic_int(data: &mut &[u8], initial_val: u8, max_val: u8) -> u32 {
    let mut val = initial_val as u32;

    if initial_val < max_val {
        return val;
    }

    loop {
        let current_val = data.read_u8().unwrap();
        val += current_val as u32;

        if current_val < 255 {
            return val;
        }
    }
}
