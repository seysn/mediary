use std::{fmt::Debug, iter::repeat_n};

use crate::bitreader::BitReader;

pub struct Inflate<'a> {
    bitreader: BitReader<'a>,
}

#[derive(Debug)]
enum BType {
    NoCompression,
    FixedHuffman,
    DynamicHuffman,
}

const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Debug)]
struct HuffmanTree {
    codes: Vec<HuffmanCode>,
}

#[derive(Default, Clone)]
struct HuffmanCode {
    code: usize,
    len: usize,
}

struct LengthCode {
    base: usize,
    extra: u8,
}

struct DistanceCode {
    base: usize,
    extra: u8,
}

impl<'a> Inflate<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            bitreader: BitReader::new(data),
        }
    }

    pub fn read_block(&mut self, output: &mut Vec<u8>) -> bool {
        let bfinal = self.bitreader.read_bits(1).unwrap() == 1;
        let btype = BType::new(self.bitreader.read_bits(2).unwrap());

        let (literal_tree, distance_tree) = match btype {
            BType::NoCompression => todo!(),
            BType::FixedHuffman => {
                const LITERAL: [u32; 288] = [
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 7, 7, 7, 7,
                    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8,
                    8, 8,
                ];

                const DISTANCE: [u32; 32] = [
                    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                    5, 5, 5, 5, 5, 5,
                ];

                let literal_tree = HuffmanTree::new(&LITERAL);
                let distance_tree = HuffmanTree::new(&DISTANCE);

                (literal_tree, distance_tree)
            }
            BType::DynamicHuffman => {
                // Read HLIT, HDIST, HCLEN
                let num_lit_codes = self.bitreader.read_bits(5).unwrap() as usize + 257;
                let num_dist_codes = self.bitreader.read_bits(5).unwrap() as usize + 1;
                let num_code_lengths = self.bitreader.read_bits(4).unwrap() as usize + 4;

                // Read code lengths
                // It represents the number of bits (value) for each symbol (index)
                let mut code_lengths = [0; 19];
                for i in 0..num_code_lengths {
                    code_lengths[CODE_LENGTH_ORDER[i]] = self.bitreader.read_bits(3).unwrap();
                }

                let reader = HuffmanTree::new(&code_lengths);

                let total_codes = num_lit_codes + num_dist_codes;
                let mut i = 0;
                let mut result = Vec::new();
                while i < total_codes {
                    let symbol = reader.decode_symbol(&mut self.bitreader);
                    match symbol {
                        0..=15 => {
                            result.push(symbol);
                            i += 1;
                        }
                        16 => {
                            let repeat_count = self.bitreader.read_bits(2).unwrap() + 3;
                            let last = *result.last().unwrap();
                            result.extend(repeat_n(last, repeat_count as usize));
                            i += repeat_count as usize;
                        }
                        17 => {
                            let repeat_count = self.bitreader.read_bits(3).unwrap() + 3;
                            result.extend(repeat_n(0, repeat_count as usize));
                            i += repeat_count as usize;
                        }
                        18 => {
                            let repeat_count = self.bitreader.read_bits(7).unwrap() + 11;
                            result.extend(repeat_n(0, repeat_count as usize));
                            i += repeat_count as usize;
                        }
                        _ => panic!("invalid symbol {symbol}"),
                    }
                }

                let (literal, distance) = result.split_at(num_lit_codes);
                let literal_tree = HuffmanTree::new(literal);
                let distance_tree = HuffmanTree::new(distance);

                (literal_tree, distance_tree)
            }
        };

        loop {
            let symbol = literal_tree.decode_symbol(&mut self.bitreader);
            match symbol {
                0..=255 => {
                    // Literal byte
                    output.push(symbol as u8);
                }
                256 => {
                    // End of block
                    break;
                }
                257..=285 => {
                    // Compressed block
                    let LengthCode { base, extra } = LengthCode::new(symbol);
                    let length = base
                        + if extra > 0 {
                            self.bitreader.read_bits(extra).unwrap() as usize
                        } else {
                            0
                        };

                    let dist_symbol = distance_tree.decode_symbol(&mut self.bitreader);
                    let DistanceCode { base, extra } = DistanceCode::new(dist_symbol);
                    let distance = base
                        + if extra > 0 {
                            self.bitreader.read_bits(extra).unwrap() as usize
                        } else {
                            0
                        };

                    let idx = output.len() - distance;
                    for i in 0..length {
                        output.push(output[idx + i]);
                    }
                }
                _ => panic!("invalid symbol {symbol}"),
            }
        }

        bfinal
    }
}

impl BType {
    pub fn new(value: u32) -> Self {
        match value {
            0b00 => Self::NoCompression,
            0b01 => Self::FixedHuffman,
            0b10 => Self::DynamicHuffman,
            _ => todo!("invalid btype 0b{value:b}"),
        }
    }
}

