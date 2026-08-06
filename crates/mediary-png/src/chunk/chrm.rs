pub const ID: [u8; 4] = *b"cHRM";

#[derive(Debug)]
pub struct PrimaryChromaticities {
    pub white_point_x: u32,
    pub white_point_y: u32,
    pub red_x: u32,
    pub red_y: u32,
    pub green_x: u32,
    pub green_y: u32,
    pub blue_x: u32,
    pub blue_y: u32,
}

impl PrimaryChromaticities {
    pub fn parse(bytes: &[u8]) -> Self {
        Self {
            white_point_x: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            white_point_y: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            red_x: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            red_y: u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
            green_x: u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            green_y: u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
            blue_x: u32::from_be_bytes(bytes[24..28].try_into().unwrap()),
            blue_y: u32::from_be_bytes(bytes[28..32].try_into().unwrap()),
        }
    }
}
