use std::{fmt, io};

#[derive(Debug)]
pub enum DecodingError {
    Io(io::Error),
    InvalidSignature,
    Format(String),
    Unuspported(String),
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
