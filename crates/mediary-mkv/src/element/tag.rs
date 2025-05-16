use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Read, Seek},
};

use mediary_ebml::{element::MasterElement, error::EbmlError};

use crate::error::{MkvError, MkvResult};

use super::MkvElement;

#[derive(Debug, Default)]
pub struct MkvTags(pub HashMap<MkvTarget, HashMap<String, TagValue>>);

#[derive(Debug)]
pub struct MkvTag {
    pub targets: MkvTargets,
    pub simple_tags: HashMap<String, TagValue>,
}

#[derive(Debug, Default)]
pub struct MkvTargets(pub Vec<MkvTarget>);

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub enum MkvTarget {
    #[default]
    Global,
    Track(u64),
}

#[derive(Debug)]
pub struct MkvSimpleTag {
    pub key: String,
    pub value: TagValue,
}

#[derive(Debug, Clone)]
pub enum TagValue {
    String(String),
    Binary(Vec<u8>),
}

impl MkvTags {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut tags = HashMap::new();

        for child in element.children() {
            let child = child?;

            if let MkvElement::Tag = child.as_inner() {
                let tag = MkvTag::read(child.try_into()?)?;
                if tag.targets.0.is_empty() {
                    tags.insert(MkvTarget::Global, tag.simple_tags);
                } else {
                    for target in tag.targets.0 {
                        match tags.entry(target) {
                            Entry::Occupied(mut entry) => {
                                entry.get_mut().extend(tag.simple_tags.clone());
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(tag.simple_tags.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(Self(tags))
    }
}

impl MkvTag {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut targets: Option<MkvTargets> = None;
        let mut simple_tags = HashMap::new();

        for child in element.children() {
            let child = child?;

            match child.as_inner() {
                MkvElement::Targets => targets = Some(MkvTargets::read(child.try_into()?)?),
                MkvElement::SimpleTag => {
                    let simple_tag = MkvSimpleTag::read(child.try_into()?)?;
                    simple_tags.insert(simple_tag.key, simple_tag.value);
                }
                _ => (),
            }
        }

        Ok(Self {
            targets: targets.unwrap_or_default(),
            simple_tags,
        })
    }
}

impl MkvTargets {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut targets = Vec::new();

        for child in element.children() {
            let child = child?;

            if let MkvElement::TagTrackUID = child.as_inner() {
                targets.push(MkvTarget::Track(child.try_into()?));
            }
        }

        Ok(Self(targets))
    }
}

impl MkvSimpleTag {
    pub fn read<R: Read + Seek>(element: MasterElement<MkvElement, R>) -> MkvResult<Self> {
        let mut name: Option<String> = None;
        let mut value: Option<TagValue> = None;

        for child in element.children() {
            let child = child?;
            match child.as_inner() {
                MkvElement::TagName => name = Some(child.try_into()?),
                MkvElement::TagString => value = Some(TagValue::String(child.try_into()?)),
                MkvElement::TagBinary => value = Some(TagValue::Binary(child.try_into()?)),
                _ => (),
            }
        }

        Ok(Self {
            key: name.ok_or(MkvError::Ebml(EbmlError::MissingElement("TagName")))?,
            value: value.ok_or(MkvError::Ebml(EbmlError::MissingElement(
                "TagString/TagBinary",
            )))?,
        })
    }
}
