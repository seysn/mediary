use std::{
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
};

use mediary_ebml::{element::LazyValueElement, reader::SharedReader, vint::Vint};
use mediary_h264::nal::NalUnitIterator;

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
        reader.seek(SeekFrom::Start(element.position + element.data_offset))?;

        let track_number = Vint::from_reader(&mut *reader)?;

        let mut buf = [0; 3];
        reader.read_exact(&mut buf)?;
        let timestamp =
            i16::from_be_bytes(buf[..2].try_into().expect("slice has fewer than 2 bytes"));

        let flags = buf[2];

        let keyframe = flags & 0x80 > 0;
        let invisible = flags & 0x08 > 0;
        let lacing = Lacing::try_from((flags >> 1) & 0b11)?;
        let discardable = flags & 1 > 0;

        drop(reader);

        let header_size = track_number.length as u64 + 3;
        let data = FrameData {
            reader: element.reader,
            position: element.position + element.data_offset + header_size,
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

    /// Return an iterator of NalUnit contained in block.
    /// Vector buf passed in argument is resized automatically.
    pub fn nal_units<'a>(&self, buf: &'a mut Vec<u8>) -> MkvResult<NalUnitIterator<'a>> {
        let size = self.data.size as usize;

        // Resize buffer if needed to handle all the data
        if buf.len() < size {
            buf.resize(size, 0);
        }

        let buf = &mut buf[..size];
        self.data.read(buf)?;
        Ok(NalUnitIterator::new(buf))
    }
}

impl<R: Read + Seek> FrameData<R> {
    pub fn read(&self, buf: &mut [u8]) -> MkvResult<()> {
        let mut reader = self.reader.borrow_mut();
        reader.seek(SeekFrom::Start(self.position))?;
        reader.read_exact(buf)?;

        Ok(())
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
