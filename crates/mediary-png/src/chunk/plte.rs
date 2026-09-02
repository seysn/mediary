use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub struct Palette {
    pub colors: Vec<PaletteColor>,
}

#[derive(Debug)]
pub struct PaletteColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Palette {
    pub const ID: [u8; 4] = *b"PLTE";
    pub const STRING_ID: &str = "PLTE";

    pub fn parse(bytes: &[u8]) -> PngResult<Self> {
        if !bytes.len().is_multiple_of(3) {
            return Err(PngError::InvalidChunkData {
                chunk_id: Self::STRING_ID,
            });
        }

        Ok(Self {
            colors: bytes
                .as_chunks::<3>()
                .0
                .iter()
                .map(|chunk| PaletteColor {
                    red: chunk[0],
                    green: chunk[1],
                    blue: chunk[2],
                })
                .collect(),
        })
    }
}
