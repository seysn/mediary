use std::{
    fmt::Debug,
    io::{BufRead, Seek},
};

use crate::{
    error::{JpegError, JpegResult},
    reader::read_u16,
};

#[derive(Clone)]
pub struct HuffmanTable {
    table_class: TableClass,
    table_destination: u8,
    n_codes: [u8; 16],
    codes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum TableClass {
    DC,
    AC,
}

impl HuffmanTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        let table_class = TableClass::try_from((data[0] >> 4) & 0x0f)?;
        let table_destination = data[0] & 0x0f;
        let n_codes = data[1..17]
            .try_into()
            .expect("slice should have a slice of 16 elements");
        let codes = data[17..].to_vec();

        Ok(Self {
            table_class,
            table_destination,
            n_codes,
            codes,
        })
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

impl Debug for HuffmanTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HuffmanTable")
            .field("table_class", &self.table_class)
            .field("table_destination", &self.table_destination)
            .field("n_codes", &format_args!("{:?}", self.n_codes))
            .field("codes", &format_args!("{:?}", self.codes))
            .finish()
    }
}
