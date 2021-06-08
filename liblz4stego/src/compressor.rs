use std::cmp::min;

use log::debug;
use xxhash_rust::xxh32::{xxh32, Xxh32};

use crate::constants::{
    END_LITERAL_NUM, LZ4_MAGIC_NUMBER, MATCH_LENGTH_OFFSET, MAX_BLOCK_SIZE, MIN_COMPRESS_LENGTH,
    TOKEN_MAX_VAL,
};
use crate::descriptors::{BdByte, BlockSize, FlgByte, Token};
use crate::numeral_coding;
use crate::occurrence_map::OccurrenceMap;
use byteorder::{WriteBytesExt, LE};
use std::collections::VecDeque;
use std::io::Write;

pub struct Compressor<'a, W: Write> {
    output_write: W,
    buffer: VecDeque<u8>,
    hash: Xxh32,
    hidden_data_encoder: numeral_coding::Decoder<'a>,
    prefer_hidden: bool,
}

impl<'a, W: Write> Compressor<'a, W> {
    pub fn new_with_hidden_data(
        writer: W,
        hidden_data: &'a [u8],
        prefer_hidden: bool,
    ) -> Result<Self, std::io::Error> {
        let mut compressor = Self {
            output_write: writer,
            buffer: VecDeque::new(),
            hash: Xxh32::new(0),
            hidden_data_encoder: numeral_coding::Decoder::new(hidden_data),
            prefer_hidden,
        };

        compressor.init()?;

        Ok(compressor)
    }

    pub fn new(writer: W) -> Result<Self, std::io::Error> {
        let mut compressor = Self {
            output_write: writer,
            buffer: VecDeque::new(),
            hash: Xxh32::new(0),
            hidden_data_encoder: numeral_coding::Decoder::new(b""),
            prefer_hidden: false,
        };

        compressor.init()?;

        Ok(compressor)
    }

    fn init(&mut self) -> Result<(), std::io::Error> {
        self.write_header()?;

        Ok(())
    }

    fn get_available_bytes(&self) -> usize {
        self.hidden_data_encoder.get_available_bytes()
    }

    pub fn finish(mut self) -> Result<usize, std::io::Error> {
        if !self.buffer.is_empty() {
            self.output_block(true)?;
        }

        self.write_footer()?;
        self.output_write.flush()?;

        Ok(self.get_available_bytes())
    }

    fn write_header(&mut self) -> Result<(), std::io::Error> {
        self.output_write.write_u32::<LE>(LZ4_MAGIC_NUMBER)?;

        let frame_descriptor = self.build_frame_descriptor();
        self.output_write.write(&frame_descriptor)?;

        Ok(())
    }

    fn build_frame_descriptor(&self) -> Vec<u8> {
        let mut output = Vec::new();

        let mut flag = FlgByte(0);
        flag.set_version(1);
        flag.set_block_independent(true);
        flag.set_content_checksum_added(true);
        output.write_u8(flag.0).unwrap();

        let mut bd = BdByte(0);
        bd.set_block_max_size(7);
        output.write_u8(bd.0).unwrap();

        let hc = ((xxh32(&output, 0) >> 8) & 0xFF) as u8;
        output.write_u8(hc).unwrap();

        output
    }

    fn write_footer(&mut self) -> Result<(), std::io::Error> {
        self.output_write.write_u32::<LE>(0)?;
        self.output_write.write_u32::<LE>(self.hash.digest())?;

        Ok(())
    }

    fn output_block(&mut self, force_write: bool) -> Result<(), std::io::Error> {
        let mut data = self.buffer.make_contiguous();
        let mut to_shrink = 0;

        while !data.is_empty() && (data.len() >= MAX_BLOCK_SIZE || force_write) {
            if data.len() < MIN_COMPRESS_LENGTH {
                output_uncompressed_block(&mut self.output_write, data)?;
                to_shrink += data.len();
                break;
            }

            let block_size = min(data.len(), MAX_BLOCK_SIZE);
            output_compressed_block(
                &mut self.output_write,
                &data[..block_size],
                &mut self.hidden_data_encoder,
                self.prefer_hidden,
            )?;

            to_shrink += block_size;
            data = &mut data[block_size..];
        }

        self.buffer.drain(..to_shrink);

        Ok(())
    }
}

