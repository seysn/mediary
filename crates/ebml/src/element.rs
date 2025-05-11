use std::{
    cell::RefCell,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    rc::Rc,
};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::EbmlResult,
    reader::{EbmlIterator, SharedReader},
    vint::Vint,
};

#[derive(Clone, Copy)]
pub struct EbmlId(pub u64);

impl EbmlId {
    pub fn from_reader<R: Read>(reader: &mut R) -> EbmlResult<Self> {
        let vint = Vint::from_reader(reader)?;

        Ok(Self(vint.raw))
    }
}

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

pub trait EbmlSpec {
    fn from_id(id: EbmlId) -> Self;
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
    pub data_offset: u64,
    pub size: u64,
    pub reader: SharedReader<R>,
    pub _spec: PhantomData<S>,
}

pub struct ValueElement<S: EbmlSpec> {
    pub element: S,
    pub data: Vec<u8>,
    pub _spec: PhantomData<S>,
}

pub struct LazyValueElement<S: EbmlSpec, R: Read + Seek> {
    pub element: S,
    pub data_offset: u64,
    pub size: u64,
    pub reader: Rc<RefCell<R>>,
    pub _spec: PhantomData<S>,
}

impl EbmlElementType {
    pub fn to_value(self, data: &[u8]) -> EbmlElementValue {
        match self {
            EbmlElementType::SignedInteger => match data.len() {
                0 => EbmlElementValue::SignedInteger(0),
                1 => EbmlElementValue::SignedInteger(data[0] as i8 as i64),
                2 => EbmlElementValue::SignedInteger(BigEndian::read_i16(data) as i64),
                3 => EbmlElementValue::SignedInteger(BigEndian::read_i24(data) as i64),
                4 => EbmlElementValue::SignedInteger(BigEndian::read_i32(data) as i64),
                8 => EbmlElementValue::SignedInteger(BigEndian::read_i64(data)),
                _ => todo!("signed integer {:?}", data),
            },
            EbmlElementType::UnsignedInteger => match data.len() {
                0 => EbmlElementValue::UnsignedInteger(0),
                1 => EbmlElementValue::UnsignedInteger(data[0] as u64),
                2 => EbmlElementValue::UnsignedInteger(BigEndian::read_u16(data) as u64),
                3 => EbmlElementValue::UnsignedInteger(BigEndian::read_u24(data) as u64),
                4 => EbmlElementValue::UnsignedInteger(BigEndian::read_u32(data) as u64),
                8 => EbmlElementValue::UnsignedInteger(BigEndian::read_u64(data)),
                _ => todo!("unsigned integer data={:?}", data),
            },
            EbmlElementType::Float => match data.len() {
                0 => EbmlElementValue::Float(0.0),
                4 => EbmlElementValue::Float(BigEndian::read_f32(data) as f64),
                8 => EbmlElementValue::Float(BigEndian::read_f64(data)),
                _ => todo!("float {:?}", data),
            },
            EbmlElementType::String | EbmlElementType::Utf8 => {
                EbmlElementValue::String(String::from_utf8_lossy(data).to_string())
            }
            EbmlElementType::Date => todo!(),
            EbmlElementType::Binary => EbmlElementValue::Binary(data.to_owned()),
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

    pub fn value(&self) -> Option<EbmlElementValue> {
        match self {
            EbmlElement::Master(_) => None,
            EbmlElement::Value(value_element) => Some(value_element.value()),
            EbmlElement::LazyValue(lazy_value_element) => Some(lazy_value_element.value()),
        }
    }
}

impl<S: EbmlSpec, R: Read + Seek> MasterElement<S, R> {
    pub fn children(&self) -> EbmlIterator<S, R> {
        EbmlIterator::new(
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
    pub fn read(&self) -> Vec<u8> {
        let mut reader = self.reader.borrow_mut();
        reader.seek(SeekFrom::Start(self.data_offset)).unwrap();
        let mut data = vec![0; self.size as usize];
        reader.read_exact(&mut data).unwrap();
        data
    }

    pub fn value(&self) -> EbmlElementValue {
        let data = self.read();
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
    pub fn value(&self) -> EbmlElementValue {
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
            fn from_id(id: $crate::element::EbmlId) -> Self {
                match id.0 {
                    $($id => Self::$k,)+
                    _ => Self::Unknown(id),
                }
            }

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
