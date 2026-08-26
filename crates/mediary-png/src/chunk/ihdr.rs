pub(super) const ID: [u8; 4] = *b"IHDR";

#[derive(Debug)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug)]
pub enum ColorType {
    /// Mono
    Greyscale,

    /// RGB
    Truecolor,

    /// Palette index
    IndexedColor,

    /// Mono with alpha
    GreyscaleWithAlpha,

    /// RGBA
    TrueColorWithAlpha,
}

impl ImageHeader {
    pub fn parse(bytes: &[u8]) -> Self {
        Self {
            width: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            height: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            bit_depth: bytes[8], // TODO: check if bit_depth is allowed with color_type
            color_type: ColorType::parse(bytes[9]),
            compression_method: bytes[10],
            filter_method: bytes[11],
            interlace_method: bytes[12],
        }
    }
}

impl ColorType {
    pub fn parse(byte: u8) -> Self {
        match byte {
            0 => Self::Greyscale,
            2 => Self::Truecolor,
            3 => Self::IndexedColor,
            4 => Self::GreyscaleWithAlpha,
            6 => Self::TrueColorWithAlpha,
            _ => todo!(),
        }
    }

    pub fn channels(&self) -> usize {
        match self {
            ColorType::Greyscale => 1,
            ColorType::Truecolor => 3,
            ColorType::IndexedColor => 1,
            ColorType::GreyscaleWithAlpha => 2,
            ColorType::TrueColorWithAlpha => 4,
        }
    }
}
