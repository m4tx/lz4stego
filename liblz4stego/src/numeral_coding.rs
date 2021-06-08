use byteorder::ReadBytesExt;

pub struct Encoder {
    values: Vec<(u16, u16)>,
    encoded: Vec<u8>,
    x: u32,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            encoded: Vec::new(),
            x: 0,
        }
    }

    pub fn add_value(&mut self, value: u16, max_value: u16) {
        assert!(value < max_value);
        self.values.push((value, max_value));
    }

    pub fn finish(mut self) -> Vec<u8> {
        for (val, max_val) in self.values.iter().rev() {
            self.x = self.x * (*max_val as u32) + (*val as u32);

            while self.x >= (1 << 16) {
                self.encoded.push((self.x & 0xFF) as u8);
                self.x >>= 8;
            }
        }

        while self.x > 0 {
            self.encoded.push((self.x & 0xFF) as u8);
            self.x >>= 8;
        }

        self.encoded.reverse();

        self.encoded
    }
}

pub struct Decoder<'a> {
    data: &'a [u8],
    x: u32,
    available_bits: f64,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            x: 0,
            available_bits: 0.0,
        }
    }

    pub fn decode_value(&mut self, max_value: u16) -> u16 {
        self.available_bits += (max_value as f64).log2();
        let max_val = max_value as u32;

        while self.x < (max_val << 8) && !self.data.is_empty() {
            self.x <<= 8;
            self.x += self.data.read_u8().unwrap() as u32;
        }

        let result = (self.x % max_val) as u16;
        self.x = self.x / max_val;

        result
    }

    pub fn get_available_bytes(&self) -> usize {
        (self.available_bits / 8.0) as usize
    }
}
