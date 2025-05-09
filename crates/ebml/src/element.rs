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
pub struct EbmlId(u64);

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
                1 => EbmlElementValue::SignedInteger(data[0] as i8 as i64),
                2 => EbmlElementValue::SignedInteger(BigEndian::read_i16(&data) as i64),
                4 => EbmlElementValue::SignedInteger(BigEndian::read_i32(&data) as i64),
                8 => EbmlElementValue::SignedInteger(BigEndian::read_i64(&data)),
                _ => todo!(),
            },
            EbmlElementType::UnsignedInteger => match data.len() {
                1 => EbmlElementValue::UnsignedInteger(data[0] as u64),
                2 => EbmlElementValue::UnsignedInteger(BigEndian::read_u16(&data) as u64),
                4 => EbmlElementValue::UnsignedInteger(BigEndian::read_u32(&data) as u64),
                8 => EbmlElementValue::UnsignedInteger(BigEndian::read_u64(&data)),
                _ => todo!(),
            },
            EbmlElementType::Float => todo!(),
            EbmlElementType::String | EbmlElementType::Utf8 => {
                EbmlElementValue::String(String::from_utf8_lossy(data).to_string())
            }
            EbmlElementType::Date => todo!(),
            EbmlElementType::Binary => EbmlElementValue::Binary(data.to_owned()),
            EbmlElementType::Master => unreachable!(),
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
    pub fn children(&self) -> std::io::Result<EbmlIterator<S, R>> {
        let mut r = self.reader.borrow_mut();
        r.seek(SeekFrom::Start(self.data_offset))?;
        Ok(EbmlIterator::new_with_end(
            self.reader.clone(),
            self.data_offset + self.size,
        ))
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
}

impl<S: EbmlSpec> ValueElement<S> {
    pub fn value(&self) -> EbmlElementValue {
        self.element.kind().to_value(&self.data)
    }
}

