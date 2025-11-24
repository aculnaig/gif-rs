use std::io::{self, Read};

use crate::{bitreader::BitReader, reader::SubBlockReader};

const MAX_CODES: usize = 4096;
const INVALID_CODE: u16 = 0xFFFF;

pub struct LzwDecoder<R> {
    reader: BitReader<R>,

    // Configuration
    min_code_size: u8,
    clear_code: u16,
    end_code: u16,

    // Current state
    code_size: u8,
    next_available_code: u16,
    old_code: u16,
    first_pixel_of_sequence: u8,

    prefix: [u16; MAX_CODES],
    suffix: [u8; MAX_CODES],

    pixel_stack: [u8; MAX_CODES],
    stack_top: usize,
}

impl<R: Read> LzwDecoder<R> {
    pub fn new(reader: R, min_code_size: u8) -> Self {
        let clear_code = 1 << min_code_size;
        let end_code = clear_code + 1;

        let mut decoder = Self {
            reader: BitReader::new(reader),
            min_code_size,
            clear_code,
            end_code,
            code_size: min_code_size + 1,
            next_available_code: end_code + 1,
            old_code: INVALID_CODE,
            first_pixel_of_sequence: 0,
            prefix: [0; MAX_CODES],
            suffix: [0; MAX_CODES],
            pixel_stack: [0; MAX_CODES],
            stack_top: 0,
        };

        decoder.reset_dictionary();
        decoder
    }

    fn reset_dictionary(&mut self) {
        self.code_size = self.min_code_size + 1;
        self.next_available_code = self.end_code + 1;
        self.old_code = INVALID_CODE;

        for i in 0..self.clear_code {
            self.prefix[i as usize] = INVALID_CODE; // Radice
            self.suffix[i as usize] = i as u8;
        }
    }

    pub fn decode_bytes(
        &mut self,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        let mut bytes_written = 0;

        while bytes_written < buf.len() {
            if self.stack_top > 0 {
                let count = std::cmp::min(
                    self.stack_top,
                    buf.len() - bytes_written,
                );
                for i in 0..count {
                    self.stack_top -= 1;
                    buf[bytes_written] =
                        self.pixel_stack[self.stack_top];
                    bytes_written += 1;
                }
                if bytes_written == buf.len() {
                    return Ok(bytes_written);
                }
            }

            let code = match self.reader.read_bits(self.code_size) {
                Ok(c) => c,
                Err(_) => break,
            };

            if code == self.clear_code {
                self.reset_dictionary();
                continue;
            } else if code == self.end_code {
                return Ok(bytes_written);
            }

            let mut current_code = code;

            if code >= self.next_available_code {
                if code == self.next_available_code
                    && self.old_code != INVALID_CODE
                {
                    self.pixel_stack[self.stack_top] =
                        self.first_pixel_of_sequence;
                    self.stack_top += 1;
                    current_code = self.old_code;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid LZW code",
                    ));
                }
            }

            while current_code >= self.clear_code {
                if self.stack_top >= MAX_CODES {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "LZW stack overflow",
                    ));
                }

                self.pixel_stack[self.stack_top] =
                    self.suffix[current_code as usize];
                self.stack_top += 1;
                current_code = self.prefix[current_code as usize];
            }

            self.first_pixel_of_sequence =
                self.suffix[current_code as usize];
            self.pixel_stack[self.stack_top] =
                self.first_pixel_of_sequence;
            self.stack_top += 1;

            if self.old_code != INVALID_CODE
                && self.next_available_code < MAX_CODES as u16
            {
                self.prefix[self.next_available_code as usize] =
                    self.old_code;
                self.suffix[self.next_available_code as usize] =
                    self.first_pixel_of_sequence;
                self.next_available_code += 1;

                if self.next_available_code >= (1 << self.code_size)
                    && self.code_size < 12
                {
                    self.code_size += 1;
                }
            }

            self.old_code = code;
        }

        Ok(bytes_written)
    }
}
