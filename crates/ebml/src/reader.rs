use std::{
    cell::RefCell,
    io::{self, Read, Seek},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    element::{
        EbmlElement, EbmlElementType, EbmlHeaderElement, EbmlId, EbmlSpec, LazyValueElement,
        MasterElement, ValueElement,
    },
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
    _spec: PhantomData<S>,
}

pub struct EbmlIterator<S: EbmlSpec, R: Read + Seek> {
    reader: SharedReader<R>,
    end: Option<u64>,
    _spec: PhantomData<S>,
}

impl<S: EbmlSpec, R: Read + Seek> EbmlDocument<S, R> {
    pub fn new(reader: R) -> std::io::Result<Self> {
        let reader = Rc::new(RefCell::new(reader));
        let mut iter = EbmlIterator::<EbmlHeaderElement, R>::new(reader.clone());

        let Some(EbmlElement::Master(header_element)) = iter.next() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected EBML Header",
            ));
        };

        if !matches!(header_element.element, EbmlHeaderElement::Ebml) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected EBML Master Element",
            ));
        }

        let mut header = EbmlHeader::default();
        for element in header_element.children()? {
            let value = element.value().unwrap();
            match element.as_inner() {
                EbmlHeaderElement::Ebml => todo!(),
                EbmlHeaderElement::EbmlVersion => header.ebml_version = value.as_u64().unwrap(),
                EbmlHeaderElement::EbmlReadVersion => {
                    header.ebml_read_version = value.as_u64().unwrap()
                }
                EbmlHeaderElement::EbmlMaxIDLength => {
                    header.max_id_length = value.as_u64().unwrap()
                }
                EbmlHeaderElement::EbmlMaxSizeLength => {
                    header.max_size_length = value.as_u64().unwrap()
                }
                EbmlHeaderElement::DocType => header.doc_type = value.as_str().unwrap().to_owned(),
                EbmlHeaderElement::DocTypeVersion => {
                    header.doc_type_version = value.as_u64().unwrap()
                }
                EbmlHeaderElement::DocTypeReadVersion => {
                    header.doc_type_read_version = value.as_u64().unwrap()
                }
                EbmlHeaderElement::DocTypeExtension => todo!(),
                EbmlHeaderElement::DocTypeExtensionName => todo!(),
                EbmlHeaderElement::DocTypeExtensionVersion => todo!(),
                EbmlHeaderElement::Crc32 => todo!(),
                EbmlHeaderElement::Void => todo!(),
                EbmlHeaderElement::Unknown(_) => todo!(),
            }
        }

        Ok(Self {
            header,
            reader,
            _spec: PhantomData,
        })
    }

    pub fn iter(&self) -> EbmlIterator<S, R> {
        EbmlIterator::new(self.reader.clone())
    }
}

impl<S: EbmlSpec, R: Read + Seek> EbmlIterator<S, R> {
    pub(crate) fn new(reader: SharedReader<R>) -> Self {
        Self {
            reader,
            end: None,
            _spec: PhantomData,
        }
    }

    pub(crate) fn new_with_end(reader: SharedReader<R>, end: u64) -> Self {
        Self {
            reader,
            end: Some(end),
            _spec: PhantomData,
        }
    }
}

impl<S: EbmlSpec, R: Read + Seek> Iterator for EbmlIterator<S, R> {
    type Item = EbmlElement<S, R>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut reader = self.reader.borrow_mut();
        let start = reader.stream_position().unwrap();
        if let Some(end) = self.end {
            if start >= end {
                return None;
            }
        }

        let id = EbmlId::from_reader(&mut *reader).unwrap();
        let size = Vint::from_reader(&mut *reader).unwrap();
        let data_offset = reader.stream_position().unwrap();

        let s = S::from_id(id);
        let elem = match s.kind() {
            EbmlElementType::SignedInteger
            | EbmlElementType::UnsignedInteger
            | EbmlElementType::Float
            | EbmlElementType::Date => {
                let mut data = vec![0; size.value as usize];
                reader.read_exact(&mut data).unwrap();
                EbmlElement::Value(ValueElement {
                    element: s,
                    data,
                    _spec: PhantomData,
                })
            }
            EbmlElementType::String | EbmlElementType::Utf8 | EbmlElementType::Binary => {
                let elem = EbmlElement::LazyValue(LazyValueElement {
                    element: s,
                    data_offset,
                    size: size.value,
                    reader: self.reader.clone(),
                    _spec: PhantomData::<S>,
                });

                reader
                    .seek(io::SeekFrom::Current(size.value as i64))
                    .unwrap();

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

                reader
                    .seek(io::SeekFrom::Current(size.value as i64))
                    .unwrap();

                elem
            }
        };

        Some(elem)
    }
}
