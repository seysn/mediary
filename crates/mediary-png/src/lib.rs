mod bitreader;
pub mod chunk;
pub mod decoder;
pub mod error;
mod inflate;
pub mod reader;
pub mod zlib;

pub const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
