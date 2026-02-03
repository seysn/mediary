use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::error::{JpegError, JpegResult};

#[derive(Debug, Clone)]
pub struct ExifData {
    pub endianness: ExifEndianness,
    pub signature: u16,
    pub tags: ExifTags<StandardExifTag>,
    pub gps_tags: ExifTags<GpsTag>,
}

#[derive(Debug, Clone)]
pub struct ExifTags<T: ExifTag>(pub HashMap<ExifTagId, T>);

#[derive(Debug, Clone)]
pub enum ExifEndianness {
    BigEndian,
    LittleEndian,
}

#[derive(Debug, Clone, Copy)]
pub enum ExifDataFormat {
    UnsignedByte = 1,
    Ascii = 2,
    UnsignedShort = 3,
    UnsignedLong = 4,
    UnsignedRational = 5,
    SignedByte = 6,
    Undefined = 7,
    SignedShort = 8,
    SignedLong = 9,
    SignedRational = 10,
    SingleFloat = 11,
    DoubleFloat = 12,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ExifTagId(u16);

#[derive(Debug, Clone)]
pub enum ExifOrientation {
    UpperLeft = 1,
    LowerRight = 3,
    UpperRight = 6,
    LowerLeft = 8,
    Undefined = 9,
}

#[derive(Debug, Clone)]
pub enum ExifResolutionUnit {
    NoUnit = 1,
    Inch = 2,
    Centimeter = 3,
}

pub trait ExifTag {
    fn new(id: ExifTagId, value: ExifValue) -> Self;
}

#[derive(Debug, Clone)]
pub enum StandardExifTag {
    Unknown(ExifTagId, ExifValue),

    // IFD0
    ImageDescription(String),
    Make(String),
    Model(String),
    Orientation(ExifOrientation),
    XResolution(u64),
    YResolution(u64),
    ResolutionUnit(ExifResolutionUnit),
    Software(String),
    DateTime(String),
    WhitePoint(u64, u64),
    PrimaryChromaticities(u64, u64, u64, u64, u64, u64),
    YCbCrCoefficients(u64, u64, u64),
    YCbCrPositioning(u16),
    ReferenceBlackWhite(u64, u64, u64, u64, u64, u64),
    Copyright(String),
    ExifOffset(u32),
    GPSInfo(u32),

    // SubIFD
    ExposureTime(u64),
    FNumber(u64),
    ExposureProgram(u16),
    ISOSpeedRatings(u16, u16),
    ExifVersion(String),
    DateTimeOriginal(String),
    DateTimeDigitized(String),
    ComponentConfiguration(u8, u8, u8, u8),
    CompressedBitsPerPixel(u64),
    ShutterSpeedValue(i64),
    ApertureValue(u64),
    BrightnessValue(i64),
    ExposureBiasValue(i64),
    MaxApertureValue(u64),
    SubjectDistance(i64),
    MeteringMode(u16),
    LightSource(u16),
    Flash(u16),
    FocalLength(u64),
    MakerNote(String),
    UserComment(String),
    FlashPixVersion(String),
    ColorSpace(u16),
    ExifImageWidth(u32),
    ExifImageHeight(u32),
    RelatedSoundFile(String),
    ExifInteroperabilityOffset(u32),
    FocalPlaneXResolution(u64),
    FocalPlaneYResolution(u64),
    FocalPlaneResolutionUnit(u16),
    SensingMethod(u16),
    FileSource(u8),
    SceneType(u8),
}

#[derive(Debug, Clone)]
pub enum GpsTag {
    Unknown(ExifTagId, ExifValue),

