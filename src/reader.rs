use std::io::{self, Read};

pub struct SubBlockReader<'a, R> {
    reader: &'a mut R,
    remaining_in_block: usize,
    finished: bool,
}

impl<'a, R: Read> SubBlockReader<'a, R> {
    /// Creates a new SubBlockReader
    /// It assumes the reader be positioned exactly at the first byte
    /// that indicates the dimension of the first block (or 0 if empty)
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            remaining_in_block: 0,
            finished: false,
        }
    }
}

impl<'a, R: Read> Read for SubBlockReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // If we find the terminator (0x00), we are finished
        if self.finished {
            return Ok(0);
        }

        // If the current block is drained, we need to read the header of the next block.
        if self.remaining_in_block == 0 {
            let mut size_byte = [0u8; 1];
            // Try to read the dimension of the next block
            match self.reader.read(&mut size_byte) {
                Ok(0) => {
                    // Unexpected EOF from the underlying reader meanwhile we were waiting for a block.
                    // Given the GIF specification, there must be an explicit terminator 0x00.
                    // We can handle it as end of stream or error.
                    // Here we are tolerant.
                    self.finished = true;
                    return Ok(0);
                }
                Ok(_) => {},
                Err(e) => return Err(e),
            }

            let block_size = size_byte[0] as usize;

            if block_size == 0 {
                self.finished = true;
                return Ok(0);
            }

            self.remaining_in_block = block_size;
        }

        let max_read = std::cmp::min(buf.len(), self.remaining_in_block);

        let read_amount = self.reader.read(&mut buf[..max_read])?;

        // File is corrupted
        if read_amount == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "GIF stream truncated inside a data sub-block"
            ));
        }

        // Update the states
        self.remaining_in_block -= read_amount;

        Ok(read_amount)
    }
}
