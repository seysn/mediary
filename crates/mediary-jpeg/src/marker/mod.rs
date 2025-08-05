mod app0;
mod app1;
mod app2;
mod comment;
mod dht;
mod dqt;
mod sof;
mod sos;

pub use app0::Jfif;
pub use app1::{App1, XmpData};
pub use app2::App2;
pub use comment::Comment;
pub use dht::HuffmanTable;
pub use dqt::{DefineQuantizationTable, QuantizationTable, QuantizationTableValues};
pub use sof::StartOfFrame;
pub use sos::StartOfScan;

use crate::error::JpegError;

#[derive(Debug)]
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
    SOF(StartOfFrame),
    DHT(HuffmanTable),
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
