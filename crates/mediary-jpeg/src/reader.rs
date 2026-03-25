use std::io::{BufRead, Seek};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    RawJpeg,
    error::JpegResult,
    marker::{App1, Comment, Marker, MarkerId},
};

pub struct JpegReader<R: BufRead + Seek> {
    reader: R,
}

impl<R: BufRead + Seek> JpegReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read(self) -> JpegResult<RawJpeg> {
        let mut jpeg = RawJpeg::default();

        for marker in self {
            match marker? {
                Marker::SOF(start_of_frame) => jpeg.start_of_frame = Some(start_of_frame),
                Marker::DHT(huffman_table) => jpeg.huffman_tables.push(huffman_table),
                Marker::DQT(quantization_tables) => {
                    jpeg.quantization_tables.push(quantization_tables);
                }
                Marker::APP0(jfif) => jpeg.jfif = Some(jfif),
                Marker::APP1(App1::Exif(exif)) => jpeg.exif = Some(exif),
                Marker::APP1(App1::Xmp(xmp)) => jpeg.xmp = Some(xmp),
                Marker::APP2(_app2) => (),
                Marker::SOS(start_of_scan) => jpeg.start_of_scan = Some(start_of_scan),
                Marker::COM(Comment(data)) => jpeg.comments.push(data),
                Marker::SOI | Marker::EOI | Marker::IGN(_) => (),
            }
        }

        Ok(jpeg)
    }
}

impl<R: BufRead + Seek> Iterator for JpegReader<R> {
    type Item = JpegResult<Marker>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let b = match read_u8(&mut self.reader) {
                Ok(b) => b,
                Err(err) => return Some(Err(err)),
            };

            if b != 0xff {
                continue;
            }

            let b = match read_u8(&mut self.reader) {
                Ok(b) => b,
                Err(err) => return Some(Err(err)),
            };

            let id = match MarkerId::try_from(b) {
                Ok(id) => id,
                Err(err) => return Some(Err(err)),
            };

            match Marker::from_reader(id, &mut self.reader) {
                Ok(Marker::SOI) => continue,
                Ok(Marker::EOI) => return None,
                Ok(marker) => return Some(Ok(marker)),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

pub(crate) fn read_u8<R: BufRead>(reader: &mut R) -> JpegResult<u8> {
    let mut data = [0];
    reader.read_exact(&mut data)?;
    Ok(data[0])
}

pub(crate) fn read_u16<R: BufRead>(reader: &mut R) -> JpegResult<u16> {
    let mut data = [0; 2];
    reader.read_exact(&mut data)?;
    Ok(BigEndian::read_u16(&data))
}
