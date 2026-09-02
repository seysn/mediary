use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use mediary_image::RgbImage;

use crate::{
    chunk::{BitDepth, ColorType, ImageData, PngChunk},
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

        let header = match self.reader.read_chunk()? {
            PngChunk::ImageHeader(header) => header,
            chunk => {
                return Err(PngError::UnexpectedChunk {
                    expected: "IHDR",
                    found: chunk.string_id(),
                });
            }
        };

        if header.interlace_method != 0 {
            todo!();
        }

        let mut palette = None;
        let mut image_data = Vec::new();
        loop {
            let chunk = self.reader.read_chunk()?;

            match chunk {
                PngChunk::ImageData(ImageData(data)) => {
                    image_data.extend(data);
                }
                PngChunk::Palette(palette_chunk) => {
                    palette = Some(palette_chunk);
                }
                PngChunk::ImageTrailer => break,
                _ => (),
            }
        }

        let mut zlib_stream = ZLibStream::new(&image_data);
        let inflated = zlib_stream.read();

        let mut output = Vec::with_capacity(
            header.width as usize * header.height as usize * header.color_type.channels(),
        );
        for row in inflated.chunks_exact(header.row_size()) {
            let filter_type = FilterType::new(row[0]);
            match filter_type {
                FilterType::None => match header.color_type {
                    ColorType::Greyscale => todo!(),
                    ColorType::Truecolor => {
                        output.extend(&row[1..]);
                    }
                    ColorType::IndexedColor => {
                        let palette = palette.as_ref().ok_or(PngError::MissingChunk("PLTE"))?;
                        let colors = &palette.colors;

                        for byte in &row[1..] {
                            match header.bit_depth {
                                BitDepth::One => todo!(),
                                BitDepth::Two => todo!(),
                                BitDepth::Four => {
                                    let color = colors
                                        .get((byte & 0xF0) as usize >> 4)
                                        .ok_or(PngError::InvalidChunkData { chunk_id: "PLTE" })?;
                                    output.push(color.red);
                                    output.push(color.green);
                                    output.push(color.blue);

                                    let color = colors
                                        .get((byte & 0x0F) as usize)
                                        .ok_or(PngError::InvalidChunkData { chunk_id: "PLTE" })?;
                                    output.push(color.red);
                                    output.push(color.green);
                                    output.push(color.blue);
                                }
                                BitDepth::Eight => todo!(),
                                BitDepth::Sixteen => todo!(),
                            }
                        }
                    }
                    ColorType::GreyscaleWithAlpha => todo!(),
                    ColorType::TrueColorWithAlpha => todo!(),
                },
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

        RgbImage::new(output, header.width as usize, header.height as usize)
            .ok_or(PngError::InvalidChunkData { chunk_id: "IDAT" })
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
