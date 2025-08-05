use std::io;

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

    /// Read a bit in the form of a byte. The value should either be 0 or 1.
    fn read_bit(&mut self) -> io::Result<u8> {
        if self.byte_pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bits in BitReader data",
            ));
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

    /// Read a bit in a form of a boolean
    pub fn read_flag(&mut self) -> io::Result<bool> {
        self.read_bit().map(|v| v == 1)
    }

    /// Read multiple bits at once
    pub fn read_bits(&mut self, n: u8) -> io::Result<u32> {
        let mut result = 0;
        for _ in 0..n {
            result <<= 1;
            result |= self.read_bit()? as u32;
        }
        Ok(result)
    }

    /// Read unsigned Exp-Golomb value
    pub fn read_ue(&mut self) -> io::Result<u32> {
        let mut zeros = 0;
        while self.read_bit()? == 0 {
            zeros += 1;
        }
        let suffix = self.read_bits(zeros)?;
        Ok((1 << zeros) - 1 + suffix)
    }

    /// Read signed Exp-Golomb value
    pub fn read_se(&mut self) -> io::Result<i32> {
        let code_num = self.read_ue()? as i32;
        Ok(if code_num % 2 == 0 {
            -(code_num / 2)
        } else {
            (code_num + 1) / 2
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_bit() {
        let mut r = BitReader::new(&[0]);
        for _ in 0..8 {
            assert_eq!(r.read_bit().unwrap(), 0);
        }
        assert!(r.read_bit().is_err());

        let mut r = BitReader::new(&[0xff]);
        for _ in 0..8 {
            assert_eq!(r.read_bit().unwrap(), 1);
        }
        assert!(r.read_bit().is_err());

        let mut r = BitReader::new(&[0b0101_0101]);
        for _ in 0..4 {
            assert_eq!(r.read_bit().unwrap(), 0);
            assert_eq!(r.read_bit().unwrap(), 1);
        }
        assert!(r.read_bit().is_err());

        let mut r = BitReader::new(&[0, 0xff, 0b0101_0101]);
        for _ in 0..8 {
            assert_eq!(r.read_bit().unwrap(), 0);
        }
        for _ in 0..8 {
            assert_eq!(r.read_bit().unwrap(), 1);
        }
        for _ in 0..4 {
            assert_eq!(r.read_bit().unwrap(), 0);
            assert_eq!(r.read_bit().unwrap(), 1);
        }
        assert!(r.read_bit().is_err());
    }

    #[test]
    fn read_flag() {
        let mut r = BitReader::new(&[0]);
        for _ in 0..8 {
            assert!(!r.read_flag().unwrap());
        }
        assert!(r.read_flag().is_err());

        let mut r = BitReader::new(&[0xff]);
        for _ in 0..8 {
            assert!(r.read_flag().unwrap());
        }
        assert!(r.read_flag().is_err());

        let mut r = BitReader::new(&[0b0101_0101]);
        for _ in 0..4 {
            assert!(!r.read_flag().unwrap());
            assert!(r.read_flag().unwrap());
        }
        assert!(r.read_flag().is_err());

        let mut r = BitReader::new(&[0, 0xff, 0b0101_0101]);
        for _ in 0..8 {
            assert!(!r.read_flag().unwrap());
        }
        for _ in 0..8 {
            assert!(r.read_flag().unwrap());
        }
        for _ in 0..4 {
            assert!(!r.read_flag().unwrap());
            assert!(r.read_flag().unwrap());
        }
        assert!(r.read_flag().is_err());
    }

    #[test]
    fn read_bits() {
        let mut r = BitReader::new(&[0xff]);
        assert_eq!(r.read_bits(0).unwrap(), 0);
        assert_eq!(r.read_bits(1).unwrap(), 1);
        assert_eq!(r.read_bits(2).unwrap(), 0b11);
        assert_eq!(r.read_bits(3).unwrap(), 0b111);
        assert!(r.read_bits(4).is_err());
        assert_eq!(r.read_bits(0).unwrap(), 0);

        let mut r = BitReader::new(&[0b0101_0101]);
        assert_eq!(r.read_bits(3).unwrap(), 0b010);
        assert_eq!(r.read_bits(3).unwrap(), 0b101);
        assert_eq!(r.read_bits(2).unwrap(), 0b01);

        let mut r = BitReader::new(&[0xff, 0b0101_0101]);
        assert_eq!(r.read_bits(12).unwrap(), 0xff0 + 0b0101);
        assert_eq!(r.read_bits(4).unwrap(), 0b0101);
    }

    #[test]
    fn read_ue() {
        let mut r = BitReader::new(&[0b1011_0100, 0b0111_0010, 0b1000_1011]);
        assert_eq!(r.read_ue().unwrap(), 0);
        assert_eq!(r.read_ue().unwrap(), 2);
        assert_eq!(r.read_ue().unwrap(), 1);
        assert_eq!(r.read_ue().unwrap(), 6);
        assert_eq!(r.read_ue().unwrap(), 4);
        assert_eq!(r.read_ue().unwrap(), 10);
    }

    #[test]
    fn read_se() {
        let mut r = BitReader::new(&[0b1011_0100, 0b0111_0010, 0b1000_1011]);
        assert_eq!(r.read_se().unwrap(), 0);
        assert_eq!(r.read_se().unwrap(), -1);
        assert_eq!(r.read_se().unwrap(), 1);
        assert_eq!(r.read_se().unwrap(), -3);
        assert_eq!(r.read_se().unwrap(), -2);
        assert_eq!(r.read_se().unwrap(), -5);
    }
}
