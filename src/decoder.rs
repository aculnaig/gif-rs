use std::{io::Read, vec};

use crate::{
    animator::GifStream, error::DecodingError, frame::Frame, lzw::LzwDecoder, reader::SubBlockReader, render::GifColor, structs::{
        Color, DisposalMethod, GraphicControl, ImageDescriptor,
        LogicalScreenDescriptor, Palette,
    }
};

pub struct Decoder<R> {
    reader: R,
    pub screen_descriptor: LogicalScreenDescriptor,
    pub global_palette: Option<Palette>,
}

pub enum Block {
    Image(ImageDescriptor, Option<GraphicControl>),
    Trailer,
    Extension,
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

        let global_palette = if screen_descriptor
            .has_global_color_table()
        {
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

    pub fn next_record(&mut self) -> Result<Block, DecodingError> {
        let mut current_graphic_control = None;

        loop {
            let mut introducer = [0u8; 1];
            if self.reader.read(&mut introducer)? == 0 {
                return Err(DecodingError::Io(
                    std::io::ErrorKind::UnexpectedEof.into(),
                ));
            }

            match introducer[0] {
                // --- Image Separator (0x2C) ---
                0x2C => {
                    let descriptor = self.read_image_descriptor()?;
                    // TODO: 1. read LocalPalette
                    // TODO: 2. read LZW data
                    return Ok(Block::Image(
                        descriptor,
                        current_graphic_control,
                    ));
                }

                // --- Extension Introducer (0x21)
                0x21 => {
                    let mut label = [0u8; 1];
                    self.reader.read_exact(&mut label)?;

                    match label[0] {
                        // Graphic Control Extension (0xF9)
                        0xF9 => {
                            current_graphic_control = Some(
                                self.read_graphic_control_ext()?,
                            );
                        }
                        // Application Extension (0xFF) - e.g. Netscape Loop
                        0xFF => {
                            // FIXME: we ignore it for now
                            self.skip_extension_blocks()?;
                        }
                        // FIXME: Skip Comment (0xFE) or Text (0x01)
                        _ => {
                            self.skip_extension_blocks()?;
                        }
                    }
                }

                // --- Trailer (0x3B) ---
                0x3B => return Ok(Block::Trailer),

                // --- Padding ---
                0x00 => continue,

                _ => {
                    return Err(DecodingError::Format(format!(
                        "Unknown block introducer: {:#04X}",
                        introducer[0]
                    )));
                }
            }
        }
    }

    /// Obtains a reader with the current frame compressed data
    ///
    /// It must be called right after getting `Block::Image` and eventually read the Local Palette.
    ///
    /// The first byte read from this reader will be `LZW Minimum Code Size`
    pub fn lzw_reader(&mut self) -> SubBlockReader<'_, R> {
        SubBlockReader::new(&mut self.reader)
    }

    /// Decodes the entire frame in a linear buffer
    /// The buffer must be big `width * height`
    /// Returns true if the image is completely decoded
    pub fn decode_frame_into(
        &mut self,
        descriptor: &ImageDescriptor,
        output_buffer: &mut [u8],
    ) -> Result<(), DecodingError> {
        let width = descriptor.width as usize;
        let height = descriptor.height as usize;
        let expected_pixels = width * height;

        if output_buffer.len() < expected_pixels {
            return Err(DecodingError::Format(format!(
                "Buffer too small. Requested {} bytes, received {}",
                expected_pixels,
                output_buffer.len(),
            )));
        }

        let mut min_code_size_buf = [0u8; 1];
        self.reader.read_exact(&mut min_code_size_buf)?;
        let min_code_size = min_code_size_buf[0];

        let mut sub_reader = SubBlockReader::new(&mut self.reader);
        let mut lzw = LzwDecoder::new(&mut sub_reader, min_code_size);

        let is_interlaced = descriptor.is_interlaced();

        let (mut pass, mut y, mut checked_pass_advance) =
            (0, 0, false);
        let pass_starts = [0, 4, 2, 1];
        let pass_steps = [8, 8, 4, 2];

        if !is_interlaced {
            let mut offset = 0;
            while offset < expected_pixels {
                let n = lzw.decode_bytes(
                    &mut output_buffer[offset..expected_pixels],
                )?;
                if n == 0 {
                    break;
                }
                offset += n;
            }
        } else {
            let mut lzw_chunk = [0u8; 1024];
            let mut x = 0;

            loop {
                let n = lzw.decode_bytes(&mut lzw_chunk)?;
                if n == 0 {
                    break;
                }

                for &pixel in &lzw_chunk[..n] {
                    if y >= height {
                        break;
                    }

                    let output_idx = y * width + x;
                    if output_idx < output_buffer.len() {
                        output_buffer[output_idx] = pixel;
                    }

                    x += 1;

                    if x == width {
                        x = 0;
                        y += pass_steps[pass];

                        while y >= height && pass < 3 {
                            pass += 1;
                            y = pass_starts[pass];
                        }
                    }
                }
            }
        }

        drop(lzw);
        sub_reader.consume_to_end()?;

        Ok(())
    }

