use thiserror::Error;

pub type NetpbmResult<T> = std::result::Result<T, NetpbmError>;

#[derive(Debug, Error)]
pub enum NetpbmError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
