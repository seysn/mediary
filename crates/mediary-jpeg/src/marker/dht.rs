use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, Seek, Write},
};

use byteorder::{BigEndian, ByteOrder};
use mediary_common::huffman::{HuffmanCode, HuffmanTable};

use crate::{
    error::{JpegError, JpegResult},
    reader::read_u16,
};

#[derive(Clone)]
pub struct DefineHuffmanTable {
    pub class: TableClass,
    pub index: u8,
    pub counts: [u8; 16],
    pub values: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum TableClass {
    DC,
    AC,
}

impl DefineHuffmanTable {
    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> JpegResult<Self> {
        let length = read_u16(reader)?;

        let mut data = vec![0; length as usize - 2];
        reader.read_exact(&mut data)?;

        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> JpegResult<Self> {
        let class = TableClass::try_from((data[0] >> 4) & 0x0f)?;
        let index = data[0] & 0x0f;
        let counts: [u8; 16] = data[1..17]
            .try_into()
            .expect("slice should have a slice of 16 elements");
        let values = data[17..].to_vec();

        Ok(Self {
            class,
            index,
            counts,
            values,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> JpegResult<()> {
        let length = 19 + self.values.len();
        let mut buf = vec![0; length];

        BigEndian::write_u16(&mut buf[0..2], length as u16);
        buf[2] = ((u8::from(self.class) & 0xf) << 4) + (self.index & 0xf);
        buf[3..19].copy_from_slice(&self.counts);
        buf[19..].copy_from_slice(&self.values);

        writer.write_all(&buf)?;

        Ok(())
    }

    pub fn from_table(class: TableClass, index: u8, table: &HuffmanTable) -> Self {
        let mut counts = [0; 16];

        // Flatten table in a Vec to be able to sort codes
        let mut entries: Vec<(u8, u16, u8)> = table
            .reverse_table()
            .iter()
            .map(|(&value, hc)| {
                // Compute counts at the same time to avoid one more loop
                counts[(hc.size - 1) as usize] += 1;

                (value, hc.code, hc.size)
            })
            .collect();

        // Ordering entries by size then by code
        entries.sort_by(|a, b| {
            let size_ordering = a.2.cmp(&b.2);
            if size_ordering.is_eq() {
                // Order by code
                a.1.cmp(&b.1)
            } else {
                size_ordering
            }
        });

        let values: Vec<u8> = entries.into_iter().map(|(symbol, _, _)| symbol).collect();

        Self {
            class,
            index,
            counts,
            values,
        }
    }

    pub fn to_table(&self) -> JpegResult<HuffmanTable> {
        let mut codes = HashMap::new();

        let mut idx = 0;
        let mut code = 0;
        for (l, &count) in self.counts.iter().enumerate() {
            let length = l + 1;
            for _ in 0..count {
                codes.insert(HuffmanCode::new(code, length as u8), self.values[idx]);

                code += 1;
                idx += 1;
            }

            code <<= 1;
        }

        Ok(HuffmanTable::new(codes)?)
    }
}

impl TryFrom<u8> for TableClass {
    type Error = JpegError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::DC,
            1 => Self::AC,
            _ => {
                return Err(JpegError::InvalidValue {
                    element: "TableClass",
                    value: Box::new(value),
                });
            }
        })
    }
}

impl From<TableClass> for u8 {
    fn from(value: TableClass) -> Self {
        match value {
            TableClass::DC => 0,
            TableClass::AC => 1,
        }
    }
}

impl Debug for DefineHuffmanTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefineHuffmanTable")
            .field("class", &self.class)
            .field("index", &self.index)
            .field("counts", &format_args!("{:?}", self.counts))
            .field("values", &format_args!("{:?}", self.values))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::marker::{DefineHuffmanTable, TableClass};

    #[test]
    fn conversion() {
        let dht = DefineHuffmanTable {
            class: TableClass::DC,
            index: 0,
            counts: [0, 1, 3, 2, 3, 4, 6, 4, 9, 7, 8, 7, 5, 9, 1, 0],
            values: vec![
                2, 0, 3, 4, 5, 18, 6, 7, 34, 1, 19, 50, 66, 8, 20, 35, 82, 98, 114, 21, 51, 130,
                146, 17, 33, 36, 52, 67, 83, 162, 178, 194, 9, 22, 37, 49, 99, 115, 210, 38, 54,
                68, 84, 97, 113, 131, 226, 23, 53, 65, 100, 116, 129, 240, 24, 69, 81, 147, 242,
                39, 40, 85, 101, 132, 145, 163, 179, 195, 117,
            ],
        };

        let huffman_table = dht.to_table().unwrap();

        let dht2 = DefineHuffmanTable::from_table(TableClass::DC, 0, &huffman_table);

        assert_eq!(dht.counts, dht2.counts);
        assert_eq!(dht.values, dht2.values);
    }
}
