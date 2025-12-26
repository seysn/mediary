use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use mediary_image::RgbImage;

use crate::error::{PnmError, PnmResult};

pub mod error;
pub mod reader;
pub mod writer;

pub struct PnmImage {
    pub format: PnmFormat,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum PnmFormat {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

pub enum PnmEncoding {
    Ascii,
    Binary,
}

impl PnmImage {
    pub fn new(image: RgbImage) -> Self {
        let width = image.width();
        let height = image.height();
        let data = image.into_data();

        Self {
            format: PnmFormat::P6,
            width,
            height,
            data,
        }
    }

    pub fn into_rgb(self) -> RgbImage {
        RgbImage::new(self.data, self.width, self.height).expect("dimensions should be correct")
    }

    pub fn write<P: AsRef<Path>>(self, path: P) -> PnmResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        writer::PnmWriter::new(self, writer).write()
    }

    pub fn read<P: AsRef<Path>>(path: P) -> PnmResult<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        reader::PnmReader::new(reader).read()
    }
}

impl PnmFormat {
    pub fn new(buf: &[u8; 2]) -> PnmResult<Self> {
        match buf {
            b"P1" => Ok(Self::P1),
            b"P2" => Ok(Self::P2),
            b"P3" => Ok(Self::P3),
            b"P4" => Ok(Self::P4),
            b"P5" => Ok(Self::P5),
            b"P6" => Ok(Self::P6),
            _ => Err(PnmError::InvalidFormat { got: *buf }),
        }
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            PnmFormat::P1 => *b"P1",
            PnmFormat::P2 => *b"P2",
            PnmFormat::P3 => *b"P3",
            PnmFormat::P4 => *b"P4",
            PnmFormat::P5 => *b"P5",
            PnmFormat::P6 => *b"P6",
        }
    }

    #[inline(always)]
    pub fn byte_per_pixel(&self) -> usize {
        match self {
            PnmFormat::P1 | PnmFormat::P4 | PnmFormat::P2 | PnmFormat::P5 => 1,
            PnmFormat::P3 | PnmFormat::P6 => 3,
        }
    }

    #[inline(always)]
    pub fn encoding(&self) -> PnmEncoding {
        match self {
            PnmFormat::P1 | PnmFormat::P2 | PnmFormat::P3 => PnmEncoding::Ascii,
            PnmFormat::P4 | PnmFormat::P5 | PnmFormat::P6 => PnmEncoding::Binary,
        }
    }
}
