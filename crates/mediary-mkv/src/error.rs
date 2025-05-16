use thiserror::Error;

pub type MkvResult<T> = std::result::Result<T, MkvError>;

#[derive(Debug, Error)]
pub enum MkvError {
    #[error("{0}")]
    Ebml(#[from] mediary_ebml::error::EbmlError),
    #[error("Invalid value '{value:?}' on element {element}")]
    InvalidValue {
        element: &'static str,
        value: Box<dyn std::fmt::Debug>,
    },
}
