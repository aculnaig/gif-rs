use std::default;

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DisposalMethod {
    #[default]
    NoAction = 0,
    DoNotDispose = 1,
    RestoreBackground = 2,
    RestorePrevious = 3,
    Reserved = 4,
}

impl From<u8> for DisposalMethod {
    fn from(n: u8) -> Self {
        match n {
            0 => DisposalMethod::NoAction,
            1 => DisposalMethod::DoNotDispose,
            2 => DisposalMethod::RestoreBackground,
            3 => DisposalMethod::RestorePrevious,
            _ => DisposalMethod::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GraphicControl {
    pub disposal_method: DisposalMethod,
    pub user_input_flag: bool,
    pub transparent_color_index: Option<u8>,
    pub delay_time_cs: u16, // 1/100 seconds unit
}

#[derive(Debug, Clone, Copy)]
pub struct ImageDescriptor {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub packed: u8,
}

impl ImageDescriptor {
    pub fn has_local_palette(&self) -> bool {
        (self.packed & 0b1000_0000) != 0
    }

    pub fn is_interlaced(&self) -> bool {
        (self.packed & 0b0100_0000) != 0
    }

    pub fn local_palette_size(&self) -> usize {
        let n = self.packed & 0b0000_0111;
        1 << (n + 1)
    }
}
