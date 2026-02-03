use std::{collections::HashMap, sync::LazyLock};

use mediary_common::huffman::{HuffmanCode, HuffmanTable};
use mediary_image::{
    mono::{MonoImageRef, MonoPixel},
    PackedImageRead, Pixel, RgbImage,
};
use mediary_yuv::{YuvChromaSubsampling, YuvPlanarImage};

use crate::{
    dct::forward::dct_naive,
    marker::{
        ComponentId, DefineQuantizationTable, ImageData, QuantizationTable,
        QuantizationTableValues, SofComponent, SosComponent, StartOfFrame, StartOfScan,
    },
    JpegResult, RawJpeg,
};

pub struct JpegEncoder {
    luma_quantization_table: QuantizationTableValues,
    chroma_quantization_table: QuantizationTableValues,
}

#[rustfmt::skip]
const ZIGZAG: [usize; 64] = [
     0,  1,  5,  6, 14, 15, 27, 28,
     2,  4,  7, 13, 16, 26, 29, 42,
     3,  8, 12, 17, 25, 30, 41, 43,
     9, 11, 18, 24, 31, 40, 44, 53,
    10, 19, 23, 32, 39, 45, 52, 54,
    20, 22, 33, 38, 46, 51, 55, 60,
    21, 34, 37, 47, 50, 56, 59, 61,
    35, 36, 48, 49, 57, 58, 62, 63,
];

/// Table K.3 – Table for luminance DC coefficient differences
static DEFAULT_LUMA_DC_TABLE: LazyLock<HuffmanTable> = LazyLock::new(|| {
    let lookup_table: HashMap<HuffmanCode, u8> = HashMap::from([
        (HuffmanCode::new(0b00, 2), 0),
        (HuffmanCode::new(0b010, 3), 1),
        (HuffmanCode::new(0b011, 3), 2),
        (HuffmanCode::new(0b100, 3), 3),
        (HuffmanCode::new(0b101, 3), 4),
        (HuffmanCode::new(0b110, 3), 5),
        (HuffmanCode::new(0b1110, 4), 6),
        (HuffmanCode::new(0b11110, 5), 7),
        (HuffmanCode::new(0b111110, 6), 8),
        (HuffmanCode::new(0b1111110, 7), 9),
        (HuffmanCode::new(0b11111110, 8), 10),
        (HuffmanCode::new(0b11111110, 9), 11),
    ]);

    HuffmanTable::new(lookup_table).expect("lookup table should not be empty")
});

/// Table K.4 – Table for chrominance DC coefficient differences
static DEFAULT_CHROMA_DC_TABLE: LazyLock<HuffmanTable> = LazyLock::new(|| {
    let lookup_table: HashMap<HuffmanCode, u8> = HashMap::from([
        (HuffmanCode::new(0b00, 2), 0),
        (HuffmanCode::new(0b01, 2), 1),
        (HuffmanCode::new(0b10, 2), 2),
        (HuffmanCode::new(0b110, 3), 3),
        (HuffmanCode::new(0b1110, 4), 4),
        (HuffmanCode::new(0b11110, 5), 5),
        (HuffmanCode::new(0b111110, 6), 6),
        (HuffmanCode::new(0b1111110, 7), 7),
        (HuffmanCode::new(0b11111110, 8), 8),
        (HuffmanCode::new(0b111111110, 9), 9),
        (HuffmanCode::new(0b1111111110, 10), 10),
        (HuffmanCode::new(0b11111111110, 11), 11),
    ]);

    HuffmanTable::new(lookup_table).expect("lookup table should not be empty")
});

