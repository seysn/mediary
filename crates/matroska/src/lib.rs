use std::io::{Read, Seek};

use ebml::EbmlDocument;
use element::MkvElement;

pub mod element;

pub struct Matroska<R: Read + Seek> {
    pub ebml_document: EbmlDocument<MkvElement, R>,
}

impl<R: Read + Seek> Matroska<R> {
    pub fn read(reader: R) -> std::io::Result<Self> {
        Ok(Self {
            ebml_document: EbmlDocument::new(reader)?,
        })
    }
}
