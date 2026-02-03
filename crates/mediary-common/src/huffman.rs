use std::{collections::HashMap, io};

use crate::bitreader::BitReader;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct HuffmanCode {
    code: u16,
    size: u8,
}

#[derive(Debug, Clone)]
pub struct HuffmanTable {
    lookup_table: HashMap<HuffmanCode, u8>,
    reverse_table: HashMap<u8, HuffmanCode>,
    max_size: u8,
}

impl HuffmanCode {
    pub const fn new(code: u16, size: u8) -> Self {
        Self { code, size }
    }
}

impl HuffmanTable {
    pub fn new(lookup_table: HashMap<HuffmanCode, u8>) -> io::Result<Self> {
        let max_size = lookup_table
            .keys()
            .max_by_key(|code| code.size)
            .ok_or(io::Error::from(io::ErrorKind::UnexpectedEof))?
            .size;

        let reverse_table = lookup_table.iter().map(|(k, v)| (*v, *k)).collect();

        Ok(Self {
            lookup_table,
            reverse_table,
            max_size,
        })
    }

    /// Decode one byte from data
    pub fn decode_one<R: io::BufRead>(&self, bitreader: &mut BitReader<R>) -> io::Result<u8> {
        let mut code: u16 = 0;

        for size in 1..=self.max_size {
            let bit = u16::from(bitreader.read_bit()?);
            code = (code << 1) | bit;

            if let Some(value) = self.lookup_table.get(&HuffmanCode { code, size }) {
                return Ok(*value);
            }
        }

        panic!("Invalid Huffman code")
    }

    /// Decode data and return it when we have a certain amount of decoded values
    pub fn decode_n<R: io::BufRead>(
        &self,
        bitreader: &mut BitReader<R>,
        n_values: usize,
    ) -> io::Result<Vec<u8>> {
        let mut res = Vec::new();

        while res.len() != n_values {
            let mut code: u16 = 0;
            for size in 1..=self.max_size {
                let bit = u16::from(bitreader.read_bit()?);
                code = (code << 1) | bit;

                if let Some(value) = self.lookup_table.get(&HuffmanCode { code, size }) {
                    res.push(*value);
                    break;
                }
            }
        }

        Ok(res)
    }

    pub fn encode(&self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let table = HuffmanTable::new(HashMap::from([
            (HuffmanCode::new(0b00, 2), b'H'),
            (HuffmanCode::new(0b01, 2), b'e'),
            (HuffmanCode::new(0b10, 2), b'l'),
            (HuffmanCode::new(0b110, 3), b'o'),
        ]))
        .unwrap();

        let mut encoded = BitReader::with_slice(&[0b0001_1010, 0b1100_0000]);
        let decoded = b"Hello";

        assert_eq!(
            table.decode_n(&mut encoded, decoded.len()).unwrap(),
            decoded
        );
    }
}
