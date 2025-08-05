use std::{
    fmt::Debug,
    io::{BufRead, Seek},
};

use crate::{error::JpegResult, reader::read_u16};

pub struct DefineQuantizationTable(pub Vec<u8>);

impl DefineQuantizationTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Ok(Self(data))
    }
}

impl Debug for DefineQuantizationTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dqt({:?})", self.0)
    }
}
