pub(super) const ID: [u8; 4] = *b"PLTE";

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
    pub fn parse(bytes: &[u8]) -> Self {
        Self {
            colors: bytes
                .chunks_exact(3)
                .map(|chunk| PaletteColor {
                    red: chunk[0],
                    green: chunk[1],
                    blue: chunk[2],
                })
                .collect(),
        }
    }
}
