use crate::error::PngResult;

#[derive(Debug)]
pub struct Transparency;

impl Transparency {
    pub const ID: [u8; 4] = *b"tRNS";
    pub const STRING_ID: &str = "tRNS";

    pub fn parse(_bytes: &[u8]) -> PngResult<Self> {
        // TODO
        Ok(Self)
    }
}
