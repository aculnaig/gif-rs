#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalScreenDescriptor {
    pub width: u16,
    pub height: u16,
    pub packed_fields: u8,
    pub bg_color_index: u8,
    pub pixel_aspect_ration: u8,
}

impl LogicalScreenDescriptor {
    pub fn has_global_color_table(&self) -> bool {
        (self.packed_fields & 0b1000_0000) != 0
    }

    pub fn color_resolution(&self) -> u8 {
        ((self.packed_fields & 0b0111_0000) >> 4) + 1
    }

    pub fn sort_flag(&self) -> bool {
        (self.packed_fields & 0b0000_1000) != 0
    }

    pub fn global_color_table_size(&self) -> usize {
        let n = self.packed_fields & 0b0000_0111;
        1 << (n + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub type Palette = Vec<Color>;
