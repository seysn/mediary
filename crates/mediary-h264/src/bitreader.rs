use crate::error::{H264Error, H264Result};

pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bit(&mut self) -> H264Result<u8> {
        if self.byte_pos >= self.data.len() {
            return Err(H264Error::UnexpectedEof);
        }

        let byte = self.data[self.byte_pos];
        let bit = (byte >> (7 - self.bit_pos)) & 1;

        self.bit_pos += 1;
        if self.bit_pos >= 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }

        Ok(bit)
    }

    pub fn read_flag(&mut self) -> H264Result<bool> {
        self.read_bit().map(|v| v == 1)
    }

    pub fn read_bits(&mut self, n: u8) -> H264Result<u32> {
        let mut result = 0;
        for _ in 0..n {
            result <<= 1;
            result |= self.read_bit()? as u32;
        }
        Ok(result)
    }

    pub fn read_ue(&mut self) -> H264Result<u32> {
        let mut zeros = 0;
        while self.read_bit()? == 0 {
            zeros += 1;
        }
        let suffix = self.read_bits(zeros)?;
        Ok((1 << zeros) - 1 + suffix)
    }

    pub fn read_se(&mut self) -> H264Result<i32> {
        let code_num = self.read_ue()? as i32;
        Ok(if code_num % 2 == 0 {
            -(code_num / 2)
        } else {
            (code_num + 1) / 2
        })
    }
}
