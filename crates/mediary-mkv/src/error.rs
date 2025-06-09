use thiserror::Error;

pub type MkvResult<T> = std::result::Result<T, MkvError>;

#[derive(Debug, Error)]
pub enum MkvError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Ebml(#[from] mediary_ebml::error::EbmlError),
    #[error("{0}")]
    H264(#[from] mediary_h264::error::H264Error),
    #[error("Invalid value '{value:?}' on element {element}")]
    InvalidValue {
        element: &'static str,
        value: Box<dyn std::fmt::Debug>,
    },
}
