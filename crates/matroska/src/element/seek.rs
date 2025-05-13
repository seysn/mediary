use std::io::{Read, Seek};

use ebml::{
    element::{EbmlElement, EbmlElementValue, EbmlId, MasterElement},
    error::EbmlError,
};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug, Default)]
pub struct MkvSeekHead {
    pub seeks: Vec<MkvSeek>,
}

#[derive(Debug)]
pub struct MkvSeek(pub MkvElement, pub usize);
impl MkvSeekHead {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut seeks = Vec::new();

        for child in element.children() {
            let child = child?;

            if let MkvElement::Seek = child.as_inner() {
                if let EbmlElement::Master(child) = child {
                    seeks.push(MkvSeek::read(child)?);
                };
            }
        }

        Ok(Self { seeks })
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
                    let Some(EbmlElementValue::Binary(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "Binary",
                            found: child.kind().name(),
                        }));
                    };

                    id = Some(MkvElement::from(EbmlId::try_from(value.as_slice())?));
                }
                MkvElement::SeekPosition => {
                    let Some(EbmlElementValue::UnsignedInteger(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "UnsignedInteger",
                            found: child.kind().name(),
                        }));
                    };

                    position = Some(value as usize);
                }
                _ => (),
            }
        }

        Ok(Self(
            id.ok_or(MkvError::Ebml(EbmlError::MissingElement("SeekId")))?,
            position.ok_or(MkvError::Ebml(EbmlError::MissingElement("SeekPosition")))?,
        ))
    }
}
