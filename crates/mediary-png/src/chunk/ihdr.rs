use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: BitDepth,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum BitDepth {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
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
    pub const ID: [u8; 4] = *b"IHDR";
    pub const STRING_ID: &str = "IHDR";

    pub fn parse(bytes: &[u8]) -> PngResult<Self> {
        if bytes.len() != 12 {
            return Err(PngError::InvalidChunkData {
                chunk_id: Self::STRING_ID,
            });
        }

        Ok(Self {
            width: u32::from_be_bytes(
                bytes[0..4]
                    .try_into()
                    .expect("bytes should be 4 bytes long"),
            ),
            height: u32::from_be_bytes(
                bytes[4..8]
                    .try_into()
                    .expect("bytes should be 4 bytes long"),
            ),
            bit_depth: BitDepth::parse(bytes[8]),
            color_type: ColorType::parse(bytes[9]),
            compression_method: bytes[10],
            filter_method: bytes[11],
            interlace_method: bytes[12],
        })
    }

    pub fn row_size(&self) -> usize {
        let color_bytes = self.width as usize * self.color_type.channels();

        // Row size includes type byte
        1 + match self.bit_depth {
            BitDepth::One => todo!(),
            BitDepth::Two => todo!(),
            BitDepth::Four => color_bytes / 2,
            BitDepth::Eight => color_bytes,
            BitDepth::Sixteen => color_bytes * 2,
        }
    }
}

impl BitDepth {
    pub fn parse(byte: u8) -> Self {
        match byte {
            1 => Self::One,
            2 => Self::Two,
            4 => Self::Four,
            8 => Self::Eight,
            16 => Self::Sixteen,
            _ => todo!(),
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
