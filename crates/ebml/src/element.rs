use std::{
    cell::RefCell,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    rc::Rc,
};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::{EbmlError, EbmlResult},
    reader::SharedReader,
    vint::Vint,
    EbmlReader,
};

#[derive(Clone, Copy)]
pub struct EbmlId(pub u64);

#[derive(Debug)]
pub enum EbmlElementType {
    SignedInteger,
    UnsignedInteger,
    Float,
    String,
    Utf8,
    Date,
    Master,
    Binary,
}

#[derive(Debug)]
pub enum EbmlElementValue {
    SignedInteger(i64),
    UnsignedInteger(u64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
}

pub trait EbmlSpec: From<EbmlId> {
    fn id(&self) -> EbmlId;
    fn name(&self) -> &'static str;
    fn kind(&self) -> EbmlElementType;
}

pub enum EbmlElement<S: EbmlSpec, R: Read + Seek> {
    Master(MasterElement<S, R>),
    Value(ValueElement<S>),
    LazyValue(LazyValueElement<S, R>),
}

pub struct MasterElement<S: EbmlSpec, R: Read + Seek> {
    pub element: S,
    pub position: u64,
    pub data_offset: u64,
    pub size: u64,
    pub reader: SharedReader<R>,
    pub _spec: PhantomData<S>,
}

pub struct ValueElement<S: EbmlSpec> {
    pub element: S,
    pub position: u64,
    pub data: Vec<u8>,
    pub _spec: PhantomData<S>,
}

pub struct LazyValueElement<S: EbmlSpec, R: Read + Seek> {
    pub element: S,
    pub position: u64,
    pub data_offset: u64,
    pub size: u64,
    pub reader: Rc<RefCell<R>>,
    pub _spec: PhantomData<S>,
}

impl EbmlId {
    pub fn from_reader<R: Read>(reader: &mut R) -> EbmlResult<Self> {
        let vint = Vint::from_reader(reader)?;

        Ok(Self(vint.raw))
    }
}

impl From<Vint> for EbmlId {
    fn from(value: Vint) -> Self {
        Self(value.raw)
    }
}

impl TryFrom<&[u8]> for EbmlId {
    type Error = EbmlError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytes_to_u64(data).map(Self)
    }
}

