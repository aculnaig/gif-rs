use crate::{render::GifColor, structs::DisposalMethod};

#[derive(Debug, Clone)]
pub struct Frame {
    pub delay_cs: u16,
    pub disposal: DisposalMethod,
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<GifColor>,
    pub transparent_index: Option<u8>,
}
