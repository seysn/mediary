use std::{
    fmt::Debug,
    io::{BufRead, Seek, Write},
};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::{JpegError, JpegResult},
    reader::{read_u16, read_u8},
};

#[derive(Debug, Clone)]
pub struct DefineQuantizationTable(pub Vec<QuantizationTable>);

#[derive(Debug, Clone)]
pub struct QuantizationTable {
    pub precision: u8,
    pub index: u8,
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
            dqt.push(QuantizationTable::from_reader(reader)?)
        }

        Ok(Self(dqt))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        let length = 2 + (65 * self.0.len());
        let mut buf = [0; 2];
        BigEndian::write_u16(&mut buf, length as u16);
        writer.write_all(&buf)?;

        for qt in &self.0 {
            qt.write(writer)?;
        }

        Ok(())
    }
}

impl QuantizationTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let b = read_u8(reader)?;
        let precision = (b >> 4) & 0xf;
        let index = b & 0xf;

        if precision != 0 {
            todo!("Precision {precision}");
        }

        let mut values = [0; 64];
        reader.read_exact(&mut values)?;

        Ok(Self {
            precision,
            index,
            values: QuantizationTableValues(values),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        let mut buf = [0; 65];
        buf[0] = ((self.precision & 0xf) << 4) + (self.index & 0xf);
        buf[1..].copy_from_slice(&self.values.0);

        writer.write_all(&buf)?;

        Ok(())
    }
}

impl Debug for QuantizationTableValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
