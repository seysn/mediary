use std::io::{Read, Seek};

use ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug, Default)]
pub struct MkvInfo {
    pub timestamp_scale: u64,
    pub muxing_app: String,
    pub writing_app: String,
    pub segment_uuid: Vec<u8>,
    pub duration: f64,
}

impl MkvInfo {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut timestamp_scale: Option<u64> = None;
        let mut muxing_app: Option<String> = None;
        let mut writing_app: Option<String> = None;
        let mut segment_uuid: Option<Vec<u8>> = None;
        let mut duration: Option<f64> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::TimestampScale => {
                    timestamp_scale = Some(child.try_into()?);
                }
                MkvElement::MuxingApp => {
                    muxing_app = Some(child.try_into()?);
                }
                MkvElement::WritingApp => {
                    writing_app = Some(child.try_into()?);
                }
                MkvElement::SegmentUuid => {
                    segment_uuid = Some(child.try_into()?);
                }
                MkvElement::Duration => {
                    duration = Some(child.try_into()?);
                }
                _ => (),
            }
        }

        Ok(Self {
            timestamp_scale: timestamp_scale
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("TimestampScale")))?,
            muxing_app: muxing_app.ok_or(MkvError::Ebml(EbmlError::MissingElement("MuxingApp")))?,
            writing_app: writing_app
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("WritingApp")))?,
            segment_uuid: segment_uuid
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("SegmentUuid")))?,
            duration: duration.ok_or(MkvError::Ebml(EbmlError::MissingElement("Duration")))?,
        })
    }
}
