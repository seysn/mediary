use std::io::Write;

use crate::{error::NetpbmResult, NetpbmFormat, NetpbmImage};

pub struct NetpbmWriter<W: Write> {
    image: NetpbmImage,
    writer: W,
}

pub enum NetpbmEncoding {
    Ascii,
    Binary,
}

impl<W: Write> NetpbmWriter<W> {
    pub fn new(image: NetpbmImage, writer: W) -> Self {
        Self { image, writer }
    }

    pub fn write(mut self) -> NetpbmResult<()> {
        let encoding = self.image.format.encoding();

        self.writer.write_all(&self.image.format.to_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer
            .write_all(self.image.width.to_string().as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer
            .write_all(self.image.height.to_string().as_bytes())?;
        self.writer.write_all(b"\n")?;

        if !matches!(self.image.format, NetpbmFormat::P1 | NetpbmFormat::P4) {
            let max = self.image.data.iter().max().unwrap_or(&255);

            self.writer.write_all(max.to_string().as_bytes())?;
            self.writer.write_all(b"\n")?;
        }

        for row in self
            .image
            .data
            .chunks(self.image.width * self.image.format.byte_per_pixel())
        {
            for pixel in row {
                encoding.write(*pixel, &mut self.writer)?;
            }

            if matches!(encoding, NetpbmEncoding::Ascii) {
                self.writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }
}

impl NetpbmEncoding {
    fn write<W: Write>(&self, value: u8, writer: &mut W) -> std::io::Result<()> {
        match self {
            NetpbmEncoding::Ascii => {
                writer.write_all(value.to_string().as_bytes())?;
                writer.write_all(b" ")?;
                Ok(())
            }
            NetpbmEncoding::Binary => writer.write_all(&[value]),
        }
    }
}

impl NetpbmFormat {
    pub fn encoding(&self) -> NetpbmEncoding {
        match self {
            NetpbmFormat::P1 | NetpbmFormat::P2 | NetpbmFormat::P3 => NetpbmEncoding::Ascii,
            NetpbmFormat::P4 | NetpbmFormat::P5 | NetpbmFormat::P6 => NetpbmEncoding::Binary,
        }
    }
}
