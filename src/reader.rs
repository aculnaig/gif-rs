use std::io::Read;

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
        todo!()
    }
}
