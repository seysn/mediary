use thiserror::Error;

pub type PnmResult<T> = std::result::Result<T, PnmError>;

#[derive(Debug, Error)]
pub enum PnmError {
    #[error("Invalid format '{got:?}'")]
    InvalidFormat { got: [u8; 2] },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