impl<'a, W: Write> Write for Compressor<'a, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.buffer.extend(buf);
        self.hash.update(buf);

        if self.buffer.len() >= MAX_BLOCK_SIZE {
            self.output_block(false)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.buffer.is_empty() {
            self.output_block(true)?;
            self.output_write.flush()?;
        }

        Ok(())
    }
}

fn output_uncompressed_block<W: Write>(
    mut output_write: W,
    data: &[u8],
) -> Result<(), std::io::Error> {
    debug!("Outputting uncompressed block with length: {}", data.len());

    let mut block_size = BlockSize(0);
    block_size.set_block_uncompressed(true);
    block_size.set_block_size(data.len() as u32);

    output_write.write_u32::<LE>(block_size.0)?;
    output_write.write(data)?;

    Ok(())
}

fn output_compressed_block<W: Write>(
    mut output_write: W,
    data: &[u8],
    hidden_data_encoder: &mut numeral_coding::Decoder,
    prefer_hidden: bool,
) -> Result<(), std::io::Error> {
    let mut output = Vec::new();

    // Reserve space for BlockSize
    output.write_u32::<LE>(0).unwrap();

    let mut occur = OccurrenceMap::new(data, prefer_hidden);
    let mut literals = Vec::new();
    let mut i = 0;

    while i < data.len() - END_LITERAL_NUM {
        let occurrences = occur.get_occurrences(i);

        if occurrences.len() > 0 {
            let chosen_index = hidden_data_encoder.decode_value(occurrences.len() as u16);
            let (index, match_length) = occurrences.choose_occurrence(chosen_index as usize);
            if match_length < 4 {
                // End of block
                literals.push(data[i]);
                i += 1;
                continue;
            }

            let offset = (i - index) as u16;

            output_sequence(&literals, offset, match_length as u32, &mut output);

            literals.clear();
            occur.add_occurrences(i, match_length);
            i += match_length;
        } else {
            literals.push(data[i]);
            occur.add_occurrences(i, 1);
            i += 1;
        }
    }

    literals.extend_from_slice(&data[data.len() - END_LITERAL_NUM..]);
    output_sequence(&literals, 0, MATCH_LENGTH_OFFSET, &mut output);

    let mut block_size = BlockSize(0);
    block_size.set_block_uncompressed(false);
    let block_size_num = output.len() - 4;
    block_size.set_block_size(block_size_num as u32);
    output.splice(0..4, block_size.0.to_le_bytes());

    debug!("Block size: {}, data size: {}", block_size_num, data.len());

    if block_size_num <= MAX_BLOCK_SIZE {
        output_write.write(&output)?;
    } else {
        output_uncompressed_block(output_write, data)?;
    }

    Ok(())
}

fn output_sequence(literals: &Vec<u8>, offset: u16, match_length: u32, output: &mut Vec<u8>) {
    debug!(
        "Outputting sequence: literals {:?}, offset={}, match_length={}",
        literals, offset, match_length
    );

    let literals_len = literals.len() as u32;
    let match_length_saved = match_length - MATCH_LENGTH_OFFSET;

    let mut token = Token(0);
    token.set_literals_length(min(literals_len, TOKEN_MAX_VAL as u32) as u8);
    token.set_match_length(min(match_length_saved, TOKEN_MAX_VAL as u32) as u8);
    output.write_u8(token.0).unwrap();

    output_lsic_int(literals_len, TOKEN_MAX_VAL, output);
    output.extend_from_slice(literals.as_slice());

    if offset != 0 {
        // Last sequence
        output.write_u16::<LE>(offset).unwrap();
        output_lsic_int(match_length_saved, TOKEN_MAX_VAL, output);
    }
}

fn output_lsic_int(val: u32, max_val: u8, output: &mut Vec<u8>) {
    if val < max_val as u32 {
        return;
    }

    let mut new_val = val - max_val as u32;

    while new_val > 255 {
        output.write_u8(255).unwrap();
        new_val -= 255;
    }
    output.write_u8(new_val as u8).unwrap();
}