impl HuffmanTree {
    fn new(code_lengths: &[u32]) -> Self {
        // Count the number of codes for each code length
        let mut bit_length_count = [0_usize; 16];
        for &code_length in code_lengths {
            if code_length > 0 {
                bit_length_count[code_length as usize] += 1;
            }
        }

        // Find the numerical value of the smallest code for each code length
        let mut code = 0;
        let mut next_code = [0; 16];
        for bits in 1..16 {
            code = (code + bit_length_count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        // Assign numerical values to all codes
        let mut codes = vec![HuffmanCode { code: 0, len: 0 }; code_lengths.len()];
        for symbol in 0..code_lengths.len() {
            let len = code_lengths[symbol] as usize;
            if len > 0 {
                codes[symbol] = HuffmanCode {
                    code: next_code[len],
                    len,
                };
                next_code[len] += 1;
            }
        }

        Self { codes }
    }

    fn decode_symbol(&self, bitreader: &mut BitReader) -> u32 {
        let mut code = 0;

        for len in 1..16 {
            let bit = bitreader.read_bits(1).unwrap();
            code = (code << 1) | bit;

            for symbol in 0..self.codes.len() {
                let huffman_code = &self.codes[symbol];
                if huffman_code.len == len && huffman_code.code as u32 == code {
                    return symbol as u32;
                }
            }
        }

        panic!("invalid huffman code");
    }
}

impl LengthCode {
    #[rustfmt::skip]
    fn new(code: u32) -> Self {
        match code {
            257 => Self { base: 3, extra: 0 },
            258 => Self { base: 4, extra: 0 },
            259 => Self { base: 5, extra: 0 },
            260 => Self { base: 6, extra: 0 },
            261 => Self { base: 7, extra: 0 },
            262 => Self { base: 8, extra: 0 },
            263 => Self { base: 9, extra: 0 },
            264 => Self { base: 10, extra: 0 },
            265 => Self { base: 11, extra: 1 },
            266 => Self { base: 13, extra: 1 },
            267 => Self { base: 15, extra: 1 },
            268 => Self { base: 17, extra: 1 },
            269 => Self { base: 19, extra: 2 },
            270 => Self { base: 23, extra: 2 },
            271 => Self { base: 27, extra: 2 },
            272 => Self { base: 31, extra: 2 },
            273 => Self { base: 35, extra: 3 },
            274 => Self { base: 43, extra: 3 },
            275 => Self { base: 51, extra: 3 },
            276 => Self { base: 59, extra: 3 },
            277 => Self { base: 67, extra: 4 },
            278 => Self { base: 83, extra: 4 },
            279 => Self { base: 99, extra: 4 },
            280 => Self { base: 115, extra: 4 },
            281 => Self { base: 131, extra: 5 },
            282 => Self { base: 163, extra: 5 },
            283 => Self { base: 195, extra: 5 },
            284 => Self { base: 227, extra: 5 },
            285 => Self { base: 258, extra: 0 },
            _ => panic!("Invalid length code: {}", code),
        }
    }
}

impl DistanceCode {
    #[rustfmt::skip]
    fn new(code: u32) -> Self {
        match code {
            0 => Self { base: 1, extra: 0},
            1 => Self { base: 2, extra: 0},
            2 => Self { base: 3, extra: 0},
            3 => Self { base: 4, extra: 0},
            4 => Self { base: 5, extra: 1},
            5 => Self { base: 7, extra: 1},
            6 => Self { base: 9, extra: 2},
            7 => Self { base: 13, extra: 2},
            8 => Self { base: 17, extra: 3},
            9 => Self { base: 25, extra: 3},
            10 => Self { base: 33, extra: 4},
            11 => Self { base: 49, extra: 4},
            12 => Self { base: 65, extra: 5},
            13 => Self { base: 97, extra: 5},
            14 => Self { base: 129, extra: 6},
            15 => Self { base: 193, extra: 6},
            16 => Self { base: 257, extra: 7},
            17 => Self { base: 385, extra: 7},
            18 => Self { base: 513, extra: 8},
            19 => Self { base: 769, extra: 8},
            20 => Self { base: 1025, extra: 9},
            21 => Self { base: 1537, extra: 9},
            22 => Self { base: 2049, extra: 10},
            23 => Self { base: 3073, extra: 10},
            24 => Self { base: 4097, extra: 11},
            25 => Self { base: 6145, extra: 11},
            26 => Self { base: 8193, extra: 12},
            27 => Self { base: 12289, extra: 12},
            28 => Self { base: 16385, extra: 13},
            29 => Self { base: 24577, extra: 13},
            _ => panic!("Invalid distance code: {}", code),
        }
    }
}

impl Debug for Inflate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeflateStream").finish()
    }
}

impl Debug for HuffmanCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b{:0width$b}", self.code, width = self.len)
    }
}
