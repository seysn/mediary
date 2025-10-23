mod app0;
mod app1;
mod app2;
mod comment;
mod dht;
mod dqt;
mod sof;
mod sos;

use std::io::{BufRead, Seek, SeekFrom, Write};

pub use app0::Jfif;
pub use app1::{App1, XmpData};
pub use app2::App2;
pub use comment::Comment;
pub use dht::{DefineHuffmanTable, TableClass};
pub use dqt::{DefineQuantizationTable, QuantizationTable, QuantizationTableValues};
pub use sof::{ComponentId, StartOfFrame};
pub use sos::StartOfScan;

use crate::{error::JpegError, reader::read_u16, JpegResult};

#[derive(Debug, Clone, Copy)]
pub enum MarkerId {
    /// Start of Frame
    SOF(u8),

    /// Define Huffman Table
    DHT,

    /// JPEG Extensions
    JPG,

    /// Define Arithmetic Coding
    DAC,

    /// Restart Marker
    RST(u8),

    /// Define Quantization Table
    DQT,

    /// Start of Image
    SOI,

    /// End of Image
    EOI,

    /// Start of Scan
    SOS,

    /// Application Segment
    APP(u8),

    /// Comment
    COM,
}

#[derive(Debug)]
pub enum Marker {
    SOI,
    EOI,
    SOF(StartOfFrame),
    DHT(DefineHuffmanTable),
    DQT(DefineQuantizationTable),
    APP0(Jfif),
    APP1(App1),
    APP2(App2),
    SOS(StartOfScan),
    COM(Comment),
    IGN(MarkerId),
}

impl TryFrom<u8> for MarkerId {
    type Error = JpegError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xc4 => Ok(Self::DHT),
            0xc8 => Ok(Self::JPG),
            0xcc => Ok(Self::DAC),
            0xc0..=0xcf => Ok(Self::SOF(value - 0xc0)),
            0xd0..=0xd7 => Ok(Self::RST(value - 0xd0)),
            0xdb => Ok(Self::DQT),
            0xd8 => Ok(Self::SOI),
            0xd9 => Ok(Self::EOI),
            0xda => Ok(Self::SOS),
            0xe0..=0xef => Ok(Self::APP(value - 0xe0)),
            0xfe => Ok(Self::COM),
            _ => Err(JpegError::InvalidValue {
                element: "Marker",
                value: Box::new(value),
            }),
        }
    }
}

impl From<MarkerId> for u8 {
    fn from(value: MarkerId) -> Self {
        match value {
            MarkerId::DHT => 0xc4,
            MarkerId::JPG => 0xc8,
            MarkerId::DAC => 0xcc,
            MarkerId::SOF(v) => 0xc0 + v,
            MarkerId::RST(v) => 0xd0 + v,
            MarkerId::DQT => 0xdb,
            MarkerId::SOI => 0xd8,
            MarkerId::EOI => 0xd9,
            MarkerId::SOS => 0xda,
            MarkerId::APP(v) => 0xe0 + v,
            MarkerId::COM => 0xfe,
        }
    }
}

impl MarkerId {
    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        writer.write_all(&[0xff, u8::from(*self)])?;

        Ok(())
    }
}

impl Marker {
    pub fn from_reader<R: BufRead + Seek>(id: MarkerId, reader: &mut R) -> JpegResult<Self> {
        Ok(match id {
            MarkerId::SOI => Self::SOI,
            MarkerId::EOI => Self::EOI,
            MarkerId::SOF(0) => Self::SOF(StartOfFrame::from_reader(reader)?),
            MarkerId::DQT => Self::DQT(DefineQuantizationTable::from_reader(reader)?),
            MarkerId::DHT => Self::DHT(DefineHuffmanTable::from_reader(reader)?),
            MarkerId::APP(0) => Self::APP0(Jfif::from_reader(reader)?),
            MarkerId::APP(1) => Self::APP1(App1::from_reader(reader)?),
            MarkerId::APP(2) => Self::APP2(App2::from_reader(reader)?),
            MarkerId::SOS => match StartOfScan::from_reader(reader) {
                Ok(sos) => {
                    // We do not have the exact size of image data but SOS is usually
                    // followed by a EOI marker which is 2 bytes long
                    if let Err(err) = reader.seek(SeekFrom::End(-2)) {
                        return Err(err.into());
                    }

                    Self::SOS(sos)
                }
                Err(err) => return Err(err),
            },
            MarkerId::COM => Self::COM(Comment::from_reader(reader)?),
            _ => {
                let length = read_u16(reader)?;

                let rest_size = length - 2;
                if rest_size > 0
                    && let Err(err) = reader.seek(SeekFrom::Current(rest_size as i64))
                {
                    return Err(err.into());
                }

                Marker::IGN(id)
            }
        })
    }

    pub fn id(&self) -> MarkerId {
        match self {
            Marker::SOI => MarkerId::SOI,
            Marker::EOI => MarkerId::EOI,
            Marker::SOF(_) => MarkerId::SOF(0),
            Marker::DHT(_) => MarkerId::DHT,
            Marker::DQT(_) => MarkerId::DQT,
            Marker::APP0(_) => MarkerId::APP(0),
            Marker::APP1(_) => MarkerId::APP(1),
            Marker::APP2(_) => MarkerId::APP(2),
            Marker::SOS(_) => MarkerId::SOS,
            Marker::COM(_) => MarkerId::COM,
            Marker::IGN(id) => *id,
        }
    }
}
