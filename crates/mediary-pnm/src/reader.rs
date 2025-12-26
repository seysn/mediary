use std::io::BufRead;

use crate::{PnmEncoding, PnmFormat, PnmImage, error::PnmResult};

pub struct PnmReader<R: BufRead> {
    reader: R,
}

impl<R: BufRead> PnmReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read(&mut self) -> PnmResult<PnmImage> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;

        // Consume newline
        let _ = read_byte(&mut self.reader)?;

        let format = PnmFormat::new(&buf)?;
        let width = read_ascii_number(&mut self.reader)? as usize;
        let height = read_ascii_number(&mut self.reader)? as usize;
        if !matches!(format, PnmFormat::P1 | PnmFormat::P4) {
            let _max = read_ascii_number(&mut self.reader)?;
        }

        let mut data = vec![0; width * height * format.byte_per_pixel()];
        let encoding = format.encoding();
        for subpixel in &mut data {
            *subpixel = encoding.read(&mut self.reader)?;
        }

        Ok(PnmImage {
            format,
            width,
            height,
            data,
        })
    }
}

fn read_ascii_number<R: BufRead>(reader: &mut R) -> PnmResult<u32> {
    let mut res = 0;

    loop {
        let ch = char::from(read_byte(reader)?);
        if ch.is_whitespace() {
            break;
        }

        res *= 10;
        res += ch.to_digit(10).unwrap();
    }

    Ok(res)
}

fn read_byte<R: BufRead>(reader: &mut R) -> PnmResult<u8> {
    let mut buf = [0];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

impl PnmEncoding {
    fn read<R: BufRead>(&self, reader: &mut R) -> PnmResult<u8> {
        match self {
            PnmEncoding::Ascii => Ok(read_ascii_number(reader)?.try_into().unwrap()),
            PnmEncoding::Binary => read_byte(reader),
        }
    }
}
