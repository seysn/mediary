use std::fmt::Debug;

pub const ID: [u8; 4] = *b"IDAT";

pub struct ImageData {
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum CompressionMethod {
    Deflate,
    Unknown,
}

#[derive(Debug)]
pub enum CompressionLevel {
    Fastest,
    Fast,
    Default,
    Maximum,
}

impl ImageData {
    pub fn parse(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Return compression method.
    /// Should be used on first IDAT chunk only.
    pub fn compression_method(&self) -> CompressionMethod {
        CompressionMethod::new(self.data[0] & 0x0F)
    }

    /// Return maximum allowed value.
    /// Should be used on first IDAT chunk only.
    pub fn maximum_allowed_value(&self) -> u32 {
        let compression_info = (self.data[0] & 0xF0) >> 4;
        2_u32.pow(compression_info as u32 + 8)
    }

    /// Return FCHECK value.
    /// Should be used on first IDAT chunk only.
    pub fn fcheck(&self) -> u32 {
        (self.data[1] & 0x1F).into()
    }

    /// Return FDICT value.
    /// Should be used on first IDAT chunk only.
    pub fn fdict(&self) -> bool {
        (self.data[1] & 0x20) > 0
    }

    /// Return compression level.
    /// Should be used on first IDAT chunk only.
    pub fn compression_level(&self) -> CompressionLevel {
        CompressionLevel::new((self.data[1] & 0xc0) >> 6)
    }
}

impl CompressionMethod {
    pub fn new(data: u8) -> Self {
        match data {
            8 => Self::Deflate,
            _ => Self::Unknown,
        }
    }
}

impl CompressionLevel {
    pub fn new(data: u8) -> Self {
        match data {
            0 => Self::Fastest,
            1 => Self::Fast,
            2 => Self::Default,
            3 => Self::Maximum,
            _ => Self::Default,
        }
    }
}

impl Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData")
            .field("data", &format!("{} bytes", self.data.len()))
            .finish()
    }
}