/// Table K.5 – Table for luminance AC coefficients
static DEFAULT_LUMA_AC_TABLE: LazyLock<HuffmanTable> = LazyLock::new(|| {
    let lookup_table: HashMap<HuffmanCode, u8> = HashMap::from([
        (HuffmanCode::new(0b1010, 4), 0x00),
        (HuffmanCode::new(0b00, 2), 0x01),
        (HuffmanCode::new(0b01, 2), 0x02),
        (HuffmanCode::new(0b100, 3), 0x03),
        (HuffmanCode::new(0b1011, 4), 0x04),
        (HuffmanCode::new(0b11010, 5), 0x05),
        (HuffmanCode::new(0b1111000, 7), 0x06),
        (HuffmanCode::new(0b11111000, 8), 0x07),
        (HuffmanCode::new(0b1111110110, 10), 0x08),
        (HuffmanCode::new(0b1111111110000010, 16), 0x09),
        (HuffmanCode::new(0b1111111110000011, 16), 0x0A),
        (HuffmanCode::new(0b1100, 4), 0x11),
        (HuffmanCode::new(0b11011, 5), 0x12),
        (HuffmanCode::new(0b1111001, 7), 0x13),
        (HuffmanCode::new(0b111110110, 9), 0x14),
        (HuffmanCode::new(0b11111110110, 11), 0x15),
        (HuffmanCode::new(0b1111111110000100, 16), 0x16),
        (HuffmanCode::new(0b1111111110000101, 16), 0x17),
        (HuffmanCode::new(0b1111111110000110, 16), 0x18),
        (HuffmanCode::new(0b1111111110000111, 16), 0x19),
        (HuffmanCode::new(0b1111111110001000, 16), 0x1A),
        (HuffmanCode::new(0b11100, 5), 0x21),
        (HuffmanCode::new(0b11111001, 8), 0x22),
        (HuffmanCode::new(0b1111110111, 10), 0x23),
        (HuffmanCode::new(0b111111110100, 12), 0x24),
        (HuffmanCode::new(0b1111111110001001, 16), 0x25),
        (HuffmanCode::new(0b1111111110001010, 16), 0x26),
        (HuffmanCode::new(0b1111111110001011, 16), 0x27),
        (HuffmanCode::new(0b1111111110001100, 16), 0x28),
        (HuffmanCode::new(0b1111111110001101, 16), 0x29),
        (HuffmanCode::new(0b1111111110001110, 16), 0x2A),
        (HuffmanCode::new(0b111010, 6), 0x31),
        (HuffmanCode::new(0b111110111, 9), 0x32),
        (HuffmanCode::new(0b111111110101, 12), 0x33),
        (HuffmanCode::new(0b1111111110001111, 16), 0x34),
        (HuffmanCode::new(0b1111111110010000, 16), 0x35),
        (HuffmanCode::new(0b1111111110010001, 16), 0x36),
        (HuffmanCode::new(0b1111111110010010, 16), 0x37),
        (HuffmanCode::new(0b1111111110010011, 16), 0x38),
        (HuffmanCode::new(0b1111111110010100, 16), 0x39),
        (HuffmanCode::new(0b1111111110010101, 16), 0x3A),
        (HuffmanCode::new(0b111011, 16), 0x41),
        (HuffmanCode::new(0b1111111000, 10), 0x42),
        (HuffmanCode::new(0b1111111110010110, 16), 0x43),
        (HuffmanCode::new(0b1111111110010111, 16), 0x44),
        (HuffmanCode::new(0b1111111110011000, 16), 0x45),
        (HuffmanCode::new(0b1111111110011001, 16), 0x46),
        (HuffmanCode::new(0b1111111110011010, 16), 0x47),
        (HuffmanCode::new(0b1111111110011011, 16), 0x48),
        (HuffmanCode::new(0b1111111110011100, 16), 0x49),
        (HuffmanCode::new(0b1111111110011101, 16), 0x4A),
        (HuffmanCode::new(0b1111010, 17), 0x51),
        (HuffmanCode::new(0b11111110111, 11), 0x52),
        (HuffmanCode::new(0b1111111110011110, 16), 0x53),
        (HuffmanCode::new(0b1111111110011111, 16), 0x54),
        (HuffmanCode::new(0b1111111110100000, 16), 0x55),
        (HuffmanCode::new(0b1111111110100001, 16), 0x56),
        (HuffmanCode::new(0b1111111110100010, 16), 0x57),
        (HuffmanCode::new(0b1111111110100011, 16), 0x58),
        (HuffmanCode::new(0b1111111110100100, 16), 0x59),
        (HuffmanCode::new(0b1111111110100101, 16), 0x5A),
        (HuffmanCode::new(0b1111011, 17), 0x61),
        (HuffmanCode::new(0b111111110110, 12), 0x62),
        (HuffmanCode::new(0b1111111110100110, 16), 0x63),
        (HuffmanCode::new(0b1111111110100111, 16), 0x64),
        (HuffmanCode::new(0b1111111110101000, 16), 0x65),
        (HuffmanCode::new(0b1111111110101001, 16), 0x66),
        (HuffmanCode::new(0b1111111110101010, 16), 0x67),
        (HuffmanCode::new(0b1111111110101011, 16), 0x68),
        (HuffmanCode::new(0b1111111110101100, 16), 0x69),
        (HuffmanCode::new(0b1111111110101101, 16), 0x6A),
        (HuffmanCode::new(0b11111010, 18), 0x71),
        (HuffmanCode::new(0b111111110111, 12), 0x72),
        (HuffmanCode::new(0b1111111110101110, 16), 0x73),
        (HuffmanCode::new(0b1111111110101111, 16), 0x74),
        (HuffmanCode::new(0b1111111110110000, 16), 0x75),
        (HuffmanCode::new(0b1111111110110001, 16), 0x76),
        (HuffmanCode::new(0b1111111110110010, 16), 0x77),
        (HuffmanCode::new(0b1111111110110011, 16), 0x78),
        (HuffmanCode::new(0b1111111110110100, 16), 0x79),
        (HuffmanCode::new(0b1111111110110101, 16), 0x7A),
        (HuffmanCode::new(0b111111000, 19), 0x81),
        (HuffmanCode::new(0b111111111000000, 15), 0x82),
        (HuffmanCode::new(0b1111111110110110, 16), 0x83),
        (HuffmanCode::new(0b1111111110110111, 16), 0x84),
        (HuffmanCode::new(0b1111111110111000, 16), 0x85),
        (HuffmanCode::new(0b1111111110111001, 16), 0x86),
        (HuffmanCode::new(0b1111111110111010, 16), 0x87),
        (HuffmanCode::new(0b1111111110111011, 16), 0x88),
        (HuffmanCode::new(0b1111111110111100, 16), 0x89),
        (HuffmanCode::new(0b1111111110111101, 16), 0x8A),
        (HuffmanCode::new(0b111111001, 19), 0x91),
        (HuffmanCode::new(0b1111111110111110, 16), 0x92),
        (HuffmanCode::new(0b1111111110111111, 16), 0x93),
        (HuffmanCode::new(0b1111111111000000, 16), 0x94),
        (HuffmanCode::new(0b1111111111000001, 16), 0x95),
        (HuffmanCode::new(0b1111111111000010, 16), 0x96),
        (HuffmanCode::new(0b1111111111000011, 16), 0x97),
        (HuffmanCode::new(0b1111111111000100, 16), 0x98),
        (HuffmanCode::new(0b1111111111000101, 16), 0x99),
        (HuffmanCode::new(0b1111111111000110, 16), 0x9A),
        (HuffmanCode::new(0b111111010, 19), 0xA1),
        (HuffmanCode::new(0b1111111111000111, 16), 0xA2),
        (HuffmanCode::new(0b1111111111001000, 16), 0xA3),
        (HuffmanCode::new(0b1111111111001001, 16), 0xA4),
        (HuffmanCode::new(0b1111111111001010, 16), 0xA5),
        (HuffmanCode::new(0b1111111111001011, 16), 0xA6),
        (HuffmanCode::new(0b1111111111001100, 16), 0xA7),
        (HuffmanCode::new(0b1111111111001101, 16), 0xA8),
        (HuffmanCode::new(0b1111111111001110, 16), 0xA9),
        (HuffmanCode::new(0b1111111111001111, 16), 0xAA),
        (HuffmanCode::new(0b1111111001, 10), 0xB1),
        (HuffmanCode::new(0b1111111111010000, 16), 0xB2),
        (HuffmanCode::new(0b1111111111010001, 16), 0xB3),
        (HuffmanCode::new(0b1111111111010010, 16), 0xB4),
        (HuffmanCode::new(0b1111111111010011, 16), 0xB5),
        (HuffmanCode::new(0b1111111111010100, 16), 0xB6),
        (HuffmanCode::new(0b1111111111010101, 16), 0xB7),
        (HuffmanCode::new(0b1111111111010110, 16), 0xB8),
        (HuffmanCode::new(0b1111111111010111, 16), 0xB9),
        (HuffmanCode::new(0b1111111111011000, 16), 0xBA),
        (HuffmanCode::new(0b1111111010, 10), 0xC1),
        (HuffmanCode::new(0b1111111111011001, 16), 0xC2),
        (HuffmanCode::new(0b1111111111011010, 16), 0xC3),
        (HuffmanCode::new(0b1111111111011011, 16), 0xC4),
        (HuffmanCode::new(0b1111111111011100, 16), 0xC5),
        (HuffmanCode::new(0b1111111111011101, 16), 0xC6),
        (HuffmanCode::new(0b1111111111011110, 16), 0xC7),
        (HuffmanCode::new(0b1111111111011111, 16), 0xC8),
        (HuffmanCode::new(0b1111111111100000, 16), 0xC9),
        (HuffmanCode::new(0b1111111111100001, 16), 0xCA),
        (HuffmanCode::new(0b11111111000, 11), 0xD1),
        (HuffmanCode::new(0b1111111111100010, 16), 0xD2),
        (HuffmanCode::new(0b1111111111100011, 16), 0xD3),
        (HuffmanCode::new(0b1111111111100100, 16), 0xD4),
        (HuffmanCode::new(0b1111111111100101, 16), 0xD5),
        (HuffmanCode::new(0b1111111111100110, 16), 0xD6),
        (HuffmanCode::new(0b1111111111100111, 16), 0xD7),
        (HuffmanCode::new(0b1111111111101000, 16), 0xD8),
        (HuffmanCode::new(0b1111111111101001, 16), 0xD9),
        (HuffmanCode::new(0b1111111111101010, 16), 0xDA),
        (HuffmanCode::new(0b1111111111101011, 16), 0xE1),
        (HuffmanCode::new(0b1111111111101100, 16), 0xE2),
        (HuffmanCode::new(0b1111111111101101, 16), 0xE3),
        (HuffmanCode::new(0b1111111111101110, 16), 0xE4),
        (HuffmanCode::new(0b1111111111101111, 16), 0xE5),
        (HuffmanCode::new(0b1111111111110000, 16), 0xE6),
        (HuffmanCode::new(0b1111111111110001, 16), 0xE7),
        (HuffmanCode::new(0b1111111111110010, 16), 0xE8),
        (HuffmanCode::new(0b1111111111110011, 16), 0xE9),
        (HuffmanCode::new(0b1111111111110100, 16), 0xEA),
        (HuffmanCode::new(0b11111111001, 11), 0xF0),
        (HuffmanCode::new(0b1111111111110101, 16), 0xF1),
        (HuffmanCode::new(0b1111111111110110, 16), 0xF2),
        (HuffmanCode::new(0b1111111111110111, 16), 0xF3),
        (HuffmanCode::new(0b1111111111111000, 16), 0xF4),
        (HuffmanCode::new(0b1111111111111001, 16), 0xF5),
        (HuffmanCode::new(0b1111111111111010, 16), 0xF6),
        (HuffmanCode::new(0b1111111111111011, 16), 0xF7),
        (HuffmanCode::new(0b1111111111111100, 16), 0xF8),
        (HuffmanCode::new(0b1111111111111101, 16), 0xF9),
        (HuffmanCode::new(0b1111111111111110, 16), 0xFA),
    ]);

    HuffmanTable::new(lookup_table).expect("lookup table should not be empty")
});

