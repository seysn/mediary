use thiserror::Error;

pub type BmpResult<T> = std::result::Result<T, BmpError>;

#[derive(Debug, Error)]
pub enum BmpError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid value '{value:?}' on element {element}")]
    InvalidValue {
        element: &'static str,
        value: Box<dyn std::fmt::Debug>,
    },
}
