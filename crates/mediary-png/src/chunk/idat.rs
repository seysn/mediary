use std::fmt::Debug;

pub struct ImageData(pub Vec<u8>);

impl ImageData {
    pub const ID: [u8; 4] = *b"IDAT";
    pub const STRING_ID: &str = "IDAT";

    pub fn parse(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData")
            .field("data", &format!("{} bytes", self.0.len()))
            .finish()
    }
}
