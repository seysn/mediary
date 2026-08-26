use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use mediary_image::RgbImage;

use crate::{
    chunk::{ColorType, ImageData, PngChunk},
    error::{PngError, PngResult},
    reader::PngReader,
    zlib::ZLibStream,
    SIGNATURE,
};

pub struct PngDecoder<R: Seek + BufRead> {
    reader: PngReader<R>,
}

#[derive(Debug)]
pub enum FilterType {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl PngDecoder<BufReader<File>> {
    pub fn with_file<P: AsRef<Path>>(path: P) -> PngResult<Self> {
        Ok(Self {
            reader: PngReader::new(BufReader::new(File::open(path)?)),
        })
    }
}

impl<R: Seek + BufRead> PngDecoder<R> {
    pub fn decode(&mut self) -> PngResult<RgbImage> {
        if self.reader.read_signature()? != SIGNATURE {
            return Err(PngError::InvalidSignature);
        }

        let PngChunk::ImageHeader(header) = self.reader.read_chunk()? else {
            todo!()
        };

        if header.interlace_method != 0 {
            todo!();
        }

        if header.bit_depth != 8 {
            todo!();
        }

        let mut image_data = Vec::new();
        loop {
            let chunk = self.reader.read_chunk()?;

            match chunk {
                PngChunk::ImageData(ImageData(data)) => {
                    image_data.extend(data);
                }
                PngChunk::ImageTrailer => break,
                _ => (),
            }
        }

        let mut zlib_stream = ZLibStream::new(&image_data);
        let inflated = zlib_stream.read();

        // Row size is type byte plus number of bytes in one row
        let row_size = 1 + (header.width as usize * header.color_type.channels());
        let mut output = Vec::with_capacity(
            header.width as usize * header.height as usize * header.color_type.channels(),
        );
        for row in inflated.chunks_exact(row_size) {
            let filter_type = FilterType::new(row[0]);
            match filter_type {
                FilterType::None => {
                    output.extend(&row[1..]);
                }
                FilterType::Sub => match header.color_type {
                    ColorType::Greyscale => todo!(),
                    ColorType::Truecolor => todo!(),
                    ColorType::IndexedColor => todo!(),
                    ColorType::GreyscaleWithAlpha => todo!(),
                    ColorType::TrueColorWithAlpha => todo!(),
                },
                FilterType::Up => todo!(),
                FilterType::Average => todo!(),
                FilterType::Paeth => todo!(),
            }
        }

        Ok(RgbImage::new(output, header.width as usize, header.height as usize).unwrap())
    }
}

impl FilterType {
    pub fn new(byte: u8) -> Self {
        match byte {
            0 => Self::None,
            1 => Self::Sub,
            2 => Self::Up,
            3 => Self::Average,
            4 => Self::Paeth,
            _ => todo!(),
        }
    }
}