    VersionID(u8, u8, u8, u8),
    LatitudeRef(String),
    Latitude(u64, u64, u64),
    LongitudeRef(String),
    Longitude(u64, u64, u64),
    AltitudeRef(u8),
    Altitude(u64),
    TimeStamp(u64, u64, u64),
    Satellites(String),
    Status(String),
    MeasureMode(String),
    DOP(u64),
    SpeedRef(String),
    Speed(u64),
    TrackRef(String),
    Track(u64),
    ImgDirectionRef(String),
    ImgDirection(u64),
    MapDatum(String),
    DestLatitudeRef(String),
    DestLatitude(u64, u64, u64),
    DestLongitudeRef(String),
    DestLongitude(u64, u64, u64),
    DestBearingRef(String),
    DestBearing(u64),
    DestDistanceRef(String),
    DestDistance(u64),
    ProcessingMethod,
    AreaInformation,
    DateStamp(String),
    Differential(u16),
    HPositioningError(u64),
}

#[derive(Debug, Clone)]
pub enum ExifValue {
    UnsignedByte(Vec<u8>),
    Ascii(String),
    UnsignedShort(Vec<u16>),
    UnsignedLong(Vec<u32>),
    UnsignedRational(Vec<u64>),
    SignedByte(Vec<i8>),
    Undefined(Vec<u32>),
    SignedShort(Vec<i16>),
    SignedLong(Vec<i32>),
    SignedRational(Vec<i64>),
    SingleFloat(Vec<f32>),
    DoubleFloat(Vec<f64>),
}

impl ExifDataFormat {
    pub fn component_bytes(&self) -> usize {
        match self {
            Self::UnsignedByte | Self::Ascii | Self::SignedByte | Self::Undefined => 1,
            Self::UnsignedShort | Self::SignedShort => 2,
            Self::UnsignedLong | Self::SignedLong | Self::SingleFloat => 4,
            Self::UnsignedRational | Self::SignedRational | Self::DoubleFloat => 8,
        }
    }
}

impl ExifValue {
    pub fn from_bytes<B: ByteOrder>(format: ExifDataFormat, data: &[u8]) -> Self {
        match format {
            ExifDataFormat::UnsignedByte => Self::UnsignedByte(data.to_vec()),
            ExifDataFormat::Ascii => {
                Self::Ascii(String::from_utf8_lossy(&data[..data.len() - 1]).to_string())
            }
            ExifDataFormat::UnsignedShort => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_u16(b));
                }
                Self::UnsignedShort(vec)
            }
            ExifDataFormat::UnsignedLong => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_u32(b));
                }
                Self::UnsignedLong(vec)
            }
            ExifDataFormat::UnsignedRational => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_u64(b));
                }
                Self::UnsignedRational(vec)
            }
            ExifDataFormat::SignedByte => todo!(),
            ExifDataFormat::Undefined => {
                Self::Ascii(String::from_utf8_lossy(&data[..data.len() - 1]).to_string())
            }
            ExifDataFormat::SignedShort => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_i16(b));
                }
                Self::SignedShort(vec)
            }
            ExifDataFormat::SignedLong => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_i32(b));
                }
                Self::SignedLong(vec)
            }
            ExifDataFormat::SignedRational => {
                let mut vec = Vec::new();
                for b in data.chunks_exact(format.component_bytes()) {
                    vec.push(B::read_i64(b));
                }
                Self::SignedRational(vec)
            }
            ExifDataFormat::SingleFloat => todo!(),
            ExifDataFormat::DoubleFloat => todo!(),
        }
    }

    pub fn from_u32(format: ExifDataFormat, value: u32) -> Self {
        match format {
            ExifDataFormat::UnsignedByte => Self::UnsignedByte(vec![value as u8]),
            ExifDataFormat::Ascii => {
                Self::Ascii(String::from_utf8_lossy(&value.to_le_bytes()).to_string())
            }
            ExifDataFormat::UnsignedShort => Self::UnsignedShort(vec![value as u16]),
            ExifDataFormat::UnsignedLong => Self::UnsignedLong(vec![value]),
            ExifDataFormat::UnsignedRational => todo!(),
            ExifDataFormat::SignedByte => todo!(),
            ExifDataFormat::Undefined => Self::Undefined(vec![value]),
            ExifDataFormat::SignedShort => todo!(),
            ExifDataFormat::SignedLong => todo!(),
            ExifDataFormat::SignedRational => todo!(),
            ExifDataFormat::SingleFloat => todo!(),
            ExifDataFormat::DoubleFloat => todo!(),
        }
    }
}

