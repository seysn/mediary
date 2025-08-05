use std::{fs::File, io::BufReader, path::Path};

use exif::ExifData;
use marker::{HuffmanTable, Jfif, StartOfFrame, StartOfScan, XmpData};

pub mod error;
pub mod exif;
pub mod marker;
pub mod reader;

#[derive(Debug, Default, Clone)]
pub struct RawJpeg {
    pub start_of_frame: Option<StartOfFrame>,
    pub huffman_tables: Vec<HuffmanTable>,
    pub quantization_tables: Vec<Vec<u8>>,
    pub jfif: Option<Jfif>,
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub comments: Vec<String>,
    pub start_of_scan: Option<StartOfScan>,
}

impl RawJpeg {
    pub fn read<P: AsRef<Path>>(path: P) -> error::JpegResult<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        reader::JpegReader::new(reader).read()
    }
}
