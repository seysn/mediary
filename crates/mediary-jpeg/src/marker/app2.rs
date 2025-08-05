use std::io::{BufRead, Seek};

use crate::{error::JpegResult, reader::read_u16};

#[derive(Debug)]
pub enum App2 {
    Icc,
    FlashPix,
}

impl App2 {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        App2::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        if data.starts_with(b"ICC_PROFILE") {
            Ok(Self::Icc)
        } else {
            todo!("FlashPix")
        }
    }
}