impl<T: ExifTag> ExifTags<T> {
    pub fn parse_ifd<B: ByteOrder>(data: &[u8], offset: usize) -> JpegResult<(Self, usize)> {
        let n_directories = B::read_u16(&data[0..2]);

        let mut ifd = &data[2..];
        let mut tags = HashMap::new();
        for _ in 0..n_directories {
            let id = ExifTagId(B::read_u16(&ifd[0..2]));
            let data_format = ExifDataFormat::try_from(B::read_u16(&ifd[2..4]))?;
            let n_components = B::read_u32(&ifd[4..8]);
            let value = B::read_u32(&ifd[8..12]);

            let size = data_format.component_bytes() * n_components as usize;
            let value = if size > 4 {
                let start = value as usize - offset;
                let end = start + size;

                ExifValue::from_bytes::<B>(data_format, &data[start..end])
            } else {
                ExifValue::from_u32(data_format, value)
            };

            tags.insert(id, T::new(id, value));

            ifd = &ifd[12..];
        }

        let next_ifd = B::read_u32(&ifd[0..4]) as usize;

        Ok((Self(tags), next_ifd))
    }
}

impl ExifData {
    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        let endianness = ExifEndianness::try_from(&data[0..2])?;
        let signature = LittleEndian::read_u16(&data[2..4]);

        let (mut tags, _next_ifd) = match endianness {
            ExifEndianness::BigEndian => {
                let ifd0_offset = BigEndian::read_u32(&data[4..8]) as usize;
                ExifTags::parse_ifd::<BigEndian>(&data[ifd0_offset..], ifd0_offset)?
            }
            ExifEndianness::LittleEndian => {
                let ifd0_offset = LittleEndian::read_u32(&data[4..8]) as usize;
                ExifTags::parse_ifd::<LittleEndian>(&data[ifd0_offset..], ifd0_offset)?
            }
        };

        if let Some(StandardExifTag::ExifOffset(subifd_offset)) = tags.0.get(&ExifTagId(0x8769)) {
            let offset = (*subifd_offset) as usize;
            let (sub_tags, _next_ifd) = match endianness {
                ExifEndianness::BigEndian => {
                    ExifTags::parse_ifd::<BigEndian>(&data[offset..], offset)?
                }
                ExifEndianness::LittleEndian => {
                    ExifTags::parse_ifd::<LittleEndian>(&data[offset..], offset)?
                }
            };

            tags.0.extend(sub_tags.0);
        }

        let gps_tags =
            if let Some(StandardExifTag::GPSInfo(gps_info)) = tags.0.get(&ExifTagId(0x8825)) {
                let offset = (*gps_info) as usize;
                let (gps_tags, _next_ifd) = match endianness {
                    ExifEndianness::BigEndian => {
                        ExifTags::parse_ifd::<BigEndian>(&data[offset..], offset)?
                    }
                    ExifEndianness::LittleEndian => {
                        ExifTags::parse_ifd::<LittleEndian>(&data[offset..], offset)?
                    }
                };

                gps_tags
            } else {
                ExifTags(HashMap::new())
            };

        Ok(Self {
            endianness,
            signature,
            tags,
            gps_tags,
        })
    }
}

