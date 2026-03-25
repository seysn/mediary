use std::io::{BufRead, Seek, Write};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::{JpegError, JpegResult},
    reader::{read_u8, read_u16},
};

#[derive(Debug, Clone)]
pub struct StartOfFrame {
    pub precision: u8,
    pub width: u16,
    pub height: u16,
    pub components: Vec<SofComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentId {
    Y,
    Cb,
    Cr,
}

#[derive(Debug, Clone)]
pub struct SofComponent {
    pub id: ComponentId,
    pub horizontal_sampling: u8,
    pub vertical_sampling: u8,
    pub quantization_table: u8,
}

impl StartOfFrame {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let _length = read_u16(reader)?;
        let precision = read_u8(reader)?;
        let height = read_u16(reader)?;
        let width = read_u16(reader)?;
        let n_components = read_u8(reader)?;

        let mut components = Vec::new();
        for i in 0..n_components {
            let id = i.try_into()?;
            let mut buf = [0; 3];
            reader.read_exact(&mut buf)?;

            let horizontal_sampling = (buf[1] >> 4) & 0x0f;
            let vertical_sampling = buf[1] & 0x0f;
            let quantization_table = buf[2];

            components.push(SofComponent {
                id,
                horizontal_sampling,
                vertical_sampling,
                quantization_table,
            })
        }

        Ok(Self {
            precision,
            width,
            height,
            components,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        let length = 8 + (3 * self.components.len());
        let mut buf = vec![0; length];
        BigEndian::write_u16(&mut buf[0..2], length as u16);
        buf[2] = self.precision;
        BigEndian::write_u16(&mut buf[3..5], self.height);
        BigEndian::write_u16(&mut buf[5..7], self.width);
        buf[7] = self.components.len() as u8;

        for (chunk, component) in buf[8..].chunks_exact_mut(3).zip(&self.components) {
            chunk[0] = u8::from(component.id) + 1;
            chunk[1] =
                ((component.horizontal_sampling & 0xf) << 4) + (component.vertical_sampling & 0xf);
            chunk[2] = component.quantization_table;
        }

        writer.write_all(&buf)?;

        Ok(())
    }
}

impl TryFrom<u8> for ComponentId {
    type Error = JpegError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Y,
            1 => Self::Cb,
            2 => Self::Cr,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "ComponentId",
                    value: Box::new(value),
                });
            }
        })
    }
}

impl From<ComponentId> for u8 {
    fn from(value: ComponentId) -> Self {
        match value {
            ComponentId::Y => 0,
            ComponentId::Cb => 1,
            ComponentId::Cr => 2,
        }
    }
}
