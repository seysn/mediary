use std::io::Write;

use mediary_image::RgbImage;

use crate::error::NetpbmResult;

pub mod error;
pub mod writer;

pub struct NetpbmImage {
    pub format: NetpbmFormat,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum NetpbmFormat {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

impl NetpbmImage {
    pub fn new(image: RgbImage) -> Self {
        let RgbImage {
            width,
            height,
            data,
        } = image;

        Self {
            format: NetpbmFormat::P3,
            width,
            height,
            data,
        }
    }

    pub fn write<W: Write>(self, writer: W) -> NetpbmResult<()> {
        writer::NetpbmWriter::new(self, writer).write()
    }
}

impl NetpbmFormat {
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            NetpbmFormat::P1 => *b"P1",
            NetpbmFormat::P2 => *b"P2",
            NetpbmFormat::P3 => *b"P3",
            NetpbmFormat::P4 => *b"P4",
            NetpbmFormat::P5 => *b"P5",
            NetpbmFormat::P6 => *b"P6",
        }
    }

    pub fn byte_per_pixel(&self) -> usize {
        match self {
            NetpbmFormat::P1 | NetpbmFormat::P4 | NetpbmFormat::P2 | NetpbmFormat::P5 => 1,
            NetpbmFormat::P3 | NetpbmFormat::P6 => 3,
        }
    }
}
