use std::{
    fmt::Debug,
    io::{BufRead, Seek, Write},
    ops::Index,
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

/// Table K.1 from ITU-T81
#[rustfmt::skip]
const LUMA_QUANTIZATION_TABLE: [u8; 64] = [
    16, 11, 10, 16,  24,  40,  51,  61,
    12, 12, 14, 19,  26,  58,  60,  55,
    14, 13, 16, 24,  40,  57,  69,  56,
    14, 17, 22, 29,  51,  87,  80,  62,
    18, 22, 37, 56,  68, 109, 103,  77,
    24, 35, 55, 64,  81, 104, 113,  92,
    49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103,  99,
];

/// Table K.2 from ITU-T81
#[rustfmt::skip]
const CHROMA_QUANTIZATION_TABLE: [u8; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99,
    18, 21, 26, 66, 99, 99, 99, 99,
    24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
];

impl QuantizationTableValues {
    pub fn new(quality: u8, mut quantization_table: [u8; 64]) -> Self {
        let quality = u32::from(quality.clamp(1, 100));
        let scale = if quality < 50 {
            5000 / quality
        } else {
            200 - 2 * quality
        };

        for v in &mut quantization_table {
            *v = ((u32::from(*v) * scale + 50) / 100).clamp(1, u32::from(u8::MAX)) as u8
        }

        Self(quantization_table)
    }

    pub fn new_luma(quality: u8) -> Self {
        Self::new(quality, LUMA_QUANTIZATION_TABLE)
    }

    pub fn new_chroma(quality: u8) -> Self {
        Self::new(quality, CHROMA_QUANTIZATION_TABLE)
    }
}

impl Index<usize> for QuantizationTableValues {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Debug for QuantizationTableValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
