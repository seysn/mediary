use std::slice;

pub struct BitReader<'a> {
    bytes: slice::Iter<'a, u8>,
    buffer: u32,
    size: u32,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.iter(),
            buffer: 0,
            size: 0,
        }
    }

    fn next_byte(&mut self) -> Option<u32> {
        match self.bytes.next() {
            Some(&b) => {
                self.buffer |= u32::from(b) << self.size;
                self.size += 8;
                Some(self.buffer)
            }
            None => None,
        }
    }

    pub fn read_bits(&mut self, n: u8) -> Option<u32> {
        while self.size < u32::from(n) {
            self.next_byte()?;
        }

        let mask = (1 << n) - 1;
        let value = self.buffer & mask;

        self.buffer >>= n;
        self.size -= u32::from(n);

        Some(value)
    }
}
