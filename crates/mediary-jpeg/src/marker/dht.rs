use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, Seek},
};

use mediary_common::huffman::{HuffmanCode, HuffmanTable};

use crate::{
    error::{JpegError, JpegResult},
    reader::read_u16,
};

#[derive(Clone)]
pub struct DefineHuffmanTable {
    pub class: TableClass,
    pub index: u8,
    pub counts: [u8; 16],
    pub values: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum TableClass {
    DC,
    AC,
}

impl DefineHuffmanTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        let class = TableClass::try_from((data[0] >> 4) & 0x0f)?;
        let index = data[0] & 0x0f;
        let counts: [u8; 16] = data[1..17]
            .try_into()
            .expect("slice should have a slice of 16 elements");
        let values = data[17..].to_vec();

        Ok(Self {
            class,
            index,
            counts,
            values,
        })
    }

    pub fn to_table(&self) -> JpegResult<HuffmanTable> {
        let mut codes = HashMap::new();

        let mut idx = 0;
        let mut code = 0;
        for (l, &count) in self.counts.iter().enumerate() {
            let length = l + 1;
            for _ in 0..count {
                codes.insert(HuffmanCode::new(code, length as u8), self.values[idx]);

                code += 1;
                idx += 1;
            }

            code <<= 1;
        }

        Ok(HuffmanTable::new(codes)?)
    }
}

impl TryFrom<u8> for TableClass {
    type Error = JpegError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::DC,
            1 => Self::AC,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "TableClass",
                    value: Box::new(value),
                })
            }
        })
    }
}

impl Debug for DefineHuffmanTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefineHuffmanTable")
            .field("class", &self.class)
            .field("index", &self.index)
            .field("counts", &format_args!("{:?}", self.counts))
            .field("values", &format_args!("{:?}", self.values))
            .finish()
    }
}
