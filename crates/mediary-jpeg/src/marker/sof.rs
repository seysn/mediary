use std::io::{BufRead, Seek};

use crate::{
    error::JpegResult,
    reader::{read_u16, read_u8},
};

#[derive(Debug, Clone)]
pub struct StartOfFrame {
    pub precision: u8,
    pub width: u16,
    pub height: u16,
    pub components: Vec<(u8, u8, u8, u8)>,
}

impl StartOfFrame {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let _length = read_u16(reader)?;
        let precision = read_u8(reader)?;
        let height = read_u16(reader)?;
        let width = read_u16(reader)?;
        let n_components = read_u8(reader)?;

        let mut components = Vec::new();
        for _ in 0..n_components {
            let c = read_u8(reader)?;
            let hv = read_u8(reader)?;
            let h = hv & 0xf0 >> 4;
            let v = hv & 0x0f;
            let tq = read_u8(reader)?;
            components.push((c, h, v, tq))
        }

        Ok(Self {
            precision,
            width,
            height,
            components,
        })
    }
}
