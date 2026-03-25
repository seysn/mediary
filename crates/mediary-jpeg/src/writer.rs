use std::io::{Seek, Write};

use crate::{JpegResult, RawJpeg, marker::MarkerId};

pub struct JpegWriter<W: Write + Seek> {
    writer: W,
}

impl<W: Write + Seek> JpegWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write(&mut self, jpeg: &RawJpeg) -> JpegResult<()> {
        MarkerId::SOI.write(&mut self.writer)?;

        if let Some(jfif) = &jpeg.jfif {
            MarkerId::APP(0).write(&mut self.writer)?;
            jfif.write(&mut self.writer)?;
        }

        for dqt in &jpeg.quantization_tables {
            MarkerId::DQT.write(&mut self.writer)?;
            dqt.write(&mut self.writer)?;
        }

        if let Some(sof) = &jpeg.start_of_frame {
            MarkerId::SOF(0).write(&mut self.writer)?;
            sof.write(&mut self.writer)?;
        }

        for dht in &jpeg.huffman_tables {
            MarkerId::DHT.write(&mut self.writer)?;
            dht.write(&mut self.writer)?;
        }

        if let Some(sos) = &jpeg.start_of_scan {
            MarkerId::SOS.write(&mut self.writer)?;
            sos.write(&mut self.writer)?;
        }

        MarkerId::EOI.write(&mut self.writer)?;

        Ok(())
    }
}
