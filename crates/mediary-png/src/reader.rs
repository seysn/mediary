use std::io::{BufRead, Seek};

use crate::{
    chunk::PngChunk,
    error::{PngError, PngResult},
};

const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub struct PngReader<R: BufRead + Seek> {
    reader: R,
}

impl<R: BufRead + Seek> PngReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read(mut self) -> PngResult<()> {
        self.read_signature()?;

        loop {
            let chunk = self.read_chunk()?;
            println!("{chunk:?}");

            if let PngChunk::ImageTrailer = chunk {
                break;
            }
        }

        Ok(())
    }

    pub fn read_signature(&mut self) -> PngResult<()> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;

        if buf != SIGNATURE {
            return Err(PngError::InvalidSignature);
        }

        Ok(())
    }

    pub fn read_chunk(&mut self) -> PngResult<PngChunk> {
        PngChunk::read(&mut self.reader)
    }
}
