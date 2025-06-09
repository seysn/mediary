use std::fmt::Debug;

pub struct NalUnit<'a> {
    /// Nal Unit Type
    pub kind: NalType,

    /// Raw payload bytes (excluding the 1-byte NAL header)
    pub payload: &'a [u8],
}

pub struct NalUnitIterator<'a> {
    buf: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NalType {
    Unspecified = 0,
    Slice = 1,
    Dpa = 2,
    Dpb = 3,
    Dpc = 4,
    IdrSlice = 5,
    Sei = 6,
    Sps = 7,
    Pps = 8,
    Aud = 9,
    EndSequence = 10,
    EndStream = 11,
    FillerData = 12,
    SpsExt = 13,
    Prefix = 14,
    SubSps = 15,
    Dps = 16,
    Reserved17 = 17,
    Reserved18 = 18,
    AuxiliarySlice = 19,
    ExtenSlice = 20,
    DepthExtenSlice = 21,
    Reserved22 = 22,
    Reserved23 = 23,
    Unspecified24 = 24,
    Unspecified25 = 25,
    Unspecified26 = 26,
    Unspecified27 = 27,
    Unspecified28 = 28,
    Unspecified29 = 29,
    Unspecified30 = 30,
    Unspecified31 = 31,
}

impl<'a> NalUnit<'a> {
    pub fn from_raw(data: &'a [u8]) -> Self {
        Self {
            kind: data[0].into(),
            payload: &data[1..],
        }
    }

    pub fn from_avc1(data: &'a [u8]) -> Self {
        let len = u32::from_be_bytes(data[..4].try_into().expect("slice has fewer than 2 bytes"));

        Self {
            kind: data[4].into(),
            payload: &data[4..4 + len as usize],
        }
    }
}

impl From<u8> for NalType {
    fn from(value: u8) -> Self {
        match value & 0x1f {
            0 => Self::Unspecified,
            1 => Self::Slice,
            2 => Self::Dpa,
            3 => Self::Dpb,
            4 => Self::Dpc,
            5 => Self::IdrSlice,
            6 => Self::Sei,
            7 => Self::Sps,
            8 => Self::Pps,
            9 => Self::Aud,
            10 => Self::EndSequence,
            11 => Self::EndStream,
            12 => Self::FillerData,
            13 => Self::SpsExt,
            14 => Self::Prefix,
            15 => Self::SubSps,
            16 => Self::Dps,
            17 => Self::Reserved17,
            18 => Self::Reserved18,
            19 => Self::AuxiliarySlice,
            20 => Self::ExtenSlice,
            21 => Self::DepthExtenSlice,
            22 => Self::Reserved22,
            23 => Self::Reserved23,
            24 => Self::Unspecified24,
            25 => Self::Unspecified25,
            26 => Self::Unspecified26,
            27 => Self::Unspecified27,
            28 => Self::Unspecified28,
            29 => Self::Unspecified29,
            30 => Self::Unspecified30,
            31 => Self::Unspecified31,
            _ => panic!("Invalid NAL type"),
        }
    }
}

impl<'a> NalUnitIterator<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for NalUnitIterator<'a> {
    type Item = NalUnit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.is_empty() {
            None
        } else {
            let nal = NalUnit::from_avc1(self.buf);
            self.buf = &self.buf[4 + nal.payload.len()..];
            Some(nal)
        }
    }
}

impl Debug for NalUnit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NalUnit")
            .field("kind", &self.kind)
            .field("payload", &format!("[ {} bytes ]", self.payload.len()))
            .finish()
    }
}
