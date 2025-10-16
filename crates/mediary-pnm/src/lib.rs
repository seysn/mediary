use std::io::Write;

use mediary_image::RgbImage;

use crate::error::PnmResult;

pub mod error;
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

impl PnmImage {
    pub fn new(image: RgbImage) -> Self {
        let RgbImage {
            width,
            height,
            data,
        } = image;

        Self {
            format: PnmFormat::P3,
            width,
            height,
            data,
        }
    }

    pub fn write<W: Write>(self, writer: W) -> PnmResult<()> {
        writer::PnmWriter::new(self, writer).write()
    }
}

impl PnmFormat {
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

    pub fn byte_per_pixel(&self) -> usize {
        match self {
            PnmFormat::P1 | PnmFormat::P4 | PnmFormat::P2 | PnmFormat::P5 => 1,
            PnmFormat::P3 | PnmFormat::P6 => 3,
        }
    }
}
