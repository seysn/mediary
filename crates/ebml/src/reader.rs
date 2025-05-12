use std::{
    cell::RefCell,
    io::{ErrorKind, Read, Seek, SeekFrom},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    element::{
        EbmlElement, EbmlElementType, EbmlHeaderElement, EbmlId, EbmlSpec, LazyValueElement,
        MasterElement, ValueElement,
    },
    error::{EbmlError, EbmlResult},
    vint::Vint,
};

pub(crate) type SharedReader<R> = Rc<RefCell<R>>;

#[derive(Debug, Default)]
pub struct EbmlHeader {
    pub ebml_version: u64,
    pub ebml_read_version: u64,
    pub max_id_length: u64,
    pub max_size_length: u64,
    pub doc_type: String,
    pub doc_type_version: u64,
    pub doc_type_read_version: u64,
}

pub struct EbmlDocument<S: EbmlSpec, R: Read + Seek> {
    pub header: EbmlHeader,
    reader: SharedReader<R>,
    start_data: u64,
    end_data: u64,
    _spec: PhantomData<S>,
}

pub struct EbmlIterator<S: EbmlSpec, R: Read + Seek> {
    reader: SharedReader<R>,
    start: u64,
    end: u64,
    _spec: PhantomData<S>,
}

impl<S: EbmlSpec, R: Read + Seek> EbmlDocument<S, R> {
    pub fn new(reader: R) -> EbmlResult<Self> {
        let reader = Rc::new(RefCell::new(reader));

        let end_data = {
            let mut reader = reader.borrow_mut();
            let start = reader.stream_position()?;
            let end_data = reader.seek(SeekFrom::End(0))?;
            reader.seek(SeekFrom::Start(start))?;
            end_data
        };

        let mut iter = EbmlIterator::<EbmlHeaderElement, R>::new(reader.clone(), 0, end_data);

        let Some(element) = iter.next() else {
            return Err(EbmlError::Io(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        };

        let element = element?;
        let EbmlElement::Master(header_element) = element else {
            return Err(EbmlError::UnexpectedElementType {
                expected: "Master",
                found: element.kind().name(),
            });
        };

        if !matches!(header_element.element, EbmlHeaderElement::Ebml) {
            return Err(EbmlError::UnexpectedElementType {
                expected: "Ebml",
                found: header_element.name(),
            });
        }

        let mut header = EbmlHeader::default();
        for element in header_element.children() {
            let element = element?;
            let Some(value) = element.value()? else {
                continue;
            };

            match element.as_inner() {
                EbmlHeaderElement::Ebml => todo!(),
                EbmlHeaderElement::EbmlVersion => {
                    header.ebml_version =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::EbmlReadVersion => {
                    header.ebml_read_version =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::EbmlMaxIDLength => {
                    header.max_id_length =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::EbmlMaxSizeLength => {
                    header.max_size_length =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::DocType => {
                    header.doc_type = value
                        .as_str()
                        .ok_or(EbmlError::UnexpectedElementType {
                            expected: "String",
                            found: element.kind().name(),
                        })?
                        .to_owned();
                }
                EbmlHeaderElement::DocTypeVersion => {
                    header.doc_type_version =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::DocTypeReadVersion => {
                    header.doc_type_read_version =
                        value.as_u64().ok_or(EbmlError::UnexpectedElementType {
                            expected: "UnsignedInteger",
                            found: element.kind().name(),
                        })?;
                }
                EbmlHeaderElement::DocTypeExtension => todo!(),
                EbmlHeaderElement::DocTypeExtensionName => todo!(),
                EbmlHeaderElement::DocTypeExtensionVersion => todo!(),
                EbmlHeaderElement::Crc32 => todo!(),
                EbmlHeaderElement::Void => todo!(),
                EbmlHeaderElement::Unknown(_) => todo!(),
            }
        }

        let start_data = reader.borrow_mut().stream_position()?;
        Ok(Self {
            header,
            start_data,
            end_data,
            reader,
            _spec: PhantomData,
        })
    }

    pub fn iter(&self) -> EbmlIterator<S, R> {
        EbmlIterator::new(self.reader.clone(), self.start_data, self.end_data)
    }
}

impl<S: EbmlSpec, R: Read + Seek> EbmlIterator<S, R> {
    pub(crate) fn new(reader: SharedReader<R>, start: u64, end: u64) -> Self {
        Self {
            reader,
            start,
            end,
            _spec: PhantomData,
        }
    }
}

impl<S: EbmlSpec, R: Read + Seek> Iterator for EbmlIterator<S, R> {
    type Item = EbmlResult<EbmlElement<S, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let mut reader = self.reader.borrow_mut();
        if let Err(err) = reader.seek(SeekFrom::Start(self.start)) {
            return Some(Err(EbmlError::Io(err)));
        }

        let id = match EbmlId::from_reader(&mut *reader) {
            Ok(id) => id,
            Err(err) => return Some(Err(err)),
        };
        let size = match Vint::from_reader(&mut *reader) {
            Ok(size) => size,
            Err(err) => return Some(Err(err)),
        };
        let data_offset = match reader.stream_position() {
            Ok(data_offset) => data_offset,
            Err(err) => return Some(Err(EbmlError::Io(err))),
        };

        let s = S::from_id(id);
        let elem = match s.kind() {
            EbmlElementType::String
            | EbmlElementType::Utf8
            | EbmlElementType::SignedInteger
            | EbmlElementType::UnsignedInteger
            | EbmlElementType::Float
            | EbmlElementType::Date => {
                let mut data = vec![0; size.value as usize];
                if let Err(err) = reader.read_exact(&mut data) {
                    return Some(Err(EbmlError::Io(err)));
                }

                EbmlElement::Value(ValueElement {
                    element: s,
                    data,
                    _spec: PhantomData,
                })
            }
            EbmlElementType::Binary => {
                let elem = EbmlElement::LazyValue(LazyValueElement {
                    element: s,
                    data_offset,
                    size: size.value,
                    reader: self.reader.clone(),
                    _spec: PhantomData::<S>,
                });

                if let Err(err) = reader.seek(SeekFrom::Current(size.value as i64)) {
                    return Some(Err(EbmlError::Io(err)));
                }

                elem
            }
            EbmlElementType::Master => {
                let elem = EbmlElement::Master(MasterElement {
                    element: s,
                    data_offset,
                    size: size.value,
                    reader: self.reader.clone(),
                    _spec: PhantomData::<S>,
                });

                if let Err(err) = reader.seek(SeekFrom::Current(size.value as i64)) {
                    return Some(Err(EbmlError::Io(err)));
                }

                elem
            }
        };

        let pos = match reader.stream_position() {
            Ok(pos) => pos,
            Err(err) => return Some(Err(EbmlError::Io(err))),
        };

        self.start = pos;
        Some(Ok(elem))
    }
}
