use std::{
    fmt::Debug,
    io::{BufRead, Seek},
};

use crate::{
    error::{JpegError, JpegResult},
    reader::{read_u16, read_u8},
};

#[derive(Debug, Clone)]
pub struct DefineQuantizationTable(pub Vec<QuantizationTable>);

#[derive(Debug, Clone)]
pub struct QuantizationTable {
    pub precision: u8,
    pub destination: u8,
    pub values: QuantizationTableValues,
}

#[derive(Clone)]
pub struct QuantizationTableValues(pub [u8; 64]);

impl DefineQuantizationTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        if (length - 2) % 65 != 0 {
            return Err(JpegError::InvalidValue {
                element: "DQT length",
                value: Box::new(length),
            });
        }

        let mut dqt = Vec::new();
        for _ in 0..(length - 2) / 65 {
            let b = read_u8(reader)?;
            let precision = (b >> 4) & 0xf;
            let destination = b & 0xf;

            if precision != 0 {
                todo!("Precision {precision}");
            }

            let mut values = [0; 64];
            reader.read_exact(&mut values)?;

            dqt.push(QuantizationTable {
                precision,
                destination,
                values: QuantizationTableValues(values),
            })
        }

        Ok(Self(dqt))
    }
}

impl Debug for QuantizationTableValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
