use std::io::{BufRead, Seek};

use crate::{
    error::JpegResult,
    reader::{read_u16, read_u8},
};

#[derive(Debug, Clone)]
pub struct StartOfScan {
    pub components: Vec<(u8, u8, u8)>,
    pub start_spectral: u8,
    pub end_spectral: u8,
    pub approximation_bit: u8,
    pub data: ImageData,
}

#[derive(Clone)]
pub struct ImageData(pub Vec<u8>);

impl StartOfScan {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let _header_length = read_u16(reader)?;
        let n_components = read_u8(reader)?;

        let mut components = Vec::new();
        for _ in 0..n_components {
            let cs = read_u8(reader)?;
            let b = read_u8(reader)?;
            let td = (b >> 4) & 0xf;
            let ta = b & 0xf;

            components.push((cs, td, ta));
        }

        let start_spectral = read_u8(reader)?;
        let end_spectral = read_u8(reader)?;
        let approximation_bit = read_u8(reader)?;

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(Self {
            components,
            start_spectral,
            end_spectral,
            approximation_bit,
            data: ImageData(data),
        })
    }
}

impl std::fmt::Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ {} bytes ]", self.0.len())
    }
}
