use std::io::{Read, Seek};

use mediary_ebml::{
    element::{EbmlId, MasterElement},
    error::EbmlError,
};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug, Default)]
pub struct MkvSeekHead(pub Vec<MkvSeek>);

#[derive(Debug)]
pub struct MkvSeek {
    pub id: MkvElement,
    pub position: usize,
}

impl MkvSeekHead {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut seeks = Vec::new();

        for child in element.children() {
            let child = child?;

            if let MkvElement::Seek = child.as_inner() {
                seeks.push(MkvSeek::read(child.try_into()?)?);
            }
        }

        Ok(Self(seeks))
    }
}

impl MkvSeek {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut id: Option<MkvElement> = None;
        let mut position: Option<usize> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::SeekId => {
                    let value: Vec<u8> = child.try_into()?;
                    id = Some(MkvElement::from(EbmlId::try_from(value.as_slice())?));
                }
                MkvElement::SeekPosition => {
                    let value: u64 = child.try_into()?;
                    position = Some(value as usize);
                }
                _ => (),
            }
        }

        Ok(Self {
            id: id.ok_or(MkvError::Ebml(EbmlError::MissingElement("SeekId")))?,
            position: position.ok_or(MkvError::Ebml(EbmlError::MissingElement("SeekPosition")))?,
        })
    }
}
