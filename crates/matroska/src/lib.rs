use std::io::{Read, Seek};

use ebml::{
    element::EbmlElement,
    reader::{EbmlHeader, EbmlIterator},
    EbmlDocument,
};
use element::{MkvElement, MkvSeekHead};

pub mod element;
pub mod error;

pub type MkvIterator<R> = EbmlIterator<MkvElement, R>;

pub struct Matroska<R: Read + Seek> {
    reader: MatroskaReader<R>,

    seek_head: MkvSeekHead,
}

pub struct MatroskaReader<R: Read + Seek> {
    ebml_document: EbmlDocument<MkvElement, R>,
}

impl<R: Read + Seek> Matroska<R> {
    pub fn read(reader: R) -> error::MkvResult<Self> {
        let reader = MatroskaReader::new(reader)?;
        let Some(segment) = reader.iter().next() else {
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
        for element in segment.children() {
            let element = element?;

            let EbmlElement::Master(element) = element else {
                continue;
            };

            match element.element {
                MkvElement::SeekHead => seek_head = Some(MkvSeekHead::read(element)?),
                MkvElement::Info => todo!(),
                MkvElement::Tracks => todo!(),
                _ => (),
            }
        }

        Ok(Self {
            reader,
            seek_head: seek_head.unwrap_or_default(),
        })
    }
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub fn new(reader: R) -> error::MkvResult<Self> {
        Ok(Self {
            ebml_document: EbmlDocument::new(reader)?,
        })
    }

    pub fn ebml_header(&self) -> &EbmlHeader {
        &self.ebml_document.header
    }

    pub fn iter(&self) -> MkvIterator<R> {
        self.ebml_document.iter()
    }
}
