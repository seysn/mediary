use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub struct ImageGamma(pub u64);

impl ImageGamma {
    pub const ID: [u8; 4] = *b"gAMA";
    pub const STRING_ID: &str = "gAMA";

    pub fn parse(bytes: &[u8]) -> PngResult<Self> {
        if bytes.len() != 4 {
            return Err(PngError::InvalidChunkData {
                chunk_id: Self::STRING_ID,
            });
        }

        Ok(Self(
            u32::from_be_bytes(bytes.try_into().expect("bytes should be 4 bytes long")) as u64
                * 100000,
        ))
    }
}