macro_rules! declare_elements {
    ($name:ident, $($k:ident($id:literal, $ty:expr)),+) => {
        #[derive(Debug)]
        pub enum $name {
            $($k,)+
            Unknown(EbmlId),
        }

        impl EbmlSpec for $name {
            fn from_id(id: EbmlId) -> Self {
                match id.0 {
                    $($id => Self::$k,)+
                    _ => Self::Unknown(id),
                }
            }

            fn id(&self) -> EbmlId {
                match self {
                    $(Self::$k => EbmlId($id),)+
                    Self::Unknown(id) => *id,
                }
            }

            fn name(&self) -> &'static str {
                match self {
                    $(Self::$k => stringify!($k),)+
                    Self::Unknown(_) => "Unknown",
                }
            }

            fn kind(&self) -> EbmlElementType {
                match self {
                    $(Self::$k => $ty,)+
                    Self::Unknown(_) => EbmlElementType::Binary,
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

// Matroska Schema
// https://datatracker.ietf.org/doc/html/rfc9559#name-matroska-schema
// MkvSegment(0x18538067, ElementType::Master),
// MkvSeekHead(0x114D9B74, ElementType::Master),
// MkvSeek(0x4DBB, ElementType::Master),
// MkvSeekId(0x53AB, ElementType::Binary),
// MkvSeekPosition(0x53AC, ElementType::UnsignedInteger),
// MkvInfo(0x1549A966, ElementType::Master),
// MkvSegmentUuid(0x73A4, ElementType::Binary),
// MkvSegmentFilename(0x7384, ElementType::Utf8),
// MkvPrevUUID(0x3CB923, ElementType::Binary),
// MkvPrevFilename(0x3C83AB, ElementType::Utf8),
// MkvNextUUID(0x3EB923, ElementType::Binary),
// MkvNextFilename(0x3E83BB, ElementType::Utf8),
// MkvSegmentFamily(0x4444, ElementType::Binary),
// MkvChapterTranslate(0x6924, ElementType::Master),
// MkvChapterTranslateID(0x69a5, ElementType::Binary),
// MkvChapterTranslateCodec(0x69BF, ElementType::UnsignedInteger),
// MkvChapterTranslateEditionUID(0x69FC, ElementType::UnsignedInteger),
// MkvTimestampScale(0x2AD7B1, ElementType::UnsignedInteger),
// MkvDuration(0x4489, ElementType::Float),
// MkvDateUTC(0x4461, ElementType::Date),
// MkvTitle(0x7BA9, ElementType::Utf8),
// MkvMuxingApp(0x4D80, ElementType::Utf8),
// MkvWritingApp(0x5741, ElementType::Utf8),
// MkvCluster(0x1F43B675, ElementType::Master),
// MkvTimestamp(0xE7, ElementType::UnsignedInteger),
// MkvPosition(0xA7, ElementType::UnsignedInteger),
// MkvPrevSize(0xAB, ElementType::UnsignedInteger),
// MkvSimpleBlock(0xA3, ElementType::Binary),
// MkvBlockGroup(0xA0, ElementType::Master),
// MkvBlock(0xA1, ElementType::Binary),
// MkvBlockAdditions(0x75A1, ElementType::Master),
// MkvBlockMore(0xA6, ElementType::Master),
// MkvBlockAdditional(0xA5, ElementType::Binary),
// MkvBlockAddID(0xEE, ElementType::UnsignedInteger),
// MkvBlockDuration(0x9B, ElementType::UnsignedInteger),
// MkvReferencePriority(0xFA, ElementType::UnsignedInteger),
// MkvReferenceBlock(0xFB, ElementType::SignedInteger),
// MkvCodecState(0xA4, ElementType::Binary),
// MkvDiscardPadding(0x75A2, ElementType::SignedInteger),
// MkvTracks(0x1654AE6B, ElementType::Master),
// MkvTrackEntry(0xAE, ElementType::Master),
// MkvTrackNumber(0xD7, ElementType::UnsignedInteger),
// MkvTrackUID(0x73C5, ElementType::UnsignedInteger),
// MkvTrackType(0x83, ElementType::UnsignedInteger),
// MkvFlagEnabled(0xB9, ElementType::UnsignedInteger),
// MkvFlagDefault(0x88, ElementType::UnsignedInteger),
// MkvFlagForced(0x55AA, ElementType::UnsignedInteger),
// MkvFlagHearingImpaired(0x55AB, ElementType::UnsignedInteger),
// MkvFlagVisualImpaired(0x55AC, ElementType::UnsignedInteger),
// MkvFlagTextDescriptions(0x55AD, ElementType::UnsignedInteger),
// MkvFlagOriginal(0x55AE, ElementType::UnsignedInteger),
// MkvFlagCommentary(0x55AF, ElementType::UnsignedInteger),
// MkvFlagLacing(0x9C, ElementType::UnsignedInteger),
// MkvDefaultDuration(0x23E383, ElementType::UnsignedInteger),
// MkvDefaultDecodedFieldDuration(0x234E7A, ElementType::UnsignedInteger),
// MkvTrackTimestampScale(0x23314F, ElementType::Float),
// MkvMaxBlockAdditionID(0x55EE, ElementType::UnsignedInteger),
// MkvBlockAdditionMapping(0x41E4, ElementType::Master),
// MkvBlockAddIDValue(0x41F0, ElementType::Master),
// MkvBlockAddIDName(0x41A4, ElementType::Master),
// MkvBlockAddIDType(0x41E7, ElementType::Master),
// MkvBlockAddIDExtraData(0x41ED, ElementType::Master),
// MkvName(0x536E, ElementType::Utf8),
// MkvLanguage(0x22B59C, ElementType::String),
// MkvLanguageBCP47(0x22B59D, ElementType::String),
// MkvCodecID(0x86, ElementType::String),
// MkvCodecPrivate(0x63A2, ElementType::Binary),
// MkvCodecName(0x258688, ElementType::Utf8),
// MkvAttachmentLink(0x7446, ElementType::UnsignedInteger),
// MkvCodecDelay(0x56AA, ElementType::UnsignedInteger),
// MkvSeekPreRoll(0x56BB, ElementType::UnsignedInteger),
// MkvTrackTranslate(0x6624, ElementType::Master),
// MkvTrackTranslateTrackID(0x66A5, ElementType::Binary),
// MkvTrackTranslateCodec(0x66BF, ElementType::UnsignedInteger),
// MkvTrackTranslateEditionUID(0x66FC, ElementType::UnsignedInteger),
// MkvVideo(0xE0, ElementType::Master),
// MkvFlagInterlaced(0x9A, ElementType::UnsignedInteger),
// MkvFieldOrder(0x9D, ElementType::UnsignedInteger),
// MkvStereoMode(0x53B8, ElementType::UnsignedInteger),
// MkvAlphaMode(0x53C0, ElementType::UnsignedInteger),
// MkvOldStereoMode(0x53B9, ElementType::UnsignedInteger),
// MkvPixelWidth(0xB0, ElementType::UnsignedInteger),
// MkvPixelHeight(0xBA, ElementType::UnsignedInteger),
// MkvPixelCropBottom(0x54AA, ElementType::UnsignedInteger),
// MkvPixelCropTop(0x54BB, ElementType::UnsignedInteger),
// MkvPixelCropLeft(0x54CC, ElementType::UnsignedInteger),
// MkvPixelCropRight(0x54DD, ElementType::UnsignedInteger),
// MkvDisplayWidth(0x54B0, ElementType::UnsignedInteger),
// MkvDisplayHeight(0x54BA, ElementType::UnsignedInteger),
// MkvDisplayUnit(0x54B2, ElementType::UnsignedInteger),
// MkvUncompressedFourCC(0x2EB524, ElementType::UnsignedInteger),
// MkvColour(0x55B0, ElementType::Master),
// MkvMatrixCoefficients(0x55B1, ElementType::UnsignedInteger),
// MkvBitsPerChannel(0x55B2, ElementType::UnsignedInteger),
// MkvChromaSubsamplingHorz(0x55B3, ElementType::UnsignedInteger),
// MkvChromaSubsamplingVert(0x55B4, ElementType::UnsignedInteger),
// MkvCbSubsamplingHorz(0x55B5, ElementType::UnsignedInteger),
// MkvCbSubsamplingVert(0x55B6, ElementType::UnsignedInteger),
// MkvChromaSitingHorz(0x55B7, ElementType::UnsignedInteger),
// MkvChromaSitingVert(0x55B8, ElementType::UnsignedInteger),
// MkvColourRange(0x55B9, ElementType::UnsignedInteger),
// MkvTransferCharacteristics(0x55BA, ElementType::UnsignedInteger),
// MkvPrimaries(0x55BB, ElementType::UnsignedInteger),
// MkvMaxCLL(0x55BC, ElementType::UnsignedInteger),
// MkvMaxFALL(0x55BD, ElementType::UnsignedInteger),
// MkvMasteringMetadata(0x55D0, ElementType::Master),
// MkvPrimaryRChromaticityX(0x55D1, ElementType::Float),
// MkvPrimaryRChromaticityY(0x55D2, ElementType::Float),
// MkvPrimaryGChromaticityX(0x55D3, ElementType::Float),
// MkvPrimaryGChromaticityY(0x55D4, ElementType::Float),
// MkvPrimaryBChromaticityX(0x55D5, ElementType::Float),
// MkvPrimaryBChromaticityY(0x55D6, ElementType::Float),
// MkvWhitePointChromaticityX(0x55D7, ElementType::Float),
// MkvWhitePointChromaticityY(0x55D8, ElementType::Float),
// MkvLuminanceMax(0x55D9, ElementType::Float),
// MkvLuminanceMin(0x55DA, ElementType::Float),
// MkvProjection(0x7670, ElementType::Master),
// MkvProjectionType(0x7671, ElementType::UnsignedInteger),
// MkvProjectionPrivate(0x7672, ElementType::Binary),
// MkvProjectionPoseYaw(0x7673, ElementType::Float),
// MkvProjectionPosePitch(0x7674, ElementType::Float),
// MkvProjectionPoseRoll(0x7675, ElementType::Float),
// MkvAudio(0xE1, ElementType::Master),
// MkvSamplingFrequency(0xB5, ElementType::Float),
// MkvOutputSamplingFrequency(0x78B5, ElementType::Float),
// MkvChannels(0x9F, ElementType::UnsignedInteger),
// MkvBitDepth(0x6264, ElementType::UnsignedInteger),
// MkvTrackOperation(0xE2, ElementType::Master),
// MkvTrackCombinePlanes(0xE3, ElementType::Master),
// MkvTrackPlane(0xE4, ElementType::Master),
// MkvTrackPlaneUID(0xE5, ElementType::UnsignedInteger),
// MkvTrackPlaneType(0xE6, ElementType::UnsignedInteger),
// MkvTrackJoinBlocks(0xE9, ElementType::Master),
// MkvTrackJoinUID(0xED, ElementType::UnsignedInteger),
// MkvContentEncoding(0x6240, ElementType::Master),
// MkvContentEncodingOrder(0x5031, ElementType::UnsignedInteger),
// MkvContentEncodingScope(0x5032, ElementType::UnsignedInteger),
// MkvContentEncodingType(0x5033, ElementType::UnsignedInteger),
// MkvContentCompression(0x5034, ElementType::Master),
// MkvContentCompAlgo(0x4254, ElementType::UnsignedInteger),
// MkvContentCompSettings(0x4255, ElementType::Binary),
// MkvContentEncryption(0x5035, ElementType::Master),
// MkvContentEncAlgo(0x47E1, ElementType::UnsignedInteger),
// MkvContentEncKeyID(0x47E2, ElementType::Binary),
// MkvContentEncAESSettings(0x47E7, ElementType::Master),
// MkvAESSettingsCipherMode(0x47E8, ElementType::UnsignedInteger),
// MkvCues(0x1C53BB6B, ElementType::Master),
// MkvCuePoint(0xBB, ElementType::Master),
// MkvCueTime(0xB3, ElementType::UnsignedInteger),
// MkvCueTrackPositions(0xB7, ElementType::Master),
// MkvCueTrack(0xF7, ElementType::UnsignedInteger),
// MkvCueClusterPosition(0xF1, ElementType::UnsignedInteger),
// MkvCueRelativePosition(0xF0, ElementType::UnsignedInteger),
// MkvCueDuration(0xB2, ElementType::UnsignedInteger),
// MkvCueBlockNumber(0x5378, ElementType::UnsignedInteger),
// MkvCueCodecState(0xEA, ElementType::UnsignedInteger),
// MkvCueReference(0xDB, ElementType::Master),
// MkvCueRefTime(0x96, ElementType::UnsignedInteger),
// MkvAttachments(0x1941A469, ElementType::Master),
// MkvAttachedFile(0x61A7, ElementType::Master),
// MkvFileDescription(0x467E, ElementType::Utf8),
// MkvFileName(0x466E, ElementType::Utf8),
// MkvFileMediaType(0x4660, ElementType::String),
// MkvFileData(0x465C, ElementType::Binary),
// MkvFileUID(0x46AE, ElementType::UnsignedInteger),
// MkvChapters(0x1043A770, ElementType::Master),
// MkvEditionEntry(0x45B9, ElementType::Master),
// MkvEditionUID(0x45BC, ElementType::UnsignedInteger),
// MkvEditionFlagDefault(0x45DB, ElementType::UnsignedInteger),
// MkvEditionFlagOrdered(0x45DD, ElementType::UnsignedInteger),
// MkvChapterAtom(0xB6, ElementType::Master),
// MkvChapterUID(0x73C4, ElementType::UnsignedInteger),
// MkvChapterStringUID(0x5654, ElementType::Utf8),
// MkvChapterTimeStart(0x91, ElementType::UnsignedInteger),
// MkvChapterTimeEnd(0x92, ElementType::UnsignedInteger),
// MkvChapterFlagHidden(0x98, ElementType::UnsignedInteger),
// MkvChapterSegmentUID(0x6E67, ElementType::Binary),
// MkvChapterSegmentEditionUID(0x6EBC, ElementType::UnsignedInteger),
// MkvChapterPhysicalEquiv(0x63C3, ElementType::UnsignedInteger),
// MkvChapterDisplay(0x80, ElementType::Master),
// MkvChapString(0x85, ElementType::Utf8),
// MkvChapLanguage(0x437C, ElementType::String),
// MkvChapLanguageBCP47(0x437D, ElementType::String),
// MkvChapCountry(0x437E, ElementType::String),
// MkvChapProcess(0x6944, ElementType::Master),
// MkvChapProcessCodecID(0x6955, ElementType::UnsignedInteger),
// MkvChapProcessPrivate(0x450D, ElementType::Binary),
// MkvChapProcessCommand(0x6911, ElementType::Master),
// MkvChapProcessTime(0x6922, ElementType::UnsignedInteger),
// MkvChapProcessData(0x6933, ElementType::Binary),
// MkvTags(0x1254C367, ElementType::Master),
// MkvTag(0x7373, ElementType::Master),
// MkvTargets(0x63C0, ElementType::Master),
// MkvTargetTypeValue(0x68CA, ElementType::UnsignedInteger),
// MkvTargetType(0x63CA, ElementType::String),
// MkvTagTrackUID(0x63C5, ElementType::UnsignedInteger),
// MkvTagEditionUID(0x63C9, ElementType::UnsignedInteger),
// MkvTagChapterUID(0x63C4, ElementType::UnsignedInteger),
// MkvTagAttachmentUID(0x63C6, ElementType::UnsignedInteger),
// MkvSimpleTag(0x67C8, ElementType::Master),
// MkvTagName(0x45A3, ElementType::Utf8),
// MkvTagLanguage(0x447A, ElementType::String),
// MkvTagLanguageBCP47(0x447B, ElementType::String),
// MkvTagDefault(0x4484, ElementType::UnsignedInteger),
// MkvTagString(0x4487, ElementType::Utf8),
// MkvTagBinary(0x4485, ElementType::Binary),
// // Matroska Deprecated Elements
// // https://datatracker.ietf.org/doc/html/rfc9559#name-historic-deprecated-element
// MkvSilentTracks(0x5854, ElementType::Master),
// MkvSilentTrackNumber(0x58D7, ElementType::UnsignedInteger),
// MkvBlockVirtual(0xA2, ElementType::Binary),
// MkvReferenceVirtual(0xFD, ElementType::SignedInteger),
// MkvSlices(0x8E, ElementType::Master),
// MkvTimeSlice(0xE8, ElementType::Master),
// MkvLaceNumber(0xCC, ElementType::UnsignedInteger),
// MkvFrameNumber(0xCD, ElementType::UnsignedInteger),
// MkvBlockAdditionID(0xCB, ElementType::UnsignedInteger),
// MkvDelay(0xCE, ElementType::UnsignedInteger),
// MkvSliceDuration(0xCF, ElementType::UnsignedInteger),
// MkvReferenceFrame(0xC8, ElementType::Master),
// MkvReferenceOffset(0xC9, ElementType::UnsignedInteger),
// MkvReferenceTimeCode(0xCA, ElementType::UnsignedInteger),
// MkvEncryptedBlock(0xAF, ElementType::Binary),
// MkvMinCache(0x6DE7, ElementType::UnsignedInteger),
// MkvMaxCache(0x6DF8, ElementType::UnsignedInteger),
// MkvTrackOffset(0x537F, ElementType::SignedInteger),
// MkvCodecSettings(0x3A9697, ElementType::Utf8),
// MkvCodecInfoURL(0x3B4040, ElementType::String),
// MkvCodecDownloadURL(0x26B240, ElementType::String),
// MkvCodecDecodeAll(0xAA, ElementType::UnsignedInteger),
// MkvTrackOverlay(0x6fab, ElementType::UnsignedInteger),
// MkvAspectRatioType(0x54B3, ElementType::UnsignedInteger),
// MkvFrameRate(0x2383E3, ElementType::Float),
// MkvChannelPositions(0x7D7B, ElementType::Binary),
// MkvTrickTrackUID(0xC0, ElementType::UnsignedInteger),
// MkvTrickTrackSegmentUID(0xC1, ElementType::Binary),
// MkvTrickTrackFlag(0xC6, ElementType::UnsignedInteger),
// MkvTrickMasterTrackUID(0xC7, ElementType::UnsignedInteger),
// MkvTrickMasterTrackSegmentUID(0xC4, ElementType::Binary),
// MkvContentSignature(0x47E3, ElementType::Binary),
// MkvContentSigKeyID(0x47E4, ElementType::Binary),
// MkvContentSigAlgo(0x47E5, ElementType::UnsignedInteger),
// MkvContentSigHashAlgo(0x47E6, ElementType::UnsignedInteger),
// MkvCueRefCluster(0x97, ElementType::UnsignedInteger),
// MkvCueRefNumber(0x535F, ElementType::UnsignedInteger),
// MkvCueRefCodecState(0xEB, ElementType::UnsignedInteger),
// MkvFileUsedStartTime(0x4661, ElementType::UnsignedInteger),
// MkvFileUsedEndTime(0x4662, ElementType::UnsignedInteger),
// MkvTagDefaultBogus(0x44B4, ElementType::UnsignedInteger)

impl std::fmt::Debug for EbmlId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("EbmlId(0x{:x})", self.0))
    }
}
