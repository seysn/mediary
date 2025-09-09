use std::{collections::HashMap, io::BufRead};

use crate::bitreader::BitReader;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct HuffmanCode {
    pub code: u8,
    pub size: u8,
}

#[derive(Debug)]
pub struct HuffmanTable {
    codes: HashMap<HuffmanCode, u8>,
    max_size: u8,
}

impl HuffmanTable {
    pub fn new(sizes: Vec<u8>, codes: Vec<u8>, values: Vec<u8>) -> Self {
        assert_eq!(sizes.len(), codes.len());
        assert_eq!(codes.len(), values.len());

        let max_size = *sizes.iter().max().unwrap();

        let mut codes_map = HashMap::new();
        for ((&size, &code), &value) in sizes.iter().zip(&codes).zip(&values) {
            codes_map.insert(HuffmanCode { code, size }, value);
        }

        Self {
            codes: codes_map,
            max_size,
        }
    }

    /// Decode one byte from data
    pub fn decode_one<R: BufRead>(&self, bitreader: &mut BitReader<R>) -> u8 {
        let mut code: u8 = 0;

        for size in 1..=self.max_size {
            let bit = bitreader.read_bit().unwrap();
            code = (code << 1) | bit;

            if let Some(value) = self.codes.get(&HuffmanCode { code, size }) {
                return *value;
            }
        }

        panic!("Invalid Huffman code")
    }

    /// Decode data and return it when we have a certain amount of decoded values
    pub fn decode_n<R: BufRead>(&self, bitreader: &mut BitReader<R>, n_values: usize) -> Vec<u8> {
        let mut res = Vec::new();

        while res.len() != n_values {
            let mut code: u8 = 0;
            for size in 1..=self.max_size {
                let bit = bitreader.read_bit().unwrap();
                code = (code << 1) | bit;

                if let Some(value) = self.codes.get(&HuffmanCode { code, size }) {
                    res.push(*value);
                    break;
                }
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let table = HuffmanTable::new(
            vec![2, 2, 2, 3],
            vec![0b00, 0b01, 0b10, 0b110],
            vec![b'H', b'e', b'l', b'o'],
        );

        let mut encoded = BitReader::with_slice(&[0b0001_1010, 0b1100_0000]);
        let decoded = b"Hello";

        assert_eq!(table.decode_n(&mut encoded, decoded.len()), decoded);
    }
}
