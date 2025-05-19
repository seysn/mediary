use std::{
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
};

use mediary_ebml::{element::LazyValueElement, error::EbmlError, reader::SharedReader, vint::Vint};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

pub struct MkvSimpleBlock<R: Read + Seek> {
    pub track_number: u64,
    pub timestamp: i16,
    pub keyframe: bool,
    pub invisible: bool,
    pub lacing: Lacing,
    pub discardable: bool,
    pub data: FrameData<R>,
}

#[derive(Debug)]
pub enum Lacing {
    None,
    Xiph,
    Ebml,
    FixedSize,
}

pub struct FrameData<R: Read + Seek> {
    reader: SharedReader<R>,
    pub position: u64,
    pub size: u64,
}

impl<R: Read + Seek> MkvSimpleBlock<R> {
    pub fn read(element: LazyValueElement<MkvElement, R>) -> MkvResult<Self> {
        let mut reader = element.reader.borrow_mut();

        let track_number = Vint::from_reader(&mut *reader)?;

        let mut buf = [0; 2];
        reader.read_exact(&mut buf).map_err(EbmlError::Io)?;
        let timestamp = i16::from_be_bytes(buf);

        let mut buf = [0];
        reader.read_exact(&mut buf).map_err(EbmlError::Io)?;
        let b = buf[0];

        let keyframe = b & 0x80 > 0;
        let invisible = b & 0x08 > 0;
        let lacing = Lacing::try_from((b >> 1) & 0b11)?;
        let discardable = b & 1 > 0;

        drop(reader);

        let header_size = track_number.length as u64 + 3;
        let data = FrameData {
            reader: element.reader,
            position: element.position + header_size,
            size: element.size - header_size,
        };

        Ok(Self {
            track_number: track_number.value,
            timestamp,
            keyframe,
            invisible,
            lacing,
            discardable,
            data,
        })
    }
}

impl<R: Read + Seek> FrameData<R> {
    pub fn read(&self) -> MkvResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.size as usize);

        let mut reader = self.reader.borrow_mut();
        reader
            .seek(SeekFrom::Start(self.position))
            .map_err(EbmlError::Io)?;
        reader.read_exact(&mut buf).map_err(EbmlError::Io)?;

        Ok(buf)
    }
}

impl TryFrom<u8> for Lacing {
    type Error = MkvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Xiph,
            2 => Self::Ebml,
            3 => Self::FixedSize,
            _ => {
                return Err(MkvError::InvalidValue {
                    element: "Lacing",
                    value: Box::new(value),
                })
            }
        })
    }
}

impl<R: Read + Seek> Debug for MkvSimpleBlock<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MkvSimpleBlock")
            .field("track_number", &self.track_number)
            .field("timestamp", &self.timestamp)
            .field("keyframe", &self.keyframe)
            .field("invisible", &self.invisible)
            .field("lacing", &self.lacing)
            .field("discardable", &self.discardable)
            .finish()
    }
}
