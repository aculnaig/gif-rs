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
        match self {
            DecodingError::Io(err) => write!(f, "IO error: {}", err),
            DecodingError::InvalidSignature => {
                write!(f, "Invalid GIF signature")
            }
            DecodingError::Format(msg) => {
                write!(f, "Format error: {}", msg)
            }
            DecodingError::Unuspported(msg) => {
                write!(f, "Unsupported feature: {}", msg)
            }
        }
    }
}

impl std::error::Error for DecodingError {}

impl From<io::Error> for DecodingError {
    fn from(err: io::Error) -> Self {
        DecodingError::Io(err)
    }
}
