use thiserror::Error;

pub type JpegResult<T> = std::result::Result<T, JpegError>;

#[derive(Debug, Error)]
pub enum JpegError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid value '{value:?}' on element {element}")]
    InvalidValue {
        element: &'static str,
        value: Box<dyn std::fmt::Debug>,
    },
}
