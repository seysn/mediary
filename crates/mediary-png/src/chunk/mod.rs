mod bkgd;
mod chrm;
mod gama;
mod idat;
mod ihdr;
mod plte;

use std::io::{BufRead, Seek};

pub use bkgd::BackgroundColor;
pub use chrm::PrimaryChromaticities;
pub use gama::ImageGamma;
pub use idat::ImageData;
pub use ihdr::{BitDepth, ColorType, ImageHeader};
pub use plte::{Palette, PaletteColor};

use crate::error::{PngError, PngResult};

#[derive(Debug)]
pub enum PngChunk {
    /// IHDR
    ImageHeader(ImageHeader),

    /// PLTE
    Palette(Palette),

    /// IDAT
    ImageData(ImageData),

    /// IEND
    ImageTrailer,

    /// tRNS
    Transparency,

    /// cHRM
    PrimaryChromaticities(PrimaryChromaticities),

    /// gAMA
    ImageGamma(ImageGamma),

    /// tEXt
    TextualData(String),

    /// bKGD
    BackgroundColor(BackgroundColor),

    /// tIME
    ImageLastModificationTime(Vec<u8>),
}

const IEND: [u8; 4] = *b"IEND";
const TRNS: [u8; 4] = *b"tRNS";
const TEXT: [u8; 4] = *b"tEXt";
const TIME: [u8; 4] = *b"tIME";

impl PngChunk {
    pub fn read<R: BufRead + Seek>(reader: &mut R) -> PngResult<Self> {
        // Reading data length
        let mut data_length = [0; 4];
        reader.read_exact(&mut data_length)?;
        let data_length = u32::from_be_bytes(data_length);

        // Reading chunk type
        let mut chunk_type = [0; 4];
        reader.read_exact(&mut chunk_type)?;

        // Reading data based on length
        let data = if data_length > 0 {
            let mut data = vec![0; data_length as usize];
            reader.read_exact(&mut data)?;
            data
        } else {
            vec![]
        };

        // Reading CRC
        let mut crc = [0; 4];
        reader.read_exact(&mut crc)?;

        Self::parse(chunk_type, data)
    }

    pub fn parse(chunk_type: [u8; 4], data: Vec<u8>) -> PngResult<Self> {
        match chunk_type {
            ihdr::ID => Ok(Self::ImageHeader(ImageHeader::parse(&data))),
            plte::ID => Ok(Self::Palette(Palette::parse(&data))),
            idat::ID => Ok(Self::ImageData(ImageData::parse(data))),
            IEND => Ok(Self::ImageTrailer),
            TRNS => Ok(Self::Transparency),
            chrm::ID => Ok(Self::PrimaryChromaticities(PrimaryChromaticities::parse(
                &data,
            ))),
            gama::ID => Ok(Self::ImageGamma(ImageGamma::parse(&data))),
            TEXT => Ok(Self::TextualData(
                String::from_utf8_lossy(&data).to_string(),
            )),
            bkgd::ID => Ok(Self::BackgroundColor(BackgroundColor::parse(&data))),
            TIME => Ok(Self::ImageLastModificationTime(data)),
            _ => Err(PngError::InvalidChunk(chunk_type)),
        }
    }
}
