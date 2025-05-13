use std::{
    io::{Read, Seek},
    time::Duration,
};

use ebml::{
    element::{EbmlElementValue, MasterElement},
    error::EbmlError,
};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug, Default)]
pub struct MkvInfo {
    pub timestamp_scale: u64,
    pub muxing_app: String,
    pub writing_app: String,
    pub segment_uuid: Vec<u8>,
    pub duration: Duration,
}

impl MkvInfo {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut timestamp_scale: Option<u64> = None;
        let mut muxing_app: Option<String> = None;
        let mut writing_app: Option<String> = None;
        let mut segment_uuid: Option<Vec<u8>> = None;
        let mut duration: Option<Duration> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::TimestampScale => {
                    let Some(EbmlElementValue::UnsignedInteger(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "UnsignedInteger",
                            found: child.kind().name(),
                        }));
                    };

                    timestamp_scale = Some(value);
                }
                MkvElement::MuxingApp => {
                    let Some(EbmlElementValue::String(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "String",
                            found: child.kind().name(),
                        }));
                    };

                    muxing_app = Some(value);
                }
                MkvElement::WritingApp => {
                    let Some(EbmlElementValue::String(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "String",
                            found: child.kind().name(),
                        }));
                    };

                    writing_app = Some(value);
                }
                MkvElement::SegmentUuid => {
                    let Some(EbmlElementValue::Binary(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "Binary",
                            found: child.kind().name(),
                        }));
                    };

                    segment_uuid = Some(value);
                }
                MkvElement::Duration => {
                    let Some(EbmlElementValue::Float(value)) = child.value()? else {
                        return Err(MkvError::Ebml(EbmlError::UnexpectedElement {
                            expected: "Float",
                            found: child.kind().name(),
                        }));
                    };

                    duration = Some(Duration::from_secs_f64(value));
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
