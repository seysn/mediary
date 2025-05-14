use std::io::{Read, Seek};

use ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::{MkvElement, MkvVideo};

#[derive(Debug, Default)]
pub struct MkvTracks(pub Vec<MkvTrackEntry>);

#[derive(Debug)]
pub struct MkvTrackEntry {
    pub track_number: u64,
    pub track_uid: u64,
    pub track_type: TrackType,
    pub flag_lacing: bool,
    pub language: String,
    pub codec_id: String,
    pub video: Option<MkvVideo>,
}

#[derive(Debug)]
pub enum TrackType {
    Video = 1,
    Audio = 2,
    Complex = 3,
    Logo = 16,
    Subtitle = 17,
    Buttons = 18,
    Control = 32,
    Metadata = 33,
}

impl MkvTracks {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut tracks = Vec::new();

        for child in element.children() {
            let child = child?;

            if let MkvElement::TrackEntry = child.as_inner() {
                tracks.push(MkvTrackEntry::read(child.try_into()?)?);
            }
        }

        Ok(Self(tracks))
    }
}

impl MkvTrackEntry {
    fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut track_number: Option<u64> = None;
        let mut track_uid: Option<u64> = None;
        let mut track_type: Option<TrackType> = None;
        let mut flag_lacing: bool = true;
        let mut language = "eng".to_owned();
        let mut codec_id: Option<String> = None;
        let mut video: Option<MkvVideo> = None;

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::TrackNumber => {
                    track_number = Some(child.try_into()?);
                }
                MkvElement::TrackUID => {
                    track_uid = Some(child.try_into()?);
                }
                MkvElement::TrackType => {
                    let value: u64 = child.try_into()?;
                    track_type = Some(TrackType::try_from(value)?);
                }
                MkvElement::FlagLacing => {
                    let value: u64 = child.try_into()?;
                    flag_lacing = match value {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(MkvError::InvalidValue {
                                element: "FlagLacing",
                                value: Box::new(value),
                            });
                        }
                    };
                }
                MkvElement::Language => {
                    language = child.try_into()?;
                }
                MkvElement::CodecID => {
                    codec_id = Some(child.try_into()?);
                }
                MkvElement::Video => {
                    video = Some(MkvVideo::read(child.try_into()?)?);
                }
                _ => (),
            }
        }

        Ok(Self {
            track_number: track_number
                .ok_or(MkvError::Ebml(EbmlError::MissingElement("TrackNumber")))?,
            track_uid: track_uid.ok_or(MkvError::Ebml(EbmlError::MissingElement("TrackUid")))?,
            track_type: track_type.ok_or(MkvError::Ebml(EbmlError::MissingElement("TrackType")))?,
            flag_lacing,
            language,
            codec_id: codec_id.ok_or(MkvError::Ebml(EbmlError::MissingElement("CodecId")))?,
            video,
        })
    }
}

impl TryFrom<u64> for TrackType {
    type Error = MkvError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Video,
            2 => Self::Audio,
            3 => Self::Complex,
            16 => Self::Logo,
            17 => Self::Subtitle,
            18 => Self::Buttons,
            32 => Self::Control,
            33 => Self::Metadata,
            _ => {
                return Err(MkvError::InvalidValue {
                    element: "TrackType",
                    value: Box::new(value),
                })
            }
        })
    }
}
