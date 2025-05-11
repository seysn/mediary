use std::io::{Read, Seek};

use ebml::{
    reader::{EbmlHeader, EbmlIterator},
    EbmlDocument,
};
use element::MkvElement;

pub mod element;
pub mod error;

pub struct MatroskaReader<R: Read + Seek> {
    ebml_document: EbmlDocument<MkvElement, R>,
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

    pub fn iter(&self) -> EbmlIterator<MkvElement, R> {
        self.ebml_document.iter()
    }
}
