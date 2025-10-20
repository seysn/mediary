use std::io::{BufRead, Read};

use mediary_common::{bitreader::BitReader, huffman::HuffmanTable};
use mediary_image::{mono::MonoImageRef, ImageSource, ImageSourceMut, RgbImage};

use crate::{
    idct,
    marker::{ComponentId, QuantizationTableValues},
    JpegError, JpegResult,
};

pub const MAX_COMPONENTS: usize = 4;

pub struct JpegDecoder<'a> {
    pub data: &'a [u8],

    pub mcu_width: u16,
    pub mcu_height: u16,
    pub components: Vec<Component>,
    pub dc_huffman_tables: [Option<HuffmanTable>; MAX_COMPONENTS],
    pub ac_huffman_tables: [Option<HuffmanTable>; MAX_COMPONENTS],
    pub quantization_tables: [Option<QuantizationTableValues>; MAX_COMPONENTS],
}

#[derive(Debug)]
pub struct Component {
    pub id: ComponentId,
    pub horizontal_sampling: u8,
    pub vertical_sampling: u8,
    pub quantization_table: usize,
    pub dc_table: usize,
    pub ac_table: usize,
}

/// Reader that filter markers
pub struct DataReader<'a> {
    data: &'a [u8],
    idx: usize,
}

#[derive(Debug)]
struct ComponentPlane {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

struct Mcu<'a> {
    planes: &'a mut [ComponentPlane],
    x: usize,
    y: usize,
}

#[rustfmt::skip]
const _ZIGZAG: [usize; 64] = [
     0,  1,  5,  6, 14, 15, 27, 28,
     2,  4,  7, 13, 16, 26, 29, 42,
     3,  8, 12, 17, 25, 30, 41, 43,
     9, 11, 18, 24, 31, 40, 44, 53,
    10, 19, 23, 32, 39, 45, 52, 54,
    20, 22, 33, 38, 46, 51, 55, 60,
    21, 34, 37, 47, 50, 56, 59, 61,
    35, 36, 48, 49, 57, 58, 62, 63,
];

#[rustfmt::skip]
pub const UN_ZIGZAG: [usize; 64] = [
     0,  1,  8, 16,  9,  2,  3, 10,
    17, 24, 32, 25, 18, 11,  4,  5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13,  6,  7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

impl JpegDecoder<'_> {
    pub fn decode(&self, output: &mut RgbImage) -> JpegResult<()> {
        let mut bitreader = BitReader::new(DataReader::new(self.data));
        let mut previous_dcs: Vec<i16> = vec![0; self.components.len()];
        let mut coeff_pool: Vec<i16> = vec![0; 64];
        let mut planes: Vec<ComponentPlane> =
            self.components.iter().map(ComponentPlane::new).collect();

        for mcu_y in 0..self.mcu_height {
            for mcu_x in 0..self.mcu_width {
                tracing::debug!("Decoding MCU ({mcu_x}, {mcu_y})");
                self.decode_mcu(
                    Mcu {
                        planes: &mut planes,
                        x: usize::from(mcu_x),
                        y: usize::from(mcu_y),
                    },
                    output,
                    &mut previous_dcs,
                    &mut coeff_pool,
                    &mut bitreader,
                )?;
            }
        }

        Ok(())
    }

    fn decode_mcu(
        &self,
        mcu: Mcu<'_>,
        output: &mut RgbImage,
        previous_dcs: &mut [i16],
        coeff_pool: &mut [i16],
        bitreader: &mut BitReader<DataReader>,
    ) -> JpegResult<()> {
        for component in &self.components {
            tracing::debug!(
                "Decoding component {:?} ({}px x {}px)",
                component.id,
                component.horizontal_sampling * 8,
                component.vertical_sampling * 8
            );
            let index = component.index();
            let previous_dc = &mut previous_dcs[index];
            let plane = &mut mcu.planes[index];

            self.decode_component(component, previous_dc, coeff_pool, plane, bitreader)?;
            tracing::trace!("Decoded component {:?}: {:?}", component.id, plane.data);
        }

        mcu.ycbcr_to_rgb(output);
        Ok(())
    }

    fn decode_component(
        &self,
        component: &Component,
        previous_dc: &mut i16,
        coeff_pool: &mut [i16],
        plane: &mut ComponentPlane,
        bitreader: &mut BitReader<DataReader>,
    ) -> JpegResult<()> {
        for block_y in 0..component.horizontal_sampling {
            for block_x in 0..component.vertical_sampling {
                tracing::debug!("Decoding block ({block_x}, {block_y})");
                self.decode_block(component, previous_dc, coeff_pool, bitreader)?;

                idct::idct_two_pass(
                    coeff_pool,
                    &mut plane.data,
                    usize::from(component.horizontal_sampling) * 8,
                    usize::from(block_x),
                    usize::from(block_y),
                );
            }
        }

        Ok(())
    }

    fn decode_block(
        &self,
        component: &Component,
        previous_dc: &mut i16,
        coeff_pool: &mut [i16],
        bitreader: &mut BitReader<DataReader>,
    ) -> JpegResult<()> {
        // Reset buffer
        coeff_pool.fill(0);

        let dc_table = self.dc_huffman_tables[component.dc_table]
            .as_ref()
            .ok_or(JpegError::MissingMarker(crate::marker::MarkerId::DHT))?;
        let ac_table = self.ac_huffman_tables[component.ac_table]
            .as_ref()
            .ok_or(JpegError::MissingMarker(crate::marker::MarkerId::DHT))?;

        let quantization_table = self.quantization_tables[component.quantization_table]
            .as_ref()
            .ok_or(JpegError::MissingMarker(crate::marker::MarkerId::DQT))?;

        let size = dc_table.decode_one(bitreader)?;

        // Value is the difference with the previous dc value because these values are close to each
        // other and it make difference a value that can be written with a low number of bits
        let difference = huffman_receive_extend(bitreader, size)?;
        let dc = *previous_dc + difference;
        *previous_dc = dc;
        coeff_pool[0] = dc * i16::from(quantization_table.0[0]);

        let mut k = 1;
        while k < 64 {
            // Value is composed of 4 bits of RUN and 4 bits of SIZE
            let value = ac_table.decode_one(bitreader)?;

            if value == 0 {
                // A pure zero value is an End of Block
                break;
            }

            if value == 0xf0 {
                // Skip 16 zeros in a row
                k += 16;
                continue;
            }

            // How many zero to skip before non-zero value
            let run = value >> 4;
            k += usize::from(run);

            // Size of non-zero value
            let size = value & 0x0f;

            if k < 64 {
                let ac = huffman_receive_extend(bitreader, size)?;
                coeff_pool[UN_ZIGZAG[k]] = ac * i16::from(quantization_table.0[k]);
                k += 1;
            }
        }

        Ok(())
    }
}

