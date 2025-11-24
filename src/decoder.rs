use std::io::Read;

use crate::{error::DecodingError, structs::{Color, LogicalScreenDescriptor, Palette}};

pub struct Decoder<R> {
    reader: R,
    pub screen_descriptor: LogicalScreenDescriptor,
    pub global_palette: Option<Palette>,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut reader: R) -> Result<Self, DecodingError> {
        let mut signature = [0u8; 6];
        reader.read_exact(&mut signature)?;

        if &signature != b"GIF89a" && &signature != b"GIF87a" {
            return Err(DecodingError::InvalidSignature);
        }

        let mut lsd_buf = [0u8; 7];
        reader.read_exact(&mut lsd_buf)?;

        let screen_descriptor = LogicalScreenDescriptor {
            width: u16::from_le_bytes([lsd_buf[0], lsd_buf[1]]),
            height: u16::from_le_bytes([lsd_buf[2], lsd_buf[3]]),
            packed_fields: lsd_buf[4],
            bg_color_index: lsd_buf[5],
            pixel_aspect_ration: lsd_buf[6],
        };

        let global_palette = if screen_descriptor.has_global_color_table() {
            let size = screen_descriptor.global_color_table_size();
            Some(Self::read_palette(&mut reader, size)?)
        } else {
            None
        };

        Ok(Self {
            reader,
            screen_descriptor,
            global_palette,
        })
    }

    fn read_palette(reader: &mut R, size: usize) -> Result<Palette, DecodingError> {
        // TOOD: use a buffer pool
        let mut buffer = vec![0u8; 3];
        reader.read_exact(&mut buffer)?;

        let mut palette = Vec::with_capacity(size);
        for chunk in buffer.chunks_exact(3) {
            palette.push(Color {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
            });
        }
        Ok(palette)
    }
}
