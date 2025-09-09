use std::io::Write;

use crate::{error::NetpbmResult, NetpbmFormat, NetpbmImage};

pub struct NetpbmWriter<W: Write> {
    image: NetpbmImage,
    writer: W,
}

impl<W: Write> NetpbmWriter<W> {
    pub fn new(image: NetpbmImage, writer: W) -> Self {
        Self { image, writer }
    }

    pub fn write(mut self) -> NetpbmResult<()> {
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
                self.writer.write_all(pixel.to_string().as_bytes())?;
                self.writer.write_all(b" ")?;
            }
            self.writer.write_all(b"\n")?;
        }

        Ok(())
    }
}
