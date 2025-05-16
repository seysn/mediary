use std::io::{Read, Seek};

use mediary_ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug)]
pub struct MkvVideo {
    pub pixel_width: u64,
    pub pixel_height: u64,
}

impl MkvVideo {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut pixel_width: Option<u64> = None;
        let mut pixel_height: Option<u64> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::PixelWidth => {
                    pixel_width = Some(child.try_into()?);
                }
                MkvElement::PixelHeight => {
                    pixel_height = Some(child.try_into()?);
                }
                _ => (),
            }
        }

        Ok(Self {
            pixel_width: pixel_width
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("PixelWidth")))?,
            pixel_height: pixel_height
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("SeekId")))?,
        })
    }
}
