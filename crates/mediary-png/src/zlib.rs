use crate::inflate::Inflate;

#[derive(Debug)]
pub enum CompressionMethod {
    Deflate,
    Unknown,
}

#[derive(Debug)]
pub enum CompressionLevel {
    Fastest,
    Fast,
    Default,
    Maximum,
}

#[derive(Debug)]
pub struct ZLibStream<'a> {
    pub header: ZLibHeader,
    pub stream: Inflate<'a>,
}

#[derive(Debug)]
pub struct ZLibHeader {
    pub compression_method: CompressionMethod,
    pub maximum_allowed_value: u32,
    pub fcheck: u32,
    pub fdict: bool,
    pub compression_level: CompressionLevel,
}

impl<'a> ZLibStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header = ZLibHeader::new(data);
        let stream = Inflate::new(&data[2..]);

        Self { header, stream }
    }

    pub fn read(&mut self) -> Vec<u8> {
        let mut output = Vec::new();
        while !self.stream.read_block(&mut output) {}
        output
    }
}

impl ZLibHeader {
    pub fn new(data: &[u8]) -> Self {
        let compression_method = CompressionMethod::new(data[0] & 0x0F);
        let compression_info = data[0] >> 4;

        Self {
            compression_method,
            maximum_allowed_value: 2_u32.pow(compression_info as u32 + 8),
            fcheck: (data[1] & 0x1F).into(),
            fdict: (data[1] & 0x20) > 0,
            compression_level: CompressionLevel::new(data[1] >> 6),
        }
    }
}

impl CompressionMethod {
    pub fn new(data: u8) -> Self {
        match data {
            8 => Self::Deflate,
            _ => Self::Unknown,
        }
    }
}

impl CompressionLevel {
    pub fn new(data: u8) -> Self {
        match data {
            0 => Self::Fastest,
            1 => Self::Fast,
            2 => Self::Default,
            3 => Self::Maximum,
            _ => Self::Default,
        }
    }
}