impl EbmlElementType {
    pub fn to_value(self, data: &[u8]) -> EbmlResult<EbmlElementValue> {
        match self {
            EbmlElementType::SignedInteger => {
                bytes_to_i64(data).map(EbmlElementValue::SignedInteger)
            }
            EbmlElementType::UnsignedInteger => {
                bytes_to_u64(data).map(EbmlElementValue::UnsignedInteger)
            }
            EbmlElementType::Float => bytes_to_f64(data).map(EbmlElementValue::Float),
            EbmlElementType::String | EbmlElementType::Utf8 => Ok(EbmlElementValue::String(
                String::from_utf8_lossy(data).to_string(),
            )),
            EbmlElementType::Date => todo!(),
            EbmlElementType::Binary => Ok(EbmlElementValue::Binary(data.to_owned())),
            EbmlElementType::Master => unreachable!(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EbmlElementType::SignedInteger => "SignedInteger",
            EbmlElementType::UnsignedInteger => "UnsignedInteger",
            EbmlElementType::Float => "Float",
            EbmlElementType::String => "String",
            EbmlElementType::Utf8 => "Utf8",
            EbmlElementType::Date => "Date",
            EbmlElementType::Master => "Master",
            EbmlElementType::Binary => "Binary",
        }
    }
}

impl EbmlElementValue {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::SignedInteger(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::UnsignedInteger(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

impl<S: EbmlSpec, R: Read + Seek> EbmlElement<S, R> {
    pub fn as_inner(&self) -> &S {
        match self {
            EbmlElement::Master(master_element) => &master_element.element,
            EbmlElement::Value(value_element) => &value_element.element,
            EbmlElement::LazyValue(lazy_value_element) => &lazy_value_element.element,
        }
    }

    pub fn id(&self) -> EbmlId {
        match self {
            EbmlElement::Master(master_element) => master_element.element.id(),
            EbmlElement::Value(value_element) => value_element.element.id(),
            EbmlElement::LazyValue(lazy_value_element) => lazy_value_element.element.id(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EbmlElement::Master(_) => "Master",
            EbmlElement::Value(_) => "Value",
            EbmlElement::LazyValue(_) => "LazyValue",
        }
    }

    pub fn element_name(&self) -> &'static str {
        match self {
            EbmlElement::Master(master_element) => master_element.element.name(),
            EbmlElement::Value(value_element) => value_element.element.name(),
            EbmlElement::LazyValue(lazy_value_element) => lazy_value_element.element.name(),
        }
    }

    pub fn kind(&self) -> EbmlElementType {
        match self {
            EbmlElement::Master(master_element) => master_element.element.kind(),
            EbmlElement::Value(value_element) => value_element.element.kind(),
            EbmlElement::LazyValue(lazy_value_element) => lazy_value_element.element.kind(),
        }
    }

    pub fn value(&self) -> EbmlResult<Option<EbmlElementValue>> {
        Ok(match self {
            EbmlElement::Master(_) => None,
            EbmlElement::Value(value_element) => Some(value_element.value()?),
            EbmlElement::LazyValue(lazy_value_element) => Some(lazy_value_element.value()?),
        })
    }
}

impl<S: EbmlSpec, R: Read + Seek> MasterElement<S, R> {
    pub fn children(&self) -> EbmlReader<S, R> {
        EbmlReader::new_with_range(
            self.reader.clone(),
            self.data_offset,
            self.data_offset + self.size,
        )
    }

    pub fn id(&self) -> EbmlId {
        self.element.id()
    }

    pub fn name(&self) -> &'static str {
        self.element.name()
    }

    pub fn kind(&self) -> EbmlElementType {
        self.element.kind()
    }
}

impl<S: EbmlSpec, R: Read + Seek> LazyValueElement<S, R> {
    pub fn read(&self) -> EbmlResult<Vec<u8>> {
        let mut reader = self.reader.borrow_mut();
        reader.seek(SeekFrom::Start(self.data_offset))?;
        let mut data = vec![0; self.size as usize];
        reader.read_exact(&mut data)?;
        Ok(data)
    }

    pub fn value(&self) -> EbmlResult<EbmlElementValue> {
        let data = self.read()?;
        self.element.kind().to_value(&data)
    }

    pub fn id(&self) -> EbmlId {
        self.element.id()
    }

    pub fn name(&self) -> &'static str {
        self.element.name()
    }

    pub fn kind(&self) -> EbmlElementType {
        self.element.kind()
    }
}

impl<S: EbmlSpec> ValueElement<S> {
    pub fn value(&self) -> EbmlResult<EbmlElementValue> {
        self.element.kind().to_value(&self.data)
    }

    pub fn id(&self) -> EbmlId {
        self.element.id()
    }

    pub fn name(&self) -> &'static str {
        self.element.name()
    }

    pub fn kind(&self) -> EbmlElementType {
        self.element.kind()
    }
}

#[macro_export]
macro_rules! declare_elements {
    ($name:ident, $($k:ident($id:literal, $ty:expr)),+) => {
        #[derive(Debug)]
        pub enum $name {
            $($k,)+
            Unknown($crate::element::EbmlId),
        }

        impl $crate::element::EbmlSpec for $name {
            fn id(&self) -> $crate::element::EbmlId {
                match self {
                    $(Self::$k => $crate::element::EbmlId($id),)+
                    Self::Unknown(id) => *id,
                }
            }

            fn name(&self) -> &'static str {
                match self {
                    $(Self::$k => stringify!($k),)+
                    Self::Unknown(_) => "Unknown",
                }
            }

            fn kind(&self) -> $crate::element::EbmlElementType {
                match self {
                    $(Self::$k => $ty,)+
                    Self::Unknown(_) => $crate::element::EbmlElementType::Binary,
                }
            }
        }

        impl From<$crate::element::EbmlId> for $name {
            fn from(id: $crate::element::EbmlId) -> Self {
                match id.0 {
                    $($id => Self::$k,)+
                    _ => Self::Unknown(id),
                }
            }
        }
    };
}

