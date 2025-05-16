use std::io::{Read, Seek};

use mediary_ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug)]
pub struct MkvCluster {
    pub position: u64,
    pub timestamp: u64,
    pub blocks: u64,
}

impl MkvCluster {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut blocks = 0;
        let mut timestamp: Option<u64> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::Timestamp => timestamp = Some(child.try_into()?),
                MkvElement::SimpleBlock => blocks += 1,
                _ => (),
            }
        }

        Ok(Self {
            position: element.position,
            timestamp: timestamp.ok_or(MkvError::Ebml(EbmlError::MissingElement("Timestamp")))?,
            blocks,
        })
    }
}
