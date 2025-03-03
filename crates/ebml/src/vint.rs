use std::io::Read;

use super::error::{EbmlError, EbmlResult};

/// Variable-Size Integer (VINT)
#[derive(Debug, PartialEq)]
pub struct Vint {
    /// Octet length
    pub(crate) length: usize,
    /// VINT Data
    pub(crate) value: u64,
    /// Full VINT (Width + Marker + Data)
    pub(crate) raw: u64,
}

impl Vint {
    pub fn from_reader<R: Read>(reader: &mut R) -> EbmlResult<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;

        let byte = buf[0];
        if byte == 0 {
            return Err(EbmlError::InvalidVint);
        }

        let mut length = 1;
        for i in 0..8 {
            let mask = 0x80 >> i;
            if byte & mask == mask {
                break;
            }

            length += 1;
        }

        let mut rest = vec![0; length - 1];
        reader.read_exact(&mut rest)?;

        let mask: u8 = if length == 8 { 0 } else { 0xff >> length };
        let mut value = (byte & mask) as u64;
        let mut raw = byte as u64;
        for b in rest {
            value <<= 8;
            value += b as u64;
            raw <<= 8;
            raw += b as u64;
        }

        Ok(Self { length, value, raw })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn reader_vint<T: AsRef<[u8]>>(value: T) -> Option<Vint> {
        let mut reader = Cursor::new(value);
        Vint::from_reader(&mut reader).ok()
    }

    fn new_vint(length: usize, value: u64, raw: u64) -> Vint {
        Vint { length, value, raw }
    }

    #[test]
    fn test_from_reader() {
        assert_eq!(reader_vint([0x82]), Some(new_vint(1, 2, 0x82)));
        assert_eq!(reader_vint([0x40, 0x02]), Some(new_vint(2, 2, 0x4002)));
        assert_eq!(
            reader_vint([0x20, 0x00, 0x02]),
            Some(new_vint(3, 2, 0x200002))
        );
        assert_eq!(
            reader_vint([0x10, 0x00, 0x00, 0x02]),
            Some(new_vint(4, 2, 0x10000002))
        );
        assert_eq!(
            reader_vint([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02]),
            Some(new_vint(8, 2, 0x0100000000000002))
        );

        assert_eq!(reader_vint([0xff]), Some(new_vint(1, 0x7f, 0xff)));
        assert_eq!(reader_vint([0x7f, 0xff]), Some(new_vint(2, 0x3fff, 0x7fff)));
        assert_eq!(
            reader_vint([0x3f, 0xff, 0xff]),
            Some(new_vint(3, 0x1fffff, 0x3fffff))
        );
        assert_eq!(
            reader_vint([0x1f, 0xff, 0xff, 0xff]),
            Some(new_vint(4, 0xfffffff, 0x1fffffff))
        );

        assert_eq!(reader_vint([]), None);
        assert_eq!(reader_vint([0]), None);
        assert_eq!(reader_vint([0, 0, 0, 0]), None);
    }
}