// EBML Header
// https://datatracker.ietf.org/doc/html/rfc8794#name-ebml-header-elements
declare_elements!(
    EbmlHeaderElement,
    Ebml(0x1A45DFA3, EbmlElementType::Master),
    EbmlVersion(0x4286, EbmlElementType::UnsignedInteger),
    EbmlReadVersion(0x42F7, EbmlElementType::UnsignedInteger),
    EbmlMaxIDLength(0x42F2, EbmlElementType::UnsignedInteger),
    EbmlMaxSizeLength(0x42F3, EbmlElementType::UnsignedInteger),
    DocType(0x4282, EbmlElementType::String),
    DocTypeVersion(0x4287, EbmlElementType::UnsignedInteger),
    DocTypeReadVersion(0x4285, EbmlElementType::UnsignedInteger),
    DocTypeExtension(0x4281, EbmlElementType::Master),
    DocTypeExtensionName(0x4283, EbmlElementType::String),
    DocTypeExtensionVersion(0x4284, EbmlElementType::UnsignedInteger),
    Crc32(0xBF, EbmlElementType::Binary),
    Void(0xEC, EbmlElementType::Binary)
);

impl std::fmt::Debug for EbmlId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("EbmlId(0x{:x})", self.0))
    }
}

fn bytes_to_u64(data: &[u8]) -> EbmlResult<u64> {
    Ok(match data.len() {
        0 => 0,
        n @ 1..=8 => BigEndian::read_uint(data, n),
        _ => return Err(EbmlError::InvalidDataLength),
    })
}

fn bytes_to_i64(data: &[u8]) -> EbmlResult<i64> {
    Ok(match data.len() {
        0 => 0,
        n @ 1..=8 => BigEndian::read_int(data, n),
        _ => return Err(EbmlError::InvalidDataLength),
    })
}

fn bytes_to_f64(data: &[u8]) -> EbmlResult<f64> {
    Ok(match data.len() {
        0 => 0.0,
        4 => BigEndian::read_f32(data) as f64,
        8 => BigEndian::read_f64(data),
        _ => return Err(EbmlError::InvalidDataLength),
    })
}

macro_rules! impl_try_from_element {
    ($in:ident, $out:ty) => {
        impl<S: EbmlSpec, R: Read + Seek> TryFrom<EbmlElement<S, R>> for $out {
            type Error = EbmlError;

            fn try_from(element: EbmlElement<S, R>) -> Result<Self, Self::Error> {
                let EbmlElement::$in(value) = element else {
                    return Err(EbmlError::UnexpectedElement {
                        expected: stringify!($in),
                        found: element.name(),
                    });
                };

                Ok(value)
            }
        }
    };
}

macro_rules! impl_try_from_element_value {
    ($in:ident, $out:ty) => {
        impl<S: EbmlSpec, R: Read + Seek> TryFrom<EbmlElement<S, R>> for $out {
            type Error = EbmlError;

            fn try_from(element: EbmlElement<S, R>) -> Result<Self, Self::Error> {
                let Some(EbmlElementValue::$in(value)) = element.value()? else {
                    return Err(EbmlError::UnexpectedElement {
                        expected: stringify!($in),
                        found: element.kind().name(),
                    });
                };

                Ok(value)
            }
        }
    };
}

impl_try_from_element!(Master, MasterElement<S, R>);
impl_try_from_element!(Value, ValueElement<S>);
impl_try_from_element!(LazyValue, LazyValueElement<S, R>);
impl_try_from_element_value!(UnsignedInteger, u64);
impl_try_from_element_value!(SignedInteger, i64);
impl_try_from_element_value!(Float, f64);
impl_try_from_element_value!(String, String);
impl_try_from_element_value!(Binary, Vec<u8>);
