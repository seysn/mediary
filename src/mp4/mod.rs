use std::io::Read;

use boxes::Ftyp;
use byteorder::{BigEndian, ByteOrder};
use error::{Mp4Error, Mp4Result};

mod boxes;
pub mod error;

#[derive(Debug)]
pub struct Mp4 {
    ftyp: Ftyp,
}

#[derive(Debug)]
pub enum Mp4BoxType {
    /// File Type Box
    Ftyp,
}

#[derive(Debug)]
struct BoxHeader {
    size: u32,
    ty: Mp4BoxType,
}

pub(crate) trait Mp4Box: Sized {
    const TYPE: Mp4BoxType;

    fn size(&self) -> usize;

    fn read<R: Read>(reader: &mut R, size: u32) -> Mp4Result<Self>;
}

impl Mp4 {
    pub fn read<R: Read>(reader: &mut R) -> Mp4Result<Self> {
        let header = BoxHeader::read(reader)?;

        Ok(Mp4 {
            ftyp: Ftyp::read(reader, header.size - 8)?,
        })
    }
}

impl TryFrom<&[u8]> for Mp4BoxType {
    type Error = Mp4Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ftyp" => Ok(Self::Ftyp),
            _ => Err(Mp4Error::InvalidHeader),
        }
    }
}

impl BoxHeader {
    fn read<R: Read>(reader: &mut R) -> Mp4Result<Self> {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;

        Ok(Self {
            size: BigEndian::read_u32(&buf[..4]),
            ty: Mp4BoxType::try_from(&buf[4..])?,
        })
    }
}