/// Table K.6 – Table for chrominance AC coefficients
static DEFAULT_CHROMA_AC_TABLE: LazyLock<HuffmanTable> = LazyLock::new(|| {
    let lookup_table: HashMap<HuffmanCode, u8> = HashMap::from([
        (HuffmanCode::new(0b00, 2), 0x00),
        (HuffmanCode::new(0b01, 2), 0x01),
        (HuffmanCode::new(0b100, 3), 0x02),
        (HuffmanCode::new(0b1010, 4), 0x03),
        (HuffmanCode::new(0b11000, 5), 0x04),
        (HuffmanCode::new(0b11001, 5), 0x05),
        (HuffmanCode::new(0b111000, 6), 0x06),
        (HuffmanCode::new(0b1111000, 7), 0x07),
        (HuffmanCode::new(0b111110100, 9), 0x08),
        (HuffmanCode::new(0b1111110110, 10), 0x09),
        (HuffmanCode::new(0b111111110100, 12), 0x0A),
        (HuffmanCode::new(0b1011, 4), 0x11),
        (HuffmanCode::new(0b111001, 6), 0x12),
        (HuffmanCode::new(0b11110110, 8), 0x13),
        (HuffmanCode::new(0b111110101, 9), 0x14),
        (HuffmanCode::new(0b11111110110, 11), 0x15),
        (HuffmanCode::new(0b111111110101, 12), 0x16),
        (HuffmanCode::new(0b1111111110001000, 16), 0x17),
        (HuffmanCode::new(0b1111111110001001, 16), 0x18),
        (HuffmanCode::new(0b1111111110001010, 16), 0x19),
        (HuffmanCode::new(0b1111111110001011, 16), 0x1A),
        (HuffmanCode::new(0b11010, 5), 0x21),
        (HuffmanCode::new(0b11110111, 8), 0x22),
        (HuffmanCode::new(0b1111110111, 10), 0x23),
        (HuffmanCode::new(0b111111110110, 12), 0x24),
        (HuffmanCode::new(0b111111111000010, 15), 0x25),
        (HuffmanCode::new(0b1111111110001100, 16), 0x26),
        (HuffmanCode::new(0b1111111110001101, 16), 0x27),
        (HuffmanCode::new(0b1111111110001110, 16), 0x28),
        (HuffmanCode::new(0b1111111110001111, 16), 0x29),
        (HuffmanCode::new(0b1111111110010000, 16), 0x2A),
        (HuffmanCode::new(0b11011, 5), 0x31),
        (HuffmanCode::new(0b11111000, 8), 0x32),
        (HuffmanCode::new(0b1111111000, 10), 0x33),
        (HuffmanCode::new(0b111111110111, 12), 0x34),
        (HuffmanCode::new(0b1111111110010001, 16), 0x35),
        (HuffmanCode::new(0b1111111110010010, 16), 0x36),
        (HuffmanCode::new(0b1111111110010011, 16), 0x37),
        (HuffmanCode::new(0b1111111110010100, 16), 0x38),
        (HuffmanCode::new(0b1111111110010101, 16), 0x39),
        (HuffmanCode::new(0b1111111110010110, 16), 0x3A),
        (HuffmanCode::new(0b111010, 6), 0x41),
        (HuffmanCode::new(0b111110110, 19), 0x42),
        (HuffmanCode::new(0b1111111110010111, 16), 0x43),
        (HuffmanCode::new(0b1111111110011000, 16), 0x44),
        (HuffmanCode::new(0b1111111110011001, 16), 0x45),
        (HuffmanCode::new(0b1111111110011010, 16), 0x46),
        (HuffmanCode::new(0b1111111110011011, 16), 0x47),
        (HuffmanCode::new(0b1111111110011100, 16), 0x48),
        (HuffmanCode::new(0b1111111110011101, 16), 0x49),
        (HuffmanCode::new(0b1111111110011110, 16), 0x4A),
        (HuffmanCode::new(0b111011, 16), 0x51),
        (HuffmanCode::new(0b1111111001, 10), 0x52),
        (HuffmanCode::new(0b1111111110011111, 16), 0x53),
        (HuffmanCode::new(0b1111111110100000, 16), 0x54),
        (HuffmanCode::new(0b1111111110100001, 16), 0x55),
        (HuffmanCode::new(0b1111111110100010, 16), 0x56),
        (HuffmanCode::new(0b1111111110100011, 16), 0x57),
        (HuffmanCode::new(0b1111111110100100, 16), 0x58),
        (HuffmanCode::new(0b1111111110100101, 16), 0x59),
        (HuffmanCode::new(0b1111111110100110, 16), 0x5A),
        (HuffmanCode::new(0b1111001, 17), 0x61),
        (HuffmanCode::new(0b11111110111, 11), 0x62),
        (HuffmanCode::new(0b1111111110100111, 16), 0x63),
        (HuffmanCode::new(0b1111111110101000, 16), 0x64),
        (HuffmanCode::new(0b1111111110101001, 16), 0x65),
        (HuffmanCode::new(0b1111111110101010, 16), 0x66),
        (HuffmanCode::new(0b1111111110101011, 16), 0x67),
        (HuffmanCode::new(0b1111111110101100, 16), 0x68),
        (HuffmanCode::new(0b1111111110101101, 16), 0x69),
        (HuffmanCode::new(0b1111111110101110, 16), 0x6A),
        (HuffmanCode::new(0b1111010, 17), 0x71),
        (HuffmanCode::new(0b11111111000, 11), 0x72),
        (HuffmanCode::new(0b1111111110101111, 16), 0x73),
        (HuffmanCode::new(0b1111111110110000, 16), 0x74),
        (HuffmanCode::new(0b1111111110110001, 16), 0x75),
        (HuffmanCode::new(0b1111111110110010, 16), 0x76),
        (HuffmanCode::new(0b1111111110110011, 16), 0x77),
        (HuffmanCode::new(0b1111111110110100, 16), 0x78),
        (HuffmanCode::new(0b1111111110110101, 16), 0x79),
        (HuffmanCode::new(0b1111111110110110, 16), 0x7A),
        (HuffmanCode::new(0b11111001, 18), 0x81),
        (HuffmanCode::new(0b1111111110110111, 16), 0x82),
        (HuffmanCode::new(0b1111111110111000, 16), 0x83),
        (HuffmanCode::new(0b1111111110111001, 16), 0x84),
        (HuffmanCode::new(0b1111111110111010, 16), 0x85),
        (HuffmanCode::new(0b1111111110111011, 16), 0x86),
        (HuffmanCode::new(0b1111111110111100, 16), 0x87),
        (HuffmanCode::new(0b1111111110111101, 16), 0x88),
        (HuffmanCode::new(0b1111111110111110, 16), 0x89),
        (HuffmanCode::new(0b1111111110111111, 16), 0x8A),
        (HuffmanCode::new(0b111110111, 19), 0x91),
        (HuffmanCode::new(0b1111111111000000, 16), 0x92),
        (HuffmanCode::new(0b1111111111000001, 16), 0x93),
        (HuffmanCode::new(0b1111111111000010, 16), 0x94),
        (HuffmanCode::new(0b1111111111000011, 16), 0x95),
        (HuffmanCode::new(0b1111111111000100, 16), 0x96),
        (HuffmanCode::new(0b1111111111000101, 16), 0x97),
        (HuffmanCode::new(0b1111111111000110, 16), 0x98),
        (HuffmanCode::new(0b1111111111000111, 16), 0x99),
        (HuffmanCode::new(0b1111111111001000, 16), 0x9A),
        (HuffmanCode::new(0b111111000, 19), 0xA1),
        (HuffmanCode::new(0b1111111111001001, 16), 0xA2),
        (HuffmanCode::new(0b1111111111001010, 16), 0xA3),
        (HuffmanCode::new(0b1111111111001011, 16), 0xA4),
        (HuffmanCode::new(0b1111111111001100, 16), 0xA5),
        (HuffmanCode::new(0b1111111111001101, 16), 0xA6),
        (HuffmanCode::new(0b1111111111001110, 16), 0xA7),
        (HuffmanCode::new(0b1111111111001111, 16), 0xA8),
        (HuffmanCode::new(0b1111111111010000, 16), 0xA9),
        (HuffmanCode::new(0b1111111111010001, 16), 0xAA),
        (HuffmanCode::new(0b111111001, 19), 0xB1),
        (HuffmanCode::new(0b1111111111010010, 16), 0xB2),
        (HuffmanCode::new(0b1111111111010011, 16), 0xB3),
        (HuffmanCode::new(0b1111111111010100, 16), 0xB4),
        (HuffmanCode::new(0b1111111111010101, 16), 0xB5),
        (HuffmanCode::new(0b1111111111010110, 16), 0xB6),
        (HuffmanCode::new(0b1111111111010111, 16), 0xB7),
        (HuffmanCode::new(0b1111111111011000, 16), 0xB8),
        (HuffmanCode::new(0b1111111111011001, 16), 0xB9),
        (HuffmanCode::new(0b1111111111011010, 16), 0xBA),
        (HuffmanCode::new(0b111111010, 19), 0xC1),
        (HuffmanCode::new(0b1111111111011011, 16), 0xC2),
        (HuffmanCode::new(0b1111111111011100, 16), 0xC3),
        (HuffmanCode::new(0b1111111111011101, 16), 0xC4),
        (HuffmanCode::new(0b1111111111011110, 16), 0xC5),
        (HuffmanCode::new(0b1111111111011111, 16), 0xC6),
        (HuffmanCode::new(0b1111111111100000, 16), 0xC7),
        (HuffmanCode::new(0b1111111111100001, 16), 0xC8),
        (HuffmanCode::new(0b1111111111100010, 16), 0xC9),
        (HuffmanCode::new(0b1111111111100011, 16), 0xCA),
        (HuffmanCode::new(0b11111111001, 11), 0xD1),
        (HuffmanCode::new(0b1111111111100100, 16), 0xD2),
        (HuffmanCode::new(0b1111111111100101, 16), 0xD3),
        (HuffmanCode::new(0b1111111111100110, 16), 0xD4),
        (HuffmanCode::new(0b1111111111100111, 16), 0xD5),
        (HuffmanCode::new(0b1111111111101000, 16), 0xD6),
        (HuffmanCode::new(0b1111111111101001, 16), 0xD7),
        (HuffmanCode::new(0b1111111111101010, 16), 0xD8),
        (HuffmanCode::new(0b1111111111101011, 16), 0xD9),
        (HuffmanCode::new(0b1111111111101100, 16), 0xDA),
        (HuffmanCode::new(0b11111111100000, 14), 0xE1),
        (HuffmanCode::new(0b1111111111101101, 16), 0xE2),
        (HuffmanCode::new(0b1111111111101110, 16), 0xE3),
        (HuffmanCode::new(0b1111111111101111, 16), 0xE4),
        (HuffmanCode::new(0b1111111111110000, 16), 0xE5),
        (HuffmanCode::new(0b1111111111110001, 16), 0xE6),
        (HuffmanCode::new(0b1111111111110010, 16), 0xE7),
        (HuffmanCode::new(0b1111111111110011, 16), 0xE8),
        (HuffmanCode::new(0b1111111111110100, 16), 0xE9),
        (HuffmanCode::new(0b1111111111110101, 16), 0xEA),
        (HuffmanCode::new(0b1111111010, 10), 0xF0),
        (HuffmanCode::new(0b111111111000011, 15), 0xF1),
        (HuffmanCode::new(0b1111111111110110, 16), 0xF2),
        (HuffmanCode::new(0b1111111111110111, 16), 0xF3),
        (HuffmanCode::new(0b1111111111111000, 16), 0xF4),
        (HuffmanCode::new(0b1111111111111001, 16), 0xF5),
        (HuffmanCode::new(0b1111111111111010, 16), 0xF6),
        (HuffmanCode::new(0b1111111111111011, 16), 0xF7),
        (HuffmanCode::new(0b1111111111111100, 16), 0xF8),
        (HuffmanCode::new(0b1111111111111101, 16), 0xF9),
        (HuffmanCode::new(0b1111111111111110, 16), 0xFA),
    ]);

    HuffmanTable::new(lookup_table).expect("lookup table should not be empty")
});

