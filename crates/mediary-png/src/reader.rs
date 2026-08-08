use std::io::{BufRead, Seek};

use crate::{chunk::PngChunk, error::PngResult};

pub struct PngReader<R: BufRead + Seek> {
    reader: R,
}

impl<R: BufRead + Seek> PngReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_signature(&mut self) -> PngResult<[u8; 8]> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_chunk(&mut self) -> PngResult<PngChunk> {
        PngChunk::read(&mut self.reader)
    }
}
