use std::{
    fmt::Debug,
    io::{Read, Seek},
};

use mediary_ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::{MkvElement, MkvSimpleBlock};

pub struct MkvCluster<R: Read + Seek> {
    pub timestamp: u64,
    pub blocks: Vec<MkvSimpleBlock<R>>,
}

impl<R: Read + Seek> MkvCluster<R> {
    pub fn read(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut blocks = Vec::new();
        let mut timestamp: Option<u64> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::Timestamp => timestamp = Some(child.try_into()?),
                MkvElement::SimpleBlock => blocks.push(MkvSimpleBlock::read(child.try_into()?)?),
                _ => (),
            }
        }

        Ok(Self {
            timestamp: timestamp.ok_or(MkvError::Ebml(EbmlError::MissingElement("Timestamp")))?,
            blocks,
        })
    }
}

impl<R: Read + Seek> Debug for MkvCluster<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MkvCluster")
            .field("timestamp", &self.timestamp)
            .field("blocks", &self.blocks.len())
            // .field("blocks", &self.blocks)
            .finish()
    }
}
