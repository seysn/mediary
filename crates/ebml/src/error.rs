use thiserror::Error;

pub type EbmlResult<T> = std::result::Result<T, EbmlError>;

#[derive(Debug, Error)]
pub enum EbmlError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Vint")]
    InvalidVint,
    #[error("Invalid data length")]
    InvalidDataLength,
    #[error("Missing element '{0}'")]
    MissingElement(&'static str),
    #[error("Unexpected element (expected '{expected}' but found '{found}')")]
    UnexpectedElement {
        expected: &'static str,
        found: &'static str,
    },
}
