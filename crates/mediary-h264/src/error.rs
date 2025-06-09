use thiserror::Error;

pub type H264Result<T> = std::result::Result<T, H264Error>;

#[derive(Debug, Error)]
pub enum H264Error {
    #[error("Unexpected end of file")]
    UnexpectedEof,
}
