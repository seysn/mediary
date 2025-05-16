use thiserror::Error;

pub type Mp4Result<T> = std::result::Result<T, Mp4Error>;

#[derive(Debug, Error)]
pub enum Mp4Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Header")]
    InvalidHeader,
}