    pub fn next_frame(&mut self) -> Result<Option<Frame>, DecodingError> {
        let (descriptor, control_ext) = match self.next_record()? {
            Block::Image(desc, ext) => (desc, ext),
            Block::Trailer => return Ok(None),
            Block::Extension => return self.next_frame(),
        };

        let active_palette: Palette = if descriptor.has_local_palette() {
            let size = descriptor.local_palette_size();
            Self::read_palette(&mut self.reader, size)?
        } else {
            match &self.global_palette {
                Some(p) => p.clone(),
                None => return Err(DecodingError::Format("No Global or Local palette found".into())),
            }
        };

        let pixel_count = (descriptor.width as usize) * (descriptor.height as usize);

        let mut index_buffer = vec![0u8; pixel_count];

        let mut rgba_buffer = vec![GifColor::transparent(); pixel_count];

        self.decode_frame_into(&descriptor, &mut index_buffer)?;

        GifColor::map_indices_to_rgba(
            &index_buffer,
            &active_palette,
            &control_ext,
            &mut rgba_buffer
        )?;

        let delay_cs = control_ext.map(|x| x.delay_time_cs).unwrap_or(0);
        let disposal = control_ext.map(|x| x.disposal_method).unwrap_or_default();
        let transparent_idx = control_ext.and_then(|x| x.transparent_color_index);

        Ok(Some(Frame {
            delay_cs,
            disposal,
            left: descriptor.left,
            top: descriptor.top,
            width: descriptor.width,
            height: descriptor.height,
            pixels: rgba_buffer,
            transparent_index: transparent_idx,
        }))
    }

    pub fn into_stream(self) -> Result<GifStream<R>, DecodingError> {
        GifStream::new(self)
    }

    fn read_palette(
        reader: &mut R,
        size: usize,
    ) -> Result<Palette, DecodingError> {
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

    fn read_image_descriptor(
        &mut self,
    ) -> Result<ImageDescriptor, DecodingError> {
        let mut buf = [0u8; 9];
        self.reader.read_exact(&mut buf)?;

        Ok(ImageDescriptor {
            left: u16::from_le_bytes([buf[0], buf[1]]),
            top: u16::from_le_bytes([buf[2], buf[3]]),
            width: u16::from_le_bytes([buf[4], buf[5]]),
            height: u16::from_le_bytes([buf[6], buf[7]]),
            packed: buf[8],
        })
    }

    fn read_graphic_control_ext(
        &mut self,
    ) -> Result<GraphicControl, DecodingError> {
        // [Block Size = 4] [Packed] [Delay L] [Delay H] [Trans Index] [Terminator = 0]
        let mut buf = [0u8; 6];
        self.reader.read_exact(&mut buf)?;

        if buf[0] != 4 {
            return Err(DecodingError::Format(
                "Invalid GCE size".into(),
            ));
        }

        let packed = buf[1];
        let disposal = (packed & 0b0001_1100) >> 2;
        let has_transparency = (packed & 1) != 0;

        let delay = u16::from_le_bytes([buf[2], buf[3]]);
        let trans_index =
            if has_transparency { Some(buf[4]) } else { None };

        Ok(GraphicControl {
            disposal_method: DisposalMethod::from(disposal),
            user_input_flag: (packed & 0b0000_0010) != 0,
            delay_time_cs: delay,
            transparent_color_index: trans_index,
        })
    }

    /// GIF metadata are divided in blocks: [Length N] [N Bytes] ... [0 (Terminator)]
    fn skip_extension_blocks(&mut self) -> Result<(), DecodingError> {
        let mut len_buf = [0u8; 1];
        loop {
            self.reader.read_exact(&mut len_buf)?;

            let len = len_buf[0] as usize;
            if len == 0 {
                break; // Terminator found
            }

            // Skip N bytes
            // We can improve by using std::io::skip if the Decoder will support std::io::Seek in a future
            // TODO: use a static buffer
            let mut temp = vec![0u8; len];
            self.reader.read_exact(&mut temp)?;
        }

        Ok(())
    }
}
