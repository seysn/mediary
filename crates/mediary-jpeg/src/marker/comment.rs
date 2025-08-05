use std::io::{BufRead, Seek};

use crate::{error::JpegResult, reader::read_u16};

#[derive(Debug)]
pub struct Comment(pub String);

impl Comment {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Ok(Self(
            String::from_utf8_lossy(&data[..length as usize - 3]).to_string(),
        ))
    }
}
