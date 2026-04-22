use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u16),

    #[error("Corrupted file: {0}")]
    Corrupted(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid magic: expected {expected:?}, got {actual:?}")]
    InvalidMagic { expected: [u8; 4], actual: [u8; 4] },

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid data offset: {0}")]
    InvalidOffset(u32),

    #[error("Image decode error: {0}")]
    ImageDecode(String),

    #[error("Image encode error: {0}")]
    ImageEncode(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<image::ImageError> for IoError {
    fn from(e: image::ImageError) -> Self {
        IoError::ImageDecode(e.to_string())
    }
}
