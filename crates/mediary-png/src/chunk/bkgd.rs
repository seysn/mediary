use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub enum BackgroundColor {
    Greyscale { value: u16 },
    Rgb { r: u16, g: u16, b: u16 },
    Palette { index: u8 },
}

impl BackgroundColor {
    pub const ID: [u8; 4] = *b"bKGD";
    pub const STRING_ID: &str = "bKGD";

    pub fn parse(bytes: &[u8]) -> PngResult<Self> {
        Ok(match bytes.len() {
            1 => Self::Palette { index: bytes[0] },
            2 => Self::Greyscale {
                value: u16::from_be_bytes(bytes.try_into().expect("bytes should be 2 bytes long")),
            },
            6 => Self::Rgb {
                r: u16::from_be_bytes(
                    bytes[0..2]
                        .try_into()
                        .expect("bytes should be 2 bytes long"),
                ),
                g: u16::from_be_bytes(
                    bytes[2..4]
                        .try_into()
                        .expect("bytes should be 2 bytes long"),
                ),
                b: u16::from_be_bytes(
                    bytes[4..6]
                        .try_into()
                        .expect("bytes should be 2 bytes long"),
                ),
            },
            _ => {
                return Err(PngError::InvalidChunkData {
                    chunk_id: Self::STRING_ID,
                })
            }
        })
    }
}
