use thiserror::Error;

pub type PngResult<T> = std::result::Result<T, PngError>;

#[derive(Debug, Error)]
pub enum PngError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid Signature")]
    InvalidSignature,

    #[error("Invalid Chunk {0:02x?}")]
    InvalidChunk([u8; 4]),
}
