use std::io::Read;

use crate::error::DecodingError;
use crate::structs::DisposalMethod;
use crate::{decoder::Decoder};

use crate::render::GifColor as Color;

pub struct GifStream<R> {
    decoder: Decoder<R>,

    canvas: Vec<Color>,
    last_canvas: Vec<Color>,

    last_disposal: DisposalMethod,
    last_rect: (u16, u16, u16, u16),

    bg_color: Color,
}

impl<R: Read> GifStream<R> {
    pub fn new(mut decoder: Decoder<R>) -> Result<Self, DecodingError> {
        let width = decoder.screen_descriptor.width as usize;
        let height = decoder.screen_descriptor.height as usize;
        let pixel_count = width * height;

        let bg_color = Color::transparent();

        Ok(Self {
            decoder,
            canvas: vec![bg_color; pixel_count],
            last_canvas: vec![bg_color; pixel_count],
            last_disposal: DisposalMethod::NoAction,
            last_rect: (0, 0, 0, 0),
            bg_color,
        })
    }

    fn dispose_previous(&mut self, screen_width: usize) {
        let (x, y, w, h) = self.last_rect;

        match self.last_disposal {
            DisposalMethod::NoAction | DisposalMethod::DoNotDispose => {},

            DisposalMethod::RestoreBackground => {
                for row in 0..h {
                    for col in 0..w {
                        let idx = (y + row) as usize * screen_width + (x + col) as usize;
                        if idx < self.canvas.len() {
                            self.canvas[idx] = self.bg_color;
                        }
                    }
                }
            },

            DisposalMethod::RestorePrevious => {
                self.canvas.clone_from_slice(&self.last_canvas);
            },

            _ => {}
        }
    }
}

impl<R: Read> Iterator for GifStream<R> {
    type Item = Result<Vec<Color>, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_frame = match self.decoder.next_frame() {
            Ok(Some(f)) => f,
            Ok(None) => return None,
            Err(e) => return Some(Err(e)),
        };

        let screen_width = self.decoder.screen_descriptor.width as usize;

        self.dispose_previous(screen_width);

        if raw_frame.disposal == DisposalMethod::RestorePrevious {
            self.last_canvas.copy_from_slice(&self.canvas);
        }

        for (i, &pixel_color) in raw_frame.pixels.iter().enumerate() {
            if pixel_color.a == 0 {
                continue;
            }

            let local_x = i % raw_frame.width as usize;
            let local_y = i / raw_frame.width as usize;

            let global_x = raw_frame.left as usize + local_x;
            let global_y = raw_frame.top as usize + local_y;

            if global_x < screen_width && global_y < self.decoder.screen_descriptor.height as usize {
                let canvas_idx = global_y * screen_width + global_x;
                self.canvas[canvas_idx] = pixel_color;
            }
        }

        self.last_disposal = raw_frame.disposal;
        self.last_rect = (raw_frame.left, raw_frame.top, raw_frame.width, raw_frame.height);

        Some(Ok(self.canvas.clone()))
    }
}