const LUMA_TABLE_INDEX: u8 = 0;
const CHROMA_TABLE_INDEX: u8 = 1;

impl JpegEncoder {
    pub fn new(quality: u8) -> Self {
        Self {
            luma_quantization_table: QuantizationTableValues::new_luma(quality),
            chroma_quantization_table: QuantizationTableValues::new_chroma(quality),
        }
    }

    pub fn encode_rgb(&self, input: &RgbImage) -> JpegResult<RawJpeg> {
        let yuv = YuvPlanarImage::new_yuv420_from_rgb(input);
        self.encode_yuv(&yuv)
    }

    pub fn encode_yuv(&self, input: &YuvPlanarImage) -> JpegResult<RawJpeg> {
        let mut coeff_pool = vec![0; 64];
        let mut dct_output = vec![0; 64];
        let mut data = Vec::new();

        let (width, height) = (input.width(), input.height());
        let (luma_width, luma_height, chroma_width, chroma_height) = match input.subsampling() {
            YuvChromaSubsampling::Yuv444 => (width, height, width, height),
            YuvChromaSubsampling::Yuv420 => (width, height, width / 2, height / 2),
        };

        let y_plane = MonoImageRef::new(input.y(), luma_width, luma_height).unwrap();
        let u_plane = MonoImageRef::new(input.u(), chroma_width, chroma_height).unwrap();
        let v_plane = MonoImageRef::new(input.v(), chroma_width, chroma_height).unwrap();

        let (
            luma_horizontal_sampling,
            luma_vertical_sampling,
            chroma_horizontal_sampling,
            chroma_vertical_sampling,
        ) = match input.as_data().subsampling {
            YuvChromaSubsampling::Yuv444 => (1, 1, 1, 1),
            YuvChromaSubsampling::Yuv420 => (2, 2, 1, 1),
        };

        let luma_dc_table = &*DEFAULT_LUMA_DC_TABLE;
        let luma_ac_table = &*DEFAULT_LUMA_AC_TABLE;
        let chroma_dc_table = &*DEFAULT_CHROMA_DC_TABLE;
        let chroma_ac_table = &*DEFAULT_CHROMA_AC_TABLE;

        let mcu_width = width / (8 * luma_horizontal_sampling);
        let mcu_height = height / (8 * luma_vertical_sampling);
        for block_y in 0..mcu_height {
            for block_x in 0..mcu_width {
                println!("# block {block_x} {block_y}");
                let block_width = 8 * luma_horizontal_sampling;
                let block_height = 8 * luma_vertical_sampling;

                for luma_y in 0..luma_vertical_sampling {
                    for luma_x in 0..luma_horizontal_sampling {
                        println!("## luma {luma_x} {luma_y}");
                        let sub_block_x = block_x * 8 * luma_x;
                        let sub_block_y = block_y * 8 * luma_y;

                        let y_view = y_plane.view(sub_block_x, sub_block_y, 8, 8).unwrap();
                        let y_block = y_view.to_vec();

                        dct_naive(MonoPixel::as_row_slice(&y_block), &mut dct_output);

                        // for (coeff, quantization) in
                        //     coeff_pool.iter_mut().zip(&self.luma_quantization_table.0)
                        // {
                        //     *coeff /= i16::from(*quantization);
                        // }

                        println!("= {:?}\n", self.luma_quantization_table);
                        println!("> {dct_output:?}\n");
                        for (i, coeff) in dct_output.iter().enumerate() {
                            // println!("\t{coeff} {}", i16::from(self.luma_quantization_table[i]));
                            coeff_pool[ZIGZAG[i]] =
                                *coeff / i16::from(self.luma_quantization_table[i]);
                        }
                        println!("< {coeff_pool:?}\n");

                        // for coeff in &

                        // println!("> {coeff_pool:?}\n");
                        // self.encode_block(&coeff_pool);
                    }
                }
            }
        }

        Ok(RawJpeg {
            start_of_frame: Some(StartOfFrame {
                precision: 8,
                width: width as u16,
                height: height as u16,
                components: vec![
                    SofComponent {
                        id: ComponentId::Y,
                        horizontal_sampling: luma_horizontal_sampling as u8,
                        vertical_sampling: luma_vertical_sampling as u8,
                        quantization_table: LUMA_TABLE_INDEX,
                    },
                    SofComponent {
                        id: ComponentId::Cb,
                        horizontal_sampling: chroma_horizontal_sampling,
                        vertical_sampling: chroma_vertical_sampling,
                        quantization_table: CHROMA_TABLE_INDEX,
                    },
                    SofComponent {
                        id: ComponentId::Cr,
                        horizontal_sampling: chroma_horizontal_sampling,
                        vertical_sampling: chroma_vertical_sampling,
                        quantization_table: CHROMA_TABLE_INDEX,
                    },
                ],
            }),
            quantization_tables: vec![
                DefineQuantizationTable(vec![QuantizationTable {
                    precision: 0,
                    index: LUMA_TABLE_INDEX,
                    values: self.luma_quantization_table.clone(),
                }]),
                DefineQuantizationTable(vec![QuantizationTable {
                    precision: 0,
                    index: CHROMA_TABLE_INDEX,
                    values: self.chroma_quantization_table.clone(),
                }]),
            ],
            huffman_tables: vec![],
            start_of_scan: Some(StartOfScan {
                components: vec![
                    SosComponent {
                        id: ComponentId::Y,
                        dc_table: LUMA_TABLE_INDEX,
                        ac_table: LUMA_TABLE_INDEX,
                    },
                    SosComponent {
                        id: ComponentId::Cb,
                        dc_table: CHROMA_TABLE_INDEX,
                        ac_table: CHROMA_TABLE_INDEX,
                    },
                    SosComponent {
                        id: ComponentId::Cr,
                        dc_table: CHROMA_TABLE_INDEX,
                        ac_table: CHROMA_TABLE_INDEX,
                    },
                ],
                start_spectral: 0,
                end_spectral: 63,
                approximation_bit: 0,
                data: ImageData(data),
            }),
            ..RawJpeg::default()
        })
    }
}
