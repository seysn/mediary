use std::io::{BufRead, Seek, SeekFrom};

use byteorder::{BigEndian, ByteOrder};

use crate::{
    error::JpegResult,
    marker::{
        App1, App2, Comment, DefineQuantizationTable, HuffmanTable, Jfif, Marker, MarkerId,
        StartOfFrame, StartOfScan,
    },
    RawJpeg,
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
                Marker::DQT(DefineQuantizationTable(quantization_table)) => {
                    jpeg.quantization_tables.push(quantization_table)
                }
                Marker::APP0(jfif) => jpeg.jfif = Some(jfif),
                Marker::APP1(App1::Exif(exif)) => jpeg.exif = Some(exif),
                Marker::APP1(App1::Xmp(xmp)) => jpeg.xmp = Some(xmp),
                Marker::APP2(_app2) => (),
                Marker::SOS(start_of_scan) => jpeg.start_of_scan = Some(start_of_scan),
                Marker::COM(Comment(data)) => jpeg.comments.push(data),
                Marker::IGN(_) => (),
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

            let marker = match MarkerId::try_from(b) {
                Ok(marker) => marker,
                Err(err) => return Some(Err(err)),
            };

            match marker {
                MarkerId::SOF(_) => {
                    return Some(StartOfFrame::from_reader(&mut self.reader).map(Marker::SOF));
                }
                MarkerId::SOI => (),
                MarkerId::EOI => {
                    return None;
                }
                MarkerId::DQT => {
                    return Some(
                        DefineQuantizationTable::from_reader(&mut self.reader).map(Marker::DQT),
                    );
                }
                MarkerId::DHT => {
                    return Some(HuffmanTable::from_reader(&mut self.reader).map(Marker::DHT));
                }
                MarkerId::APP(0) => {
                    return Some(Jfif::from_reader(&mut self.reader).map(Marker::APP0));
                }
                MarkerId::APP(1) => {
                    return Some(App1::from_reader(&mut self.reader).map(Marker::APP1));
                }
                MarkerId::APP(2) => {
                    return Some(App2::from_reader(&mut self.reader).map(Marker::APP2));
                }
                MarkerId::SOS => match StartOfScan::from_reader(&mut self.reader) {
                    Ok(sos) => {
                        // We do not have the exact size of image data but SOS is usually
                        // followed by a EOI marker which is 2 bytes long
                        if let Err(err) = self.reader.seek(SeekFrom::End(-2)) {
                            return Some(Err(err.into()));
                        }

                        return Some(Ok(Marker::SOS(sos)));
                    }
                    Err(err) => return Some(Err(err)),
                },
                MarkerId::COM => {
                    return Some(Comment::from_reader(&mut self.reader).map(Marker::COM));
                }
                _ => {
                    let length = match read_u16(&mut self.reader) {
                        Ok(length) => length,
                        Err(err) => return Some(Err(err)),
                    };

                    let rest_size = length - 2;
                    if rest_size > 0 {
                        if let Err(err) = self.reader.seek(SeekFrom::Current(rest_size as i64)) {
                            return Some(Err(err.into()));
                        }
                    }

                    return Some(Ok(Marker::IGN(marker)));
                }
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