impl ExifTag for StandardExifTag {
    fn new(id: ExifTagId, value: ExifValue) -> Self {
        match value {
            ExifValue::UnsignedByte(v) => match (id.0, v.as_slice()) {
                (0x9101, &[a, b, c, d]) => Self::ComponentConfiguration(a, b, c, d),
                (0xa300, &[a]) => Self::FileSource(a),
                (0xa301, &[a]) => Self::SceneType(a),
                _ => Self::Unknown(id, ExifValue::UnsignedByte(v)),
            },
            ExifValue::Ascii(v) => match id.0 {
                0x010e => Self::ImageDescription(v),
                0x010f => Self::Make(v),
                0x0110 => Self::Model(v),
                0x0131 => Self::Software(v),
                0x0132 => Self::DateTime(v),
                0x8298 => Self::Copyright(v),
                0x9000 => Self::ExifVersion(v),
                0x9003 => Self::DateTimeOriginal(v),
                0x9004 => Self::DateTimeDigitized(v),
                0x927c => Self::MakerNote(v),
                0x9286 => Self::UserComment(v),
                0xa000 => Self::FlashPixVersion(v),
                0xa004 => Self::RelatedSoundFile(v),
                _ => Self::Unknown(id, ExifValue::Ascii(v)),
            },
            ExifValue::UnsignedShort(v) => match (id.0, v.as_slice()) {
                (0x0112, &[a]) => a
                    .try_into()
                    .map(Self::Orientation)
                    .unwrap_or_else(|_| Self::Unknown(id, ExifValue::UnsignedShort(v))),
                (0x0128, &[a]) => a
                    .try_into()
                    .map(Self::ResolutionUnit)
                    .unwrap_or_else(|_| Self::Unknown(id, ExifValue::UnsignedShort(v))),
                (0x0213, &[a]) => Self::YCbCrPositioning(a),
                (0x8822, &[a]) => Self::ExposureProgram(a),
                (0x8827, &[a, b]) => Self::ISOSpeedRatings(a, b),
                (0x9207, &[a]) => Self::MeteringMode(a),
                (0x9208, &[a]) => Self::LightSource(a),
                (0x9209, &[a]) => Self::Flash(a),
                (0xa001, &[a]) => Self::ColorSpace(a),
                (0xa210, &[a]) => Self::FocalPlaneResolutionUnit(a),
                (0xa217, &[a]) => Self::SensingMethod(a),
                _ => Self::Unknown(id, ExifValue::UnsignedShort(v)),
            },
            ExifValue::UnsignedLong(v) => match (id.0, v.as_slice()) {
                (0x8769, &[a]) => Self::ExifOffset(a),
                (0xa002, &[a]) => Self::ExifImageWidth(a),
                (0xa003, &[a]) => Self::ExifImageHeight(a),
                (0xa005, &[a]) => Self::ExifInteroperabilityOffset(a),
                (0x8825, &[a]) => Self::GPSInfo(a),
                _ => Self::Unknown(id, ExifValue::UnsignedLong(v)),
            },
            ExifValue::UnsignedRational(v) => match (id.0, v.as_slice()) {
                (0x011a, &[a]) => Self::XResolution(a),
                (0x011b, &[a]) => Self::YResolution(a),
                (0x013e, &[a, b]) => Self::WhitePoint(a, b),
                (0x013f, &[a, b, c, d, e, f]) => Self::PrimaryChromaticities(a, b, c, d, e, f),
                (0x0211, &[a, b, c]) => Self::YCbCrCoefficients(a, b, c),
                (0x0214, &[a, b, c, d, e, f]) => Self::ReferenceBlackWhite(a, b, c, d, e, f),
                (0x829a, &[a]) => Self::ExposureTime(a),
                (0x829d, &[a]) => Self::FNumber(a),
                (0x9102, &[a]) => Self::CompressedBitsPerPixel(a),
                (0x9202, &[a]) => Self::ApertureValue(a),
                (0x9205, &[a]) => Self::MaxApertureValue(a),
                (0x920a, &[a]) => Self::FocalLength(a),
                (0xa20e, &[a]) => Self::FocalPlaneXResolution(a),
                (0xa20f, &[a]) => Self::FocalPlaneYResolution(a),
                _ => Self::Unknown(id, ExifValue::UnsignedRational(v)),
            },
            ExifValue::SignedByte(v) => Self::Unknown(id, ExifValue::SignedByte(v)),
            ExifValue::Undefined(v) => Self::Unknown(id, ExifValue::Undefined(v)),
            ExifValue::SignedShort(v) => Self::Unknown(id, ExifValue::SignedShort(v)),
            ExifValue::SignedLong(v) => Self::Unknown(id, ExifValue::SignedLong(v)),
            ExifValue::SignedRational(v) => match (id.0, v.as_slice()) {
                (0x9201, &[a]) => Self::ShutterSpeedValue(a),
                (0x9203, &[a]) => Self::BrightnessValue(a),
                (0x9204, &[a]) => Self::ExposureBiasValue(a),
                (0x9206, &[a]) => Self::SubjectDistance(a),
                _ => Self::Unknown(id, ExifValue::SignedRational(v)),
            },
            ExifValue::SingleFloat(v) => Self::Unknown(id, ExifValue::SingleFloat(v)),
            ExifValue::DoubleFloat(v) => Self::Unknown(id, ExifValue::DoubleFloat(v)),
        }
    }
}

