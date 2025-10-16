use std::{fs::File, io::BufReader, path::Path};

use decoder::{Component, MAX_COMPONENTS};
use exif::ExifData;
use marker::{DefineHuffmanTable, Jfif, QuantizationTable, StartOfFrame, StartOfScan, XmpData};
use mediary_image::RgbImage;

pub use crate::error::{JpegError, JpegResult};

pub mod dct;
pub mod decoder;
pub mod error;
pub mod exif;
pub mod marker;
pub mod reader;

#[derive(Debug, Default, Clone)]
pub struct RawJpeg {
    pub start_of_frame: Option<StartOfFrame>,
    pub huffman_tables: Vec<DefineHuffmanTable>,
    pub quantization_tables: Vec<QuantizationTable>,
    pub jfif: Option<Jfif>,
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub comments: Vec<String>,
    pub start_of_scan: Option<StartOfScan>,
}

impl RawJpeg {
    pub fn read<P: AsRef<Path>>(path: P) -> JpegResult<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        reader::JpegReader::new(reader).read()
    }

    pub fn decode(&self) -> JpegResult<RgbImage> {
        let start_of_frame = self
            .start_of_frame
            .as_ref()
            .ok_or(JpegError::MissingMarker(marker::MarkerId::SOF(0)))?;

        let start_of_scan = self
            .start_of_scan
            .as_ref()
            .ok_or(JpegError::MissingMarker(marker::MarkerId::SOS))?;

        let mut h_max = 0;
        let mut v_max = 0;
        let mut components = Vec::new();
        for (sof, sos) in start_of_frame
            .components
            .iter()
            .zip(&start_of_scan.components)
        {
            h_max = h_max.max(sof.horizontal_sampling);
            v_max = v_max.max(sof.vertical_sampling);
            components.push(Component {
                id: sof.id,
                horizontal_sampling: sof.horizontal_sampling,
                vertical_sampling: sof.vertical_sampling,
                quantization_table: usize::from(sof.quantization_table),
                dc_table: usize::from(sos.dc_table),
                ac_table: usize::from(sos.dc_table),
            })
        }

        let width = start_of_frame.width;
        let height = start_of_frame.height;
        let mcu_width = width / (8 * u16::from(h_max));
        let mcu_height = height / (8 * u16::from(h_max));

        let mut dc_huffman_tables = [const { None }; MAX_COMPONENTS];
        let mut ac_huffman_tables = [const { None }; MAX_COMPONENTS];
        for dht in &self.huffman_tables {
            let idx = usize::from(dht.index);
            let table = dht.to_table()?;
            match dht.class {
                marker::TableClass::DC => dc_huffman_tables[idx] = Some(table),
                marker::TableClass::AC => ac_huffman_tables[idx] = Some(table),
            }
        }

        let mut quantization_tables = [const { None }; MAX_COMPONENTS];
        for qt in &self.quantization_tables {
            let idx = usize::from(qt.index);
            quantization_tables[idx] = Some(qt.values.clone());
        }

        let decoder = decoder::JpegDecoder {
            data: &start_of_scan.data.0,
            mcu_width,
            mcu_height,
            components,
            dc_huffman_tables,
            ac_huffman_tables,
            quantization_tables,
        };

        let mut output = RgbImage {
            data: vec![0; usize::from(width) * usize::from(height) * 3],
            width: usize::from(width),
            height: usize::from(height),
        };
        decoder.decode(&mut output)?;

        Ok(output)
    }
}
