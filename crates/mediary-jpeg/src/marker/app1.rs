use std::io::{BufRead, Seek};

use crate::{error::JpegResult, exif::ExifData, reader::read_u16};

#[derive(Debug)]
pub enum App1 {
    Exif(ExifData),
    Xmp(XmpData),
}

impl App1 {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        if data.starts_with(b"Exif\0\0") {
            Ok(Self::Exif(ExifData::from_bytes(&data[6..])?))
        } else if data.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
            Ok(Self::Xmp(XmpData::from_bytes(&data[29..])))
        } else {
            todo!()
        }
    }
}

#[derive(Clone)]
pub struct XmpData(pub String);

impl XmpData {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(String::from_utf8_lossy(data).to_string())
    }
}

impl std::fmt::Debug for XmpData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Using Display implementation of String so we can render newlines
        f.debug_tuple("XmpData")
            .field(&format_args!("{}", self.0))
            .finish()
    }
}