impl ExifTag for GpsTag {
    fn new(id: ExifTagId, value: ExifValue) -> Self {
        match value {
            ExifValue::UnsignedByte(v) => match (id.0, v.as_slice()) {
                (0x0000, &[a, b, c, d]) => Self::VersionID(a, b, c, d),
                (0x0005, &[a]) => Self::AltitudeRef(a),
                _ => Self::Unknown(id, ExifValue::UnsignedByte(v)),
            },
            ExifValue::Ascii(v) => match id.0 {
                0x0001 => Self::LatitudeRef(v),
                0x0003 => Self::LongitudeRef(v),
                0x0008 => Self::Satellites(v),
                0x0009 => Self::Status(v),
                0x000a => Self::MeasureMode(v),
                0x000c => Self::SpeedRef(v),
                0x000e => Self::TrackRef(v),
                0x0010 => Self::ImgDirectionRef(v),
                0x0012 => Self::MapDatum(v),
                0x0013 => Self::DestLatitudeRef(v),
                0x0015 => Self::DestLongitudeRef(v),
                0x0017 => Self::DestBearingRef(v),
                0x0019 => Self::DestDistanceRef(v),
                0x001d => Self::DateStamp(v),
                _ => Self::Unknown(id, ExifValue::Ascii(v)),
            },
            ExifValue::UnsignedShort(v) => match (id.0, v.as_slice()) {
                (0x001e, &[a]) => Self::Differential(a),
                _ => Self::Unknown(id, ExifValue::UnsignedShort(v)),
            },
            ExifValue::UnsignedLong(v) => Self::Unknown(id, ExifValue::UnsignedLong(v)),
            ExifValue::UnsignedRational(v) => match (id.0, v.as_slice()) {
                (0x0002, &[a, b, c]) => Self::Latitude(a, b, c),
                (0x0004, &[a, b, c]) => Self::Longitude(a, b, c),
                (0x0006, &[a]) => Self::Altitude(a),
                (0x0007, &[a, b, c]) => Self::TimeStamp(a, b, c),
                (0x000b, &[a]) => Self::DOP(a),
                (0x000d, &[a]) => Self::Speed(a),
                (0x000f, &[a]) => Self::Track(a),
                (0x0011, &[a]) => Self::ImgDirection(a),
                (0x0014, &[a, b, c]) => Self::DestLatitude(a, b, c),
                (0x0016, &[a, b, c]) => Self::DestLongitude(a, b, c),
                (0x0018, &[a]) => Self::DestBearing(a),
                (0x001a, &[a]) => Self::DestDistance(a),
                (0x001f, &[a]) => Self::HPositioningError(a),
                _ => Self::Unknown(id, ExifValue::UnsignedRational(v)),
            },
            ExifValue::SignedByte(v) => Self::Unknown(id, ExifValue::SignedByte(v)),
            ExifValue::Undefined(v) => Self::Unknown(id, ExifValue::Undefined(v)),
            ExifValue::SignedShort(v) => Self::Unknown(id, ExifValue::SignedShort(v)),
            ExifValue::SignedLong(v) => Self::Unknown(id, ExifValue::SignedLong(v)),
            ExifValue::SignedRational(v) => Self::Unknown(id, ExifValue::SignedRational(v)),
            ExifValue::SingleFloat(v) => Self::Unknown(id, ExifValue::SingleFloat(v)),
            ExifValue::DoubleFloat(v) => Self::Unknown(id, ExifValue::DoubleFloat(v)),
        }
    }
}

