use std::io::{BufRead, Seek, SeekFrom, Write};

use crate::{
    error::{JpegError, JpegResult},
    reader::{read_u16, read_u8},
};

#[derive(Debug, Clone)]
pub struct Jfif {
    pub version: u16,
    pub density_unit: u8,
    pub x_density: u16,
    pub y_density: u16,
    pub x_thumbnail: u8,
    pub y_thumbnail: u8,
}

impl Jfif {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let _length = read_u16(reader)?;
        let mut identifier = [0; 5];
        reader.read_exact(&mut identifier)?;
        if &identifier != b"JFIF\0" {
            return Err(JpegError::InvalidValue {
                element: "App0 identifier",
                value: Box::new(identifier),
            });
        }

        let version = read_u16(reader)?;
        let density_unit = read_u8(reader)?;
        let x_density = read_u16(reader)?;
        let y_density = read_u16(reader)?;
        let x_thumbnail = read_u8(reader)?;
        let y_thumbnail = read_u8(reader)?;

        // Skipping thumbnail for now
        let thumbnail_size = x_thumbnail as i64 * y_thumbnail as i64;
        if thumbnail_size > 0 {
            reader.seek(SeekFrom::Current(thumbnail_size))?;
        }

        Ok(Self {
            version,
            density_unit,
            x_density,
            y_density,
            x_thumbnail,
            y_thumbnail,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        const LENGTH: u8 = 16;

        let version_hi = (self.version >> 8) as u8;
        let version_lo = (self.version & 0xff) as u8;

        let x_density_hi = (self.x_density >> 8) as u8;
        let x_density_lo = (self.x_density & 0xff) as u8;

        let y_density_hi = (self.y_density >> 8) as u8;
        let y_density_lo = (self.y_density & 0xff) as u8;

        let data: [u8; LENGTH as usize] = [
            0,
            LENGTH,
            b'J',
            b'F',
            b'I',
            b'F',
            0,
            version_hi,
            version_lo,
            self.density_unit,
            x_density_hi,
            x_density_lo,
            y_density_hi,
            y_density_lo,
            self.x_thumbnail,
            self.y_thumbnail,
        ];

        writer.write_all(&data)?;
        Ok(())
    }
}
