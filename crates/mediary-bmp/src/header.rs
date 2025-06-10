use std::io::{Cursor, Read};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

use crate::error::{BmpError, BmpResult};

#[derive(Debug)]
pub enum BitmapSignature {
    /// Windows 3.1x, 95, NT, ... etc.
    BM = 0x424d,

    /// OS/2 struct bitmap array
    BA = 0x4241,

    /// OS/2 struct color icon
    CI = 0x4349,

    /// OS/2 const color pointer
    CP = 0x4350,

    /// OS/2 struct icon
    IC = 0x4943,

    /// OS/2 pointer
    PT = 0x5054,
}

#[derive(Debug)]
pub struct BMPHeader {
    pub signature: BitmapSignature,
    pub size: u32,
    pub data_offset: u32,
}

#[derive(Debug)]
pub struct DIBHeader {
    pub dib_size: u32,
    pub width: u32,
    pub height: u32,
    pub planes: u16,
    pub bits_per_pixel: u16,
    pub compression: u32,
    pub data_size: u32,
    pub resolution_horizontal: u32,
    pub resolution_vertical: u32,
    pub colors: u32,
    pub important_colors: u32,
}

impl BMPHeader {
    pub fn from_reader<R: Read>(reader: &mut R) -> BmpResult<Self> {
        let signature = reader.read_u16::<BigEndian>()?.try_into()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let _unused = reader.read_u32::<LittleEndian>()?;
        let data_offset = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            signature,
            size,
            data_offset,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> BmpResult<Self> {
        Self::from_reader(&mut Cursor::new(bytes))
    }
}

impl DIBHeader {
    pub fn from_reader<R: Read>(reader: &mut R) -> BmpResult<Self> {
        let dib_size = reader.read_u32::<LittleEndian>()?;
        let width = reader.read_u32::<LittleEndian>()?;
        let height = reader.read_u32::<LittleEndian>()?;
        let planes = reader.read_u16::<LittleEndian>()?;
        let bits_per_pixel = reader.read_u16::<LittleEndian>()?;
        let compression = reader.read_u32::<LittleEndian>()?;
        let data_size = reader.read_u32::<LittleEndian>()?;
        let resolution_horizontal = reader.read_u32::<LittleEndian>()?;
        let resolution_vertical = reader.read_u32::<LittleEndian>()?;
        let colors = reader.read_u32::<LittleEndian>()?;
        let important_colors = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            dib_size,
            width,
            height,
            planes,
            bits_per_pixel,
            compression,
            data_size,
            resolution_horizontal,
            resolution_vertical,
            colors,
            important_colors,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> BmpResult<Self> {
        Self::from_reader(&mut Cursor::new(bytes))
    }
}

impl TryFrom<u16> for BitmapSignature {
    type Error = BmpError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x424d => Ok(Self::BM),
            0x4241 => Ok(Self::BA),
            0x4349 => Ok(Self::CI),
            0x4350 => Ok(Self::CP),
            0x4943 => Ok(Self::IC),
            0x5054 => Ok(Self::PT),
            _ => Err(BmpError::InvalidValue {
                element: "BitmapSignature",
                value: Box::new(value),
            }),
        }
    }
}