impl StandardExifTag {
    pub fn id(&self) -> ExifTagId {
        match self {
            StandardExifTag::ImageDescription(_) => ExifTagId(0x010e),
            StandardExifTag::Make(_) => ExifTagId(0x010f),
            StandardExifTag::Model(_) => ExifTagId(0x0110),
            StandardExifTag::Orientation(_) => ExifTagId(0x0112),
            StandardExifTag::XResolution(_) => ExifTagId(0x011a),
            StandardExifTag::YResolution(_) => ExifTagId(0x011b),
            StandardExifTag::ResolutionUnit(_) => ExifTagId(0x0128),
            StandardExifTag::Software(_) => ExifTagId(0x0131),
            StandardExifTag::DateTime(_) => ExifTagId(0x0132),
            StandardExifTag::WhitePoint(_, _) => ExifTagId(0x013e),
            StandardExifTag::PrimaryChromaticities(_, _, _, _, _, _) => ExifTagId(0x013f),
            StandardExifTag::YCbCrCoefficients(_, _, _) => ExifTagId(0x0211),
            StandardExifTag::YCbCrPositioning(_) => ExifTagId(0x0213),
            StandardExifTag::ReferenceBlackWhite(_, _, _, _, _, _) => ExifTagId(0x0214),
            StandardExifTag::Copyright(_) => ExifTagId(0x8298),
            StandardExifTag::ExifOffset(_) => ExifTagId(0x8769),
            StandardExifTag::GPSInfo(_) => ExifTagId(0x8825),
            StandardExifTag::ExposureTime(_) => ExifTagId(0x829a),
            StandardExifTag::FNumber(_) => ExifTagId(0x829d),
            StandardExifTag::ExposureProgram(_) => ExifTagId(0x8822),
            StandardExifTag::ISOSpeedRatings(_, _) => ExifTagId(0x8827),
            StandardExifTag::ExifVersion(_) => ExifTagId(0x9000),
            StandardExifTag::DateTimeOriginal(_) => ExifTagId(0x9003),
            StandardExifTag::DateTimeDigitized(_) => ExifTagId(0x9004),
            StandardExifTag::ComponentConfiguration(_, _, _, _) => ExifTagId(0x9101),
            StandardExifTag::CompressedBitsPerPixel(_) => ExifTagId(0x9102),
            StandardExifTag::ShutterSpeedValue(_) => ExifTagId(0x9201),
            StandardExifTag::ApertureValue(_) => ExifTagId(0x9202),
            StandardExifTag::BrightnessValue(_) => ExifTagId(0x9203),
            StandardExifTag::ExposureBiasValue(_) => ExifTagId(0x9204),
            StandardExifTag::MaxApertureValue(_) => ExifTagId(0x9205),
            StandardExifTag::SubjectDistance(_) => ExifTagId(0x9206),
            StandardExifTag::MeteringMode(_) => ExifTagId(0x9207),
            StandardExifTag::LightSource(_) => ExifTagId(0x9208),
            StandardExifTag::Flash(_) => ExifTagId(0x9209),
            StandardExifTag::FocalLength(_) => ExifTagId(0x920a),
            StandardExifTag::MakerNote(_) => ExifTagId(0x927c),
            StandardExifTag::UserComment(_) => ExifTagId(0x9286),
            StandardExifTag::FlashPixVersion(_) => ExifTagId(0xa000),
            StandardExifTag::ColorSpace(_) => ExifTagId(0xa001),
            StandardExifTag::ExifImageWidth(_) => ExifTagId(0xa002),
            StandardExifTag::ExifImageHeight(_) => ExifTagId(0xa003),
            StandardExifTag::RelatedSoundFile(_) => ExifTagId(0xa004),
            StandardExifTag::ExifInteroperabilityOffset(_) => ExifTagId(0xa005),
            StandardExifTag::FocalPlaneXResolution(_) => ExifTagId(0xa20e),
            StandardExifTag::FocalPlaneYResolution(_) => ExifTagId(0xa20f),
            StandardExifTag::FocalPlaneResolutionUnit(_) => ExifTagId(0xa210),
            StandardExifTag::SensingMethod(_) => ExifTagId(0xa217),
            StandardExifTag::FileSource(_) => ExifTagId(0xa300),
            StandardExifTag::SceneType(_) => ExifTagId(0xa301),
            StandardExifTag::Unknown(id, _) => *id,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StandardExifTag::ImageDescription(_) => "ImageDescription",
            StandardExifTag::Make(_) => "Make",
            StandardExifTag::Model(_) => "Model",
            StandardExifTag::Orientation(_) => "Orientation",
            StandardExifTag::XResolution(_) => "XResolution",
            StandardExifTag::YResolution(_) => "YResolution",
            StandardExifTag::ResolutionUnit(_) => "ResolutionUnit",
            StandardExifTag::Software(_) => "Software",
            StandardExifTag::DateTime(_) => "DateTime",
            StandardExifTag::WhitePoint(_, _) => "WhitePoint",
            StandardExifTag::PrimaryChromaticities(_, _, _, _, _, _) => "PrimaryChromaticities",
            StandardExifTag::YCbCrCoefficients(_, _, _) => "YCbCrCoefficients",
            StandardExifTag::YCbCrPositioning(_) => "YCbCrPositioning",
            StandardExifTag::ReferenceBlackWhite(_, _, _, _, _, _) => "ReferenceBlackWhite",
            StandardExifTag::Copyright(_) => "Copyright",
            StandardExifTag::ExifOffset(_) => "ExifOffset",
            StandardExifTag::GPSInfo(_) => "GPSInfo",
            StandardExifTag::ExposureTime(_) => "ExposureTime",
            StandardExifTag::FNumber(_) => "FNumber",
            StandardExifTag::ExposureProgram(_) => "ExposureProgram",
            StandardExifTag::ISOSpeedRatings(_, _) => "ISOSpeedRatings",
            StandardExifTag::ExifVersion(_) => "ExifVersion",
            StandardExifTag::DateTimeOriginal(_) => "DateTimeOriginal",
            StandardExifTag::DateTimeDigitized(_) => "DateTimeDigitized",
            StandardExifTag::ComponentConfiguration(_, _, _, _) => "ComponentConfiguration",
            StandardExifTag::CompressedBitsPerPixel(_) => "CompressedBitsPerPixel",
            StandardExifTag::ShutterSpeedValue(_) => "ShutterSpeedValue",
            StandardExifTag::ApertureValue(_) => "ApertureValue",
            StandardExifTag::BrightnessValue(_) => "BrightnessValue",
            StandardExifTag::ExposureBiasValue(_) => "ExposureBiasValue",
            StandardExifTag::MaxApertureValue(_) => "MaxApertureValue",
            StandardExifTag::SubjectDistance(_) => "SubjectDistance",
            StandardExifTag::MeteringMode(_) => "MeteringMode",
            StandardExifTag::LightSource(_) => "LightSource",
            StandardExifTag::Flash(_) => "Flash",
            StandardExifTag::FocalLength(_) => "FocalLength",
            StandardExifTag::MakerNote(_) => "MakerNote",
            StandardExifTag::UserComment(_) => "UserComment",
            StandardExifTag::FlashPixVersion(_) => "FlashPixVersion",
            StandardExifTag::ColorSpace(_) => "ColorSpace",
            StandardExifTag::ExifImageWidth(_) => "ExifImageWidth",
            StandardExifTag::ExifImageHeight(_) => "ExifImageHeight",
            StandardExifTag::RelatedSoundFile(_) => "RelatedSoundFile",
            StandardExifTag::ExifInteroperabilityOffset(_) => "ExifInteroperabilityOffset",
            StandardExifTag::FocalPlaneXResolution(_) => "FocalPlaneXResolution",
            StandardExifTag::FocalPlaneYResolution(_) => "FocalPlaneYResolution",
            StandardExifTag::FocalPlaneResolutionUnit(_) => "FocalPlaneResolutionUnit",
            StandardExifTag::SensingMethod(_) => "SensingMethod",
            StandardExifTag::FileSource(_) => "FileSource",
            StandardExifTag::SceneType(_) => "SceneType",
            StandardExifTag::Unknown(_, _) => "Unknown",
        }
    }
}

