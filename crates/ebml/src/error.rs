use thiserror::Error;

pub type EbmlResult<T> = std::result::Result<T, EbmlError>;

#[derive(Debug, Error)]
pub enum EbmlError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Vint")]
    InvalidVint,
}
