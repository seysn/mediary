use std::io::Write;

use crate::{PnmEncoding, PnmFormat, PnmImage, error::PnmResult};

pub struct PnmWriter<W: Write> {
    image: PnmImage,
    writer: W,
}

impl<W: Write> PnmWriter<W> {
    pub fn new(image: PnmImage, writer: W) -> Self {
        Self { image, writer }
    }

    pub fn write(mut self) -> PnmResult<()> {
        let encoding = self.image.format.encoding();

        self.writer.write_all(&self.image.format.to_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer
            .write_all(self.image.width.to_string().as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer
            .write_all(self.image.height.to_string().as_bytes())?;
        self.writer.write_all(b"\n")?;

        if !matches!(self.image.format, PnmFormat::P1 | PnmFormat::P4) {
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

            if matches!(encoding, PnmEncoding::Ascii) {
                self.writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }
}

impl PnmEncoding {
    fn write<W: Write>(&self, value: u8, writer: &mut W) -> std::io::Result<()> {
        match self {
            PnmEncoding::Ascii => {
                writer.write_all(value.to_string().as_bytes())?;
                writer.write_all(b" ")?;
                Ok(())
            }
            PnmEncoding::Binary => writer.write_all(&[value]),
        }
    }
}
