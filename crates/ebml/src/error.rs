use thiserror::Error;

pub type EbmlResult<T> = std::result::Result<T, EbmlError>;

#[derive(Debug, Error)]
pub enum EbmlError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Vint")]
    InvalidVint,
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Unexpected element (expected '{expected}' but found '{found}')")]
    UnexpectedElement {
        expected: &'static str,
        found: &'static str,
    },
    #[error("Unexpected element type (expected '{expected}' but found '{found}')")]
    UnexpectedElementType {
        expected: &'static str,
        found: &'static str,
    },
}
