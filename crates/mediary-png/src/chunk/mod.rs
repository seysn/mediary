mod bkgd;
mod chrm;
mod gama;
mod idat;
mod iend;
mod ihdr;
mod plte;
mod text;
mod time;
mod trns;

use std::{
    fmt::Display,
    io::{BufRead, Seek},
};

pub use bkgd::BackgroundColor;
pub use chrm::PrimaryChromaticities;
pub use gama::ImageGamma;
pub use idat::ImageData;
pub use iend::ImageTrailer;
pub use ihdr::{BitDepth, ColorType, ImageHeader};
pub use plte::{Palette, PaletteColor};
pub use text::TextualData;
pub use trns::Transparency;

use crate::{
    chunk::time::ImageLastModificationTime,
    error::{PngError, PngResult},
};

#[derive(Debug)]
pub enum PngChunk {
    /// IHDR
    ImageHeader(ImageHeader),

    /// PLTE
    Palette(Palette),

    /// IDAT
    ImageData(ImageData),

    /// IEND
    ImageTrailer(ImageTrailer),

    /// tRNS
    Transparency(Transparency),

    /// cHRM
    PrimaryChromaticities(PrimaryChromaticities),

    /// gAMA
    ImageGamma(ImageGamma),

    /// tEXt
    TextualData(TextualData),

    /// bKGD
    BackgroundColor(BackgroundColor),

    /// tIME
    ImageLastModificationTime(ImageLastModificationTime),
}

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
            ImageHeader::ID => Ok(Self::ImageHeader(ImageHeader::parse(&data)?)),
            Palette::ID => Ok(Self::Palette(Palette::parse(&data)?)),
            ImageData::ID => Ok(Self::ImageData(ImageData::parse(data))),
            ImageTrailer::ID => Ok(Self::ImageTrailer(ImageTrailer)),
            Transparency::ID => Ok(Self::Transparency(Transparency::parse(&data)?)),
            PrimaryChromaticities::ID => Ok(Self::PrimaryChromaticities(
                PrimaryChromaticities::parse(&data)?,
            )),
            ImageGamma::ID => Ok(Self::ImageGamma(ImageGamma::parse(&data)?)),
            TextualData::ID => Ok(Self::TextualData(TextualData::parse(&data))),
            BackgroundColor::ID => Ok(Self::BackgroundColor(BackgroundColor::parse(&data)?)),
            ImageLastModificationTime::ID => Ok(Self::ImageLastModificationTime(
                ImageLastModificationTime::parse(&data)?,
            )),
            _ => Err(PngError::UnknownChunk(chunk_type)),
        }
    }

    pub fn string_id(&self) -> &'static str {
        match self {
            PngChunk::ImageHeader(_) => ImageHeader::STRING_ID,
            PngChunk::Palette(_) => Palette::STRING_ID,
            PngChunk::ImageData(_) => ImageData::STRING_ID,
            PngChunk::ImageTrailer(_) => ImageTrailer::STRING_ID,
            PngChunk::Transparency(_) => Transparency::STRING_ID,
            PngChunk::PrimaryChromaticities(_) => PrimaryChromaticities::STRING_ID,
            PngChunk::ImageGamma(_) => ImageGamma::STRING_ID,
            PngChunk::TextualData(_) => TextualData::STRING_ID,
            PngChunk::BackgroundColor(_) => BackgroundColor::STRING_ID,
            PngChunk::ImageLastModificationTime(_) => ImageLastModificationTime::STRING_ID,
        }
    }
}

impl Display for PngChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.string_id())
    }
}
