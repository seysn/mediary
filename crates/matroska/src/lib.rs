use std::{
    fmt::Debug,
    io::{Read, Seek},
};

use ebml::{element::EbmlElement, error::EbmlResult, reader::EbmlHeader, EbmlReader};
use element::{MkvElement, MkvInfo, MkvSeekHead};

pub mod element;
pub mod error;

pub struct Matroska<R: Read + Seek> {
    reader: MatroskaReader<R>,
    pub seek_head: MkvSeekHead,
    pub info: MkvInfo,
}

pub struct MatroskaReader<R: Read + Seek> {
    ebml_reader: EbmlReader<MkvElement, R>,
    pub ebml_header: EbmlHeader,
}

impl<R: Read + Seek> Matroska<R> {
    pub fn read(reader: R) -> error::MkvResult<Self> {
        let mut reader = MatroskaReader::read(reader)?;
        let Some(segment) = reader.next() else {
            todo!();
        };

        let segment = segment?;
        let EbmlElement::Master(segment) = segment else {
            return Err(error::MkvError::Ebml(
                ebml::error::EbmlError::UnexpectedElement {
                    expected: "Master",
                    found: segment.kind().name(),
                },
            ));
        };

        if !matches!(segment.element, MkvElement::Segment) {
            return Err(error::MkvError::Ebml(
                ebml::error::EbmlError::UnexpectedElement {
                    expected: "Segment",
                    found: segment.kind().name(),
                },
            ));
        }

        let mut seek_head: Option<MkvSeekHead> = None;
        let mut info: Option<MkvInfo> = None;
        for element in segment.children() {
            let element = element?;

            let EbmlElement::Master(element) = element else {
                continue;
            };

            match element.element {
                MkvElement::SeekHead => seek_head = Some(MkvSeekHead::read(element)?),
                MkvElement::Info => info = Some(MkvInfo::read(element)?),
                _ => (),
            }
        }

        Ok(Self {
            reader,
            seek_head: seek_head.unwrap_or_default(),
            info: info.unwrap_or_default(),
        })
    }

    pub fn ebml_header(&self) -> &EbmlHeader {
        &self.reader.ebml_header
    }
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub fn read(reader: R) -> error::MkvResult<Self> {
        let mut ebml_reader = EbmlReader::new(reader)?;
        let ebml_header = ebml_reader.read_ebml_header()?;

        Ok(Self {
            ebml_reader,
            ebml_header,
        })
    }
}

impl<R: Read + Seek> Iterator for MatroskaReader<R> {
    type Item = EbmlResult<EbmlElement<MkvElement, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ebml_reader.next()
    }
}

impl<R: Read + Seek> Debug for Matroska<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matroska")
            .field("ebml_header", &self.reader.ebml_header)
            .field("seek_head", &self.seek_head)
            .field("info", &self.info)
            .finish()
    }
}
