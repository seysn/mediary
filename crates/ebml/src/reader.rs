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

pub struct EbmlReader<S: EbmlSpec, R: Read + Seek> {
    reader: SharedReader<R>,
    start: u64,
    end: u64,
    _spec: PhantomData<S>,
}

impl<S: EbmlSpec, R: Read + Seek> EbmlReader<S, R> {
    pub fn new(mut reader: R) -> EbmlResult<Self> {
        let end = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        Ok(Self {
            start: 0,
            end,
            reader: Rc::new(RefCell::new(reader)),
            _spec: PhantomData,
        })
    }

    pub fn from_range(reader: SharedReader<R>, start: u64, end: u64) -> Self {
        Self {
            reader,
            start,
            end,
            _spec: PhantomData,
        }
    }

    pub fn read_ebml_header(&mut self) -> EbmlResult<EbmlHeader> {
        let mut document: EbmlReader<EbmlHeaderElement, R> = EbmlReader {
            reader: self.reader.clone(),
            start: 0,
            end: self.end,
            _spec: PhantomData,
        };

        let Some(element) = document.next() else {
            return Err(EbmlError::Io(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        };

        let header_element: MasterElement<EbmlHeaderElement, R> = element?.try_into()?;
        if !matches!(header_element.element, EbmlHeaderElement::Ebml) {
            return Err(EbmlError::UnexpectedElement {
                expected: "Ebml",
                found: header_element.name(),
            });
        }

        let mut header = EbmlHeader::default();
        for element in header_element.children() {
            let element = element?;

            match element.as_inner() {
                EbmlHeaderElement::Ebml => todo!(),
                EbmlHeaderElement::EbmlVersion => header.ebml_version = element.try_into()?,
                EbmlHeaderElement::EbmlReadVersion => {
                    header.ebml_read_version = element.try_into()?
                }
                EbmlHeaderElement::EbmlMaxIDLength => header.max_id_length = element.try_into()?,
                EbmlHeaderElement::EbmlMaxSizeLength => {
                    header.max_size_length = element.try_into()?
                }
                EbmlHeaderElement::DocType => header.doc_type = element.try_into()?,
                EbmlHeaderElement::DocTypeVersion => {
                    header.doc_type_version = element.try_into()?
                }
                EbmlHeaderElement::DocTypeReadVersion => {
                    header.doc_type_read_version = element.try_into()?
                }
                EbmlHeaderElement::DocTypeExtension => todo!(),
                EbmlHeaderElement::DocTypeExtensionName => todo!(),
                EbmlHeaderElement::DocTypeExtensionVersion => todo!(),
                EbmlHeaderElement::Crc32 => todo!(),
                EbmlHeaderElement::Void => todo!(),
                EbmlHeaderElement::Unknown(_) => todo!(),
            }
        }

        self.start = document.start;
        Ok(header)
    }
}

impl<S: EbmlSpec, R: Read + Seek> Iterator for EbmlReader<S, R> {
    type Item = EbmlResult<EbmlElement<S, R>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let mut position = self.start;
        let mut reader = self.reader.borrow_mut();
        if let Err(err) = reader.seek(SeekFrom::Start(position)) {
            return Some(Err(EbmlError::Io(err)));
        }

        let id = match Vint::from_reader(&mut *reader) {
            Ok(vint) => {
                position += vint.length as u64;
                EbmlId::from(vint)
            }
            Err(err) => return Some(Err(err)),
        };

        let size = match Vint::from_reader(&mut *reader) {
            Ok(size) => {
                position += size.length as u64;
                size
            }
            Err(err) => return Some(Err(err)),
        };

        let data_offset = position - self.start;
        position += size.value;

        let s = S::from(id);
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
                    position: self.start,
                    data,
                    _spec: PhantomData,
                })
            }
            EbmlElementType::Binary => {
                if size.value <= 20 {
                    let mut data = vec![0; size.value as usize];
                    if let Err(err) = reader.read_exact(&mut data) {
                        return Some(Err(EbmlError::Io(err)));
                    }

                    EbmlElement::Value(ValueElement {
                        element: s,
                        position: self.start,
                        data,
                        _spec: PhantomData,
                    })
                } else {
                    EbmlElement::LazyValue(LazyValueElement {
                        element: s,
                        position: self.start,
                        data_offset,
                        size: size.value,
                        reader: self.reader.clone(),
                        _spec: PhantomData::<S>,
                    })
                }
            }
            EbmlElementType::Master => {
                let elem = EbmlElement::Master(MasterElement {
                    element: s,
                    position: self.start,
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

        self.start = position;
        Some(Ok(elem))
    }
}
