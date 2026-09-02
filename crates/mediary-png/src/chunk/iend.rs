#[derive(Debug)]
pub struct ImageTrailer;

impl ImageTrailer {
    pub const ID: [u8; 4] = *b"IEND";
    pub const STRING_ID: &str = "IEND";
}
