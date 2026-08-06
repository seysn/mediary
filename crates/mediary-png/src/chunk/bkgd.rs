pub const ID: [u8; 4] = *b"bKGD";

#[derive(Debug)]
pub enum BackgroundColor {
    Greyscale { value: u16 },
    Rgb { r: u16, g: u16, b: u16 },
    Palette { index: u8 },
}

impl BackgroundColor {
    pub fn parse(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => Self::Palette { index: bytes[0] },
            2 => Self::Greyscale {
                value: u16::from_be_bytes(bytes.try_into().unwrap()),
            },
            6 => Self::Rgb {
                r: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                g: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
                b: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
            },
            _ => todo!(),
        }
    }
}
