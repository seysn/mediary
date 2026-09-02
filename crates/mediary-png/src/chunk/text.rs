#[derive(Debug)]
pub struct TextualData(pub String);

impl TextualData {
    pub const ID: [u8; 4] = *b"tEXt";
    pub const STRING_ID: &str = "tEXt";

    pub fn parse(bytes: &[u8]) -> Self {
        Self(String::from_utf8_lossy(bytes).to_string())
    }
}
