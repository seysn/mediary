use std::io::Read;

use byteorder::{BigEndian, ByteOrder};

use crate::{error::Mp4Result, FourCC, Mp4Box, Mp4BoxType};

/// File Type Box
#[derive(Debug)]
pub(crate) struct Ftyp {
    major_brand: FourCC,
    minor_version: u32,
    compatible_brands: Vec<FourCC>,
}

impl Mp4Box for Ftyp {
    const TYPE: Mp4BoxType = Mp4BoxType::Ftyp;

    fn size(&self) -> usize {
        8 + 4 * self.compatible_brands.len()
    }

    fn read<R: Read>(reader: &mut R, size: u32) -> Mp4Result<Self> {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;

        let major_brand = FourCC::from(&buf[..4]);
        let minor_version = BigEndian::read_u32(&buf[4..8]);

        let compatible_brands = if size > 8 {
            let mut buf = vec![0; size as usize - 8];
            reader.read_exact(&mut buf)?;

            buf.chunks_exact(4).map(FourCC::from).collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            major_brand,
            minor_version,
            compatible_brands,
        })
    }
}
