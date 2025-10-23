use std::io::{BufRead, Seek, Write};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::JpegResult,
    reader::{read_u16, read_u8},
};

use super::ComponentId;

#[derive(Debug, Clone)]
pub struct StartOfScan {
    pub components: Vec<SosComponent>,
    pub start_spectral: u8,
    pub end_spectral: u8,
    pub approximation_bit: u8,
    pub data: ImageData,
}

#[derive(Debug, Clone)]
pub struct SosComponent {
    pub id: ComponentId,
    pub dc_table: u8,
    pub ac_table: u8,
}

#[derive(Clone)]
pub struct ImageData(pub Vec<u8>);

impl StartOfScan {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let _header_length = read_u16(reader)?;
        let n_components = read_u8(reader)?;

        let mut components = Vec::new();
        for i in 0..n_components {
            let id = i.try_into()?;

            let mut buf = [0; 2];
            reader.read_exact(&mut buf)?;

            let dc_table = (buf[1] >> 4) & 0xf;
            let ac_table = buf[1] & 0xf;

            components.push(SosComponent {
                id,
                dc_table,
                ac_table,
            });
        }

        let start_spectral = read_u8(reader)?;
        let end_spectral = read_u8(reader)?;
        let approximation_bit = read_u8(reader)?;

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(Self {
            components,
            start_spectral,
            end_spectral,
            approximation_bit,
            data: ImageData(data),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        let header_length = 6 + (2 * self.components.len());
        let mut buf = vec![0; header_length];

        BigEndian::write_u16(&mut buf[0..2], header_length as u16);
        buf[2] = self.components.len() as u8;
        for (chunk, component) in buf[3..].chunks_exact_mut(2).zip(&self.components) {
            chunk[0] = u8::from(component.id) + 1;
            chunk[1] = ((component.dc_table & 0xf) << 4) + (component.ac_table & 0xf);
        }

        writer.write_all(&buf)?;
        writer.write_all(&self.data.0)?;

        Ok(())
    }
}

impl std::fmt::Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ {} bytes ]", self.0.len())
    }
}
