use thiserror::Error;

pub type MkvResult<T> = std::result::Result<T, MkvError>;

#[derive(Debug, Error)]
pub enum MkvError {
    #[error("{0}")]
    Ebml(#[from] ebml::error::EbmlError),
}
