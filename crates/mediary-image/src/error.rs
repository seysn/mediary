use thiserror::Error;

pub type ImageResult<T> = std::result::Result<T, ImageError>;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Out of bounds")]
    OutOfBounds,
}
