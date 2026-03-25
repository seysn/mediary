use std::io::{self, BufWriter, Cursor, Write};

use mediary_common::{
    bitwriter::BitWriter,
    huffman::{HuffmanCode, HuffmanTable},
};
use mediary_image::{PackedImageRead, RgbImage, mono::MonoImageRef};
use mediary_yuv::{YuvChromaSubsampling, YuvPlanarImage};

use crate::{
    JpegResult, RawJpeg,
    dct::forward::dct_naive,
    huffman::{
        DEFAULT_CHROMA_AC_TABLE, DEFAULT_CHROMA_DC_TABLE, DEFAULT_LUMA_AC_TABLE,
        DEFAULT_LUMA_DC_TABLE,
    },
    marker::{
        ComponentId, DefineHuffmanTable, DefineQuantizationTable, ImageData, QuantizationTable,
        QuantizationTableValues, SofComponent, SosComponent, StartOfFrame, StartOfScan, TableClass,
    },
};

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

const LUMA_TABLE_INDEX: u8 = 0;
const CHROMA_TABLE_INDEX: u8 = 1;

pub struct JpegEncoder {
    luma_quantization_table: QuantizationTableValues,
    chroma_quantization_table: QuantizationTableValues,
}

struct Component<'a> {
    id: ComponentId,
    blocks: Vec<[i16; 64]>,
    horizontal_sampling: usize,
    vertical_sampling: usize,
    dc_table: &'a HuffmanTable,
    ac_table: &'a HuffmanTable,
    plane: MonoImageRef<'a>,
}

struct DataWriter<W: Write> {
    writer: BufWriter<W>,
}

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
        ) = match input.subsampling() {
            YuvChromaSubsampling::Yuv444 => (1, 1, 1, 1),
            YuvChromaSubsampling::Yuv420 => (2, 2, 1, 1),
        };

        let luma_dc_table = &*DEFAULT_LUMA_DC_TABLE;
        let luma_ac_table = &*DEFAULT_LUMA_AC_TABLE;
        let chroma_dc_table = &*DEFAULT_CHROMA_DC_TABLE;
        let chroma_ac_table = &*DEFAULT_CHROMA_AC_TABLE;
        let mut components = [
            Component {
                id: ComponentId::Y,
                blocks: Vec::new(),
                horizontal_sampling: luma_horizontal_sampling,
                vertical_sampling: luma_vertical_sampling,
                dc_table: luma_dc_table,
                ac_table: luma_ac_table,
                plane: y_plane,
            },
            Component {
                id: ComponentId::Cb,
                blocks: Vec::new(),
                horizontal_sampling: chroma_horizontal_sampling,
                vertical_sampling: chroma_vertical_sampling,
                dc_table: chroma_dc_table,
                ac_table: chroma_ac_table,
                plane: u_plane,
            },
            Component {
                id: ComponentId::Cr,
                blocks: Vec::new(),
                horizontal_sampling: chroma_horizontal_sampling,
                vertical_sampling: chroma_vertical_sampling,
                dc_table: chroma_dc_table,
                ac_table: chroma_ac_table,
                plane: v_plane,
            },
        ];

        let mcu_width = width / (8 * luma_horizontal_sampling);
        let mcu_height = height / (8 * luma_vertical_sampling);
        for mcu_y in 0..mcu_height {
            for mcu_x in 0..mcu_width {
                self.encode_mcu(mcu_x, mcu_y, &mut components);
            }
        }

        let data = Cursor::new(Vec::new());
        let mut previous_dcs: Vec<i16> = vec![0; components.len()];
        let mut bitwriter = BitWriter::new(DataWriter::new(data));
        for i in 0..mcu_height * mcu_width {
            for (component, last_dc) in components.iter().zip(previous_dcs.iter_mut()) {
                let n_blocks = component.horizontal_sampling * component.vertical_sampling;
                for j in 0..n_blocks {
                    let index = i * n_blocks + j;
                    *last_dc = component.write_block(index, *last_dc, &mut bitwriter);
                }
            }
        }
        let data = bitwriter
            .into_writer()
            .writer
            .into_inner()
            .unwrap()
            .into_inner();

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
                        horizontal_sampling: chroma_horizontal_sampling as u8,
                        vertical_sampling: chroma_vertical_sampling as u8,
                        quantization_table: CHROMA_TABLE_INDEX,
                    },
                    SofComponent {
                        id: ComponentId::Cr,
                        horizontal_sampling: chroma_horizontal_sampling as u8,
                        vertical_sampling: chroma_vertical_sampling as u8,
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
            huffman_tables: vec![
                DefineHuffmanTable::from_table(TableClass::DC, 0, luma_dc_table),
                DefineHuffmanTable::from_table(TableClass::DC, 1, chroma_dc_table),
                DefineHuffmanTable::from_table(TableClass::AC, 0, luma_ac_table),
                DefineHuffmanTable::from_table(TableClass::AC, 1, chroma_ac_table),
            ],
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

    fn encode_mcu(&self, mcu_x: usize, mcu_y: usize, components: &mut [Component]) {
        tracing::debug!("Encoding MCU ({mcu_x}, {mcu_y})");
        for component in components {
            self.encode_component(
                mcu_x * component.horizontal_sampling * 8,
                mcu_y * component.vertical_sampling * 8,
                component,
            );
        }
    }

    fn encode_component(&self, x_padding: usize, y_padding: usize, component: &mut Component) {
        tracing::debug!(
            "Encoding Component {:?} ({}px x {}px)",
            component.id,
            component.horizontal_sampling * 8,
            component.vertical_sampling * 8
        );
        let mut dct_output = [0; 64];
        for block_y in 0..component.vertical_sampling {
            for block_x in 0..component.horizontal_sampling {
                tracing::debug!("Encoding block ({block_x}, {block_y})");

                let x = x_padding + block_x * 8;
                let y = y_padding + block_y * 8;
                let block_view = component.plane.view(x, y, 8, 8).unwrap();

                dct_naive(&block_view, &mut dct_output);

                let mut block = [0; 64];
                let quantization_table = if component.id == ComponentId::Y {
                    &self.luma_quantization_table
                } else {
                    &self.chroma_quantization_table
                };

                for (i, coeff) in dct_output.iter().enumerate() {
                    block[ZIGZAG[i]] = *coeff / i16::from(quantization_table[ZIGZAG[i]]);
                }
                component.blocks.push(block);
            }
        }
    }
}

