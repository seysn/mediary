use thiserror::Error;

pub type PngResult<T> = std::result::Result<T, PngError>;

#[derive(Debug, Error)]
pub enum PngError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid Signature")]
    InvalidSignature,

    #[error("Unknown Chunk {0:02x?}")]
    UnknownChunk([u8; 4]),

    #[error("Unexpected Chunk (expected {expected} but found {found})")]
    UnexpectedChunk {
        expected: &'static str,
        found: &'static str,
    },

    #[error("Missing chunk {0}")]
    MissingChunk(&'static str),

    #[error("Invalid Data in chunk {chunk_id}")]
    InvalidChunkData { chunk_id: &'static str },
}
