use std::io::{self, Read};

pub struct BitReader<R> {
    input: R,
    bit_buffer: u64,
    bits_in_buffer: u8,
}

impl<R: Read> BitReader<R> {
    pub fn new(input: R) -> Self {
        Self {
            input,
            bit_buffer: 0,
            bits_in_buffer: 0,
        }
    }

    pub fn read_bits(&mut self, n: u8) -> io::Result<u16> {
        if n > 16 {
            panic!("Cannot read more than 16 bits at time");
        }

        while self.bits_in_buffer < n {
            let mut byte = [0u8; 1];
            let bytes_read = self.input.read(&mut byte)?;

            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "End of stream while reading LZW bits",
                ));
            }

            self.bit_buffer |=
                (byte[0] as u64) << self.bits_in_buffer;
            self.bits_in_buffer += 8;
        }

        let result = (self.bit_buffer & ((1 << n) - 1)) as u16;

        self.bit_buffer >>= n;
        self.bits_in_buffer -= n;

        Ok(result)
    }
}
