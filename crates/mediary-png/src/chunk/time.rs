use jiff::civil::DateTime;

use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub struct ImageLastModificationTime(pub DateTime);

impl ImageLastModificationTime {
    pub const ID: [u8; 4] = *b"tIME";
    pub const STRING_ID: &str = "tIME";

    pub fn parse(bytes: &[u8]) -> PngResult<Self> {
        if bytes.len() != 8 {
            return Err(PngError::InvalidChunkData {
                chunk_id: Self::STRING_ID,
            });
        }

        let year = u16::from_be_bytes(
            bytes[0..2]
                .try_into()
                .expect("bytes should be 2 bytes long"),
        );

        let month = bytes[3];
        let day = bytes[4];
        let hour = bytes[5];
        let minute = bytes[6];
        let second = bytes[7];

        Ok(Self(
            DateTime::new(
                year as i16,
                month as i8,
                day as i8,
                hour as i8,
                minute as i8,
                second as i8,
                0,
            )
            .or(Err(PngError::InvalidChunkData {
                chunk_id: Self::STRING_ID,
            }))?,
        ))
    }
}
