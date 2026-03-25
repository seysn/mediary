use std::io::{self, Write};

use crate::huffman::HuffmanCode;

pub struct BitWriter<W: Write> {
    writer: W,
    buffer: u32,
    bit_count: usize,
}

impl<W: Write> BitWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: 0,
            bit_count: 0,
        }
    }

    pub fn into_writer(self) -> W {
        self.writer
    }

    pub fn write_bits(&mut self, data: u32, size: usize) -> io::Result<()> {
        self.buffer <<= size;
        self.buffer |= data;
        self.bit_count += size;

        self.flush_full_bytes()?;

        Ok(())
    }

    pub fn write_code(&mut self, value: &HuffmanCode) -> io::Result<()> {
        self.write_bits(u32::from(value.code), usize::from(value.size))
    }

    fn flush_full_bytes(&mut self) -> io::Result<()> {
        while self.bit_count >= 8 {
            let byte = (self.buffer >> (self.bit_count - 8)) as u8;
            self.writer.write_all(&[byte])?;
            self.bit_count -= 8;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.flush_full_bytes()?;

        if self.bit_count > 0 {
            self.buffer <<= 8 - self.bit_count;
            self.writer.write_all(&[self.buffer as u8])?;
            self.bit_count = 0;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn write_byte() {
        let buffer = Cursor::new(Vec::new());
        let mut bitwriter = BitWriter::new(buffer);

        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.write_bits(0b010, 3).unwrap();
        bitwriter.flush().unwrap();

        let buffer = bitwriter.into_writer().into_inner();
        assert_eq!(buffer, [0b11011010]);
    }

    #[test]
    fn write_two_bytes() {
        let buffer = Cursor::new(Vec::new());
        let mut bitwriter = BitWriter::new(buffer);

        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.write_bits(0b010, 3).unwrap();
        bitwriter.write_bits(0b010, 3).unwrap();
        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.flush().unwrap();

        let buffer = bitwriter.into_writer().into_inner();
        assert_eq!(buffer, [0b01111010, 0b01001111]);
    }

    #[test]
    fn write_incomplete_byte() {
        let buffer = Cursor::new(Vec::new());
        let mut bitwriter = BitWriter::new(buffer);

        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.flush().unwrap();

        let buffer = bitwriter.into_writer().into_inner();
        assert_eq!(buffer, [0b11011000]);
    }

    #[test]
    fn write_two_incomplete_byte() {
        let buffer = Cursor::new(Vec::new());
        let mut bitwriter = BitWriter::new(buffer);

        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.write_bits(0b010, 3).unwrap();
        bitwriter.write_bits(0b11, 2).unwrap();
        bitwriter.write_bits(0b011, 3).unwrap();
        bitwriter.flush().unwrap();

        let buffer = bitwriter.into_writer().into_inner();
        assert_eq!(buffer, [0b01111010, 0b11011000]);
    }
}
