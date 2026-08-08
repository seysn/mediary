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
        if self.read_signature()? != SIGNATURE {
            return Err(PngError::InvalidSignature);
        }

        let mut found_idat = false;
        loop {
            let chunk = self.read_chunk()?;
            println!("{chunk:?}");

            if let PngChunk::ImageData(idat) = &chunk && !found_idat {
                println!("Compression Method: {:?}", idat.compression_method());
                println!("Maximum Allowed Value: {} bytes", idat.maximum_allowed_value());
                println!("FCHECK: {}", idat.fcheck());
                println!("FDICT: {}", idat.fdict());
                println!("Compression Level: {:?}", idat.compression_level());

                found_idat = true;
            }

            if let PngChunk::ImageTrailer = chunk {
                break;
            }
        }

        Ok(())
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
