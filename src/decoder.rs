use std::io::Read;

use crate::{error::DecodingError, structs::{LogicalScreenDescriptor, Palette}};

pub struct Decoder<R> {
    reader: R,
    pub screen_descriptor: LogicalScreenDescriptor,
    pub global_palette: Option<Palette>,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut reader: R) -> Result<Self, DecodingError> {
        todo!()
    }

    fn read_palette(reader: &mut R, size: usize) -> Result<Palette, DecodingError> {
        todo!()
    }
}