impl Mcu<'_> {
    #[allow(clippy::unwrap_used)]
    fn ycbcr_to_rgb(&self, output: &mut RgbImage) {
        let y_width = self.planes[0].width;
        let y_height = self.planes[0].height;
        let y_plane = MonoImageRef::new(&self.planes[0].data, y_width, y_height).unwrap();

        let cb_width = self.planes[1].width;
        let cb_height = self.planes[1].height;
        let cb_plane = MonoImageRef::new(&self.planes[1].data, cb_width, cb_height).unwrap();
        let cb_plane_up = cb_plane.upscale(y_width / cb_width, y_height / cb_height);

        let cr_width = self.planes[2].width;
        let cr_height = self.planes[2].height;
        let cr_plane = MonoImageRef::new(&self.planes[2].data, cr_width, cr_height).unwrap();
        let cr_plane_up = cr_plane.upscale(y_width / cr_width, y_height / cr_height);

        let mut output = output
            .view_mut(self.x * y_width, self.y * y_height, y_width, y_height)
            .unwrap();

        // We loop over Y component because we know that it is always going to
        // be the one that has the larger dimension everytime. We can then pick
        // other components values based on difference of size to simulate an upscaling.
        for row in 0..y_height {
            for col in 0..y_width {
                let y = f32::from(y_plane.get(col, row).unwrap().0);
                let cr = f32::from(cr_plane_up.get(col, row).unwrap().0) - 128.0;
                let cb = f32::from(cb_plane_up.get(col, row).unwrap().0) - 128.0;

                let px = output.get_mut(col, row).unwrap();
                px.r = (y + 1.402 * cr).round().clamp(0.0, 255.0) as u8;
                px.g = (y - 0.344136 * cb - 0.714136 * cr)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                px.b = (y + 1.772 * cb).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Receive and extend a coefficients, converting it to a signed value.
/// Section F.2.2.1 of ITU-T81 describe this procedure.
#[inline(always)]
fn huffman_receive_extend(bitreader: &mut BitReader<DataReader>, size: u8) -> JpegResult<i16> {
    if size == 0 {
        return Ok(0);
    }

    let value = bitreader.read_bits(size)? as i16;
    let vt = 1 << (size - 1);

    Ok(if value < vt {
        value - ((1 << size) - 1)
    } else {
        value
    })
}

impl ComponentPlane {
    fn new(component: &Component) -> Self {
        let width = usize::from(component.horizontal_sampling) * 8;
        let height = usize::from(component.vertical_sampling) * 8;

        Self {
            data: vec![0; width * height],
            width,
            height,
        }
    }
}

impl Component {
    pub fn index(&self) -> usize {
        match self.id {
            ComponentId::Y => 0,
            ComponentId::Cb => 1,
            ComponentId::Cr => 2,
        }
    }
}

impl<'a> DataReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, idx: 0 }
    }
}

impl<'a> Read for DataReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n = 0;
        let mut src = self.data[self.idx..].iter();
        let mut dst = buf.iter_mut();

        while let (Some(&a), Some(b)) = (src.next(), dst.next()) {
            self.idx += 1;

            if a == 0xFF {
                self.idx += 1;

                match src.next() {
                    Some(0) => *b = 0xFF,
                    Some(0xD9) | None => break,
                    _ => panic!(),
                }
            } else {
                *b = a;
            }

            n += 1;
        }

        Ok(n)
    }
}

impl<'a> BufRead for DataReader<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(&self.data[self.idx..])
    }

    fn consume(&mut self, amount: usize) {
        self.idx += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_reader_simple() {
        let mut reader = DataReader::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut buf = [0; 6];

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 6);
        assert_eq!(&buf, &[0, 1, 2, 3, 4, 5]);

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], &[6, 7, 8, 9]);
    }

    #[test]
    fn data_reader_with_markers() {
        let mut reader = DataReader::new(&[1, 0xFF, 0, 3, 4, 5, 6, 7, 0xFF, 0xD9]);

        let mut buf = [0; 5];

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, &[1, 0xFF, 3, 4, 5]);

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&buf[..2], &[6, 7]);
    }

    #[test]
    fn data_reader_with_bit_reader() {
        let reader = DataReader::new(&[0b0001_1010, 0xFF, 0x00, 0b1100_0000]);
        let mut bit_reader = BitReader::new(reader);

        assert_eq!(bit_reader.read_bits(4).unwrap(), 0b0001);
        assert_eq!(bit_reader.read_bits(8).unwrap(), 0b1010_1111);
        assert_eq!(bit_reader.read_bits(8).unwrap(), 0b1111_1100);
        assert_eq!(bit_reader.read_bits(4).unwrap(), 0b0000);
    }
}
