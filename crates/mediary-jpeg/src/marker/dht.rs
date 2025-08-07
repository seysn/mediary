use std::{
    fmt::Debug,
    io::{BufRead, Seek},
};

use mediary_common::huffman::HuffmanTable;

use crate::{
    error::{JpegError, JpegResult},
    reader::read_u16,
};

#[derive(Clone)]
pub struct DefineHuffmanTable {
    pub table_class: TableClass,
    pub table_destination: u8,
    pub counts: [u8; 17],
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
        let table_class = TableClass::try_from((data[0] >> 4) & 0x0f)?;
        let table_destination = data[0] & 0x0f;
        let mut counts: [u8; 17] = data[0..17]
            .try_into()
            .expect("slice should have a slice of 16 elements");
        counts[0] = 0;
        let values = data[17..].to_vec();

        Ok(Self {
            table_class,
            table_destination,
            counts,
            values,
        })
    }

    pub fn to_table(&self) -> HuffmanTable {
        let mut huffsize = Vec::new();
        let mut huffcode = Vec::new();
        let mut values = Vec::new();

        let mut idx = 0;
        let mut code = 0;
        for (length, &count) in self.counts.iter().enumerate() {
            for _ in 0..count {
                huffsize.push(length as u8);
                huffcode.push(code);
                values.push(self.values[idx]);

                code += 1;
                idx += 1;
            }

            code <<= 1;
        }

        HuffmanTable::new(huffsize, huffcode, values)
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
            .field("table_class", &self.table_class)
            .field("table_destination", &self.table_destination)
            .field("counts", &format_args!("{:?}", self.counts))
            .field("values", &format_args!("{:?}", self.values))
            .finish()
    }
}