impl Component<'_> {
    fn write_block<W: io::Write>(
        &self,
        index: usize,
        last_dc: i16,
        bitwriter: &mut BitWriter<W>,
    ) -> i16 {
        tracing::debug!("Write {:?} block {index}", self.id);
        let block = &self.blocks[index];
        let dc = block[0];
        let dc_difference = dc - last_dc;
        if dc_difference == 0 {
            let code = self.dc_table.get_code(0).unwrap();
            bitwriter.write_code(code).unwrap();
        } else {
            let code = HuffmanCode::from_bitcode(dc_difference);
            let size = self.dc_table.get_code(code.size).unwrap();
            bitwriter.write_code(size).unwrap();
            bitwriter.write_code(&code).unwrap();
        }

        let mut zero_count = 0;
        for &ac in &block[1..] {
            if ac == 0 {
                zero_count += 1;
                continue;
            }

            while zero_count >= 16 {
                let code = self.ac_table.get_code(0).unwrap();
                bitwriter.write_code(code).unwrap();
                zero_count -= 16;
            }

            let ac_code = HuffmanCode::from_bitcode(ac);
            let run_length = (zero_count << 4) | ac_code.size;
            let run_length_code = self.ac_table.get_code(run_length).unwrap();
            bitwriter.write_code(run_length_code).unwrap();
            bitwriter.write_code(&ac_code).unwrap();
            zero_count = 0;
        }

        // If there are still leading zeros when the block ends, write an EOB marker
        if zero_count > 0 {
            let code = self.ac_table.get_code(0).unwrap();
            bitwriter.write_code(code).unwrap();
        }

        dc
    }
}

impl<W: Write> DataWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
        }
    }
}

impl<W: Write> Write for DataWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &byte in buf {
            if byte == 0xFF {
                self.writer.write_all(&[0xFF, 0x00])?;
            } else {
                self.writer.write_all(&[byte])?;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
