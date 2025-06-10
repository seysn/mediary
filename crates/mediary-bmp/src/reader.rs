use std::fmt::Debug;
use std::io::{Read, Seek, SeekFrom};

use crate::{
    error::BmpResult,
    header::{BMPHeader, DIBHeader},
};

pub struct Bmp {
    /// Bitmap File Header
    pub header: BMPHeader,

    /// DIB Header
    pub dib: DIBHeader,

    /// Raw data without padding bytes
    pub data: Vec<u8>,
}

pub struct BmpReader<R: Read + Seek> {
    reader: R,
}

impl Bmp {
    pub fn from_reader<R: Read + Seek>(reader: R) -> BmpResult<Self> {
        BmpReader::new(reader).decode()
    }
}

impl<R: Read + Seek> BmpReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn decode(&mut self) -> BmpResult<Bmp> {
        let header = BMPHeader::from_reader(&mut self.reader)?;
        let dib = DIBHeader::from_reader(&mut self.reader)?;

        self.reader
            .seek(SeekFrom::Start(header.data_offset.into()))?;

        let channels = dib.bits_per_pixel / 8;
        let chunk_size = dib.width as usize * channels as usize;
        let padding = (4 - (dib.width * 3).rem_euclid(4)).rem_euclid(4);

        let mut data = vec![0; chunk_size * dib.height as usize];
        for row in data.chunks_mut(chunk_size) {
            self.reader.read_exact(row)?;

            if padding > 0 {
                self.reader.seek(SeekFrom::Current(padding.into()))?;
            }
        }

        Ok(Bmp { header, dib, data })
    }
}

impl Debug for Bmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bmp")
            .field("header", &self.header)
            .field("dib", &self.dib)
            .field("data_length", &self.data.len())
            .finish()
    }
}
