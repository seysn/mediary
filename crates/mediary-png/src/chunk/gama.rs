pub const ID: [u8; 4] = *b"gAMA";

#[derive(Debug)]
pub struct ImageGamma(pub u64);

impl ImageGamma {
    pub fn parse(bytes: &[u8]) -> Self {
        Self(u32::from_be_bytes(bytes.try_into().unwrap()) as u64 * 100000)
    }
}