impl TryFrom<u16> for ExifOrientation {
    type Error = JpegError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::UpperLeft,
            3 => Self::LowerRight,
            6 => Self::UpperRight,
            8 => Self::LowerLeft,
            9 => Self::Undefined,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "ExifOrientation",
                    value: Box::new(value),
                })
            }
        })
    }
}

impl TryFrom<u16> for ExifResolutionUnit {
    type Error = JpegError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::NoUnit,
            2 => Self::Inch,
            3 => Self::Centimeter,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "ExifResolutionUnit",
                    value: Box::new(value),
                })
            }
        })
    }
}

impl TryFrom<&[u8]> for ExifEndianness {
    type Error = JpegError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(match value {
            b"II" => Self::LittleEndian,
            b"MM" => Self::BigEndian,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "ExifEndianness",
                    value: Box::new(value.to_vec()),
                })
            }
        })
    }
}

impl TryFrom<u16> for ExifDataFormat {
    type Error = JpegError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::UnsignedByte,
            2 => Self::Ascii,
            3 => Self::UnsignedShort,
            4 => Self::UnsignedLong,
            5 => Self::UnsignedRational,
            6 => Self::SignedByte,
            7 => Self::Undefined,
            8 => Self::SignedShort,
            9 => Self::SignedLong,
            10 => Self::SignedRational,
            11 => Self::SingleFloat,
            12 => Self::DoubleFloat,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "ExifDataFormat",
                    value: Box::new(value),
                })
            }
        })
    }
}

impl std::fmt::Debug for ExifTagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExifTagId(0x{:04x})", self.0)
    }
}
