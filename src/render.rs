use crate::{
    error::DecodingError,
    structs::{Color, GraphicControl, Palette},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GifColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl GifColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn map_indices_to_rgba(
        index_buffer: &[u8],
        palette: &Palette,
        graphic_control_extension: &Option<GraphicControl>,
        rgba_buffer: &mut [GifColor],
    ) -> Result<(), DecodingError> {
        let transparent_index = graphic_control_extension.as_ref().and_then(|g| g.transparent_color_index);

        for (i, &index) in index_buffer.iter().enumerate() {
            let alpha = match transparent_index {
                Some(t_index) if index == t_index => 0,
                _ => 255,
            };

            let rgb_color = if (index as usize) < palette.len() {
                palette[index as usize]
            } else {
                Color::default()
            };

            rgba_buffer[i] = GifColor::new(rgb_color.r, rgb_color.g, rgb_color.b, alpha);
        }

        Ok(())
    }
}
