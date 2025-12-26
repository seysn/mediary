use mediary_image::{
    PackedImageRead, Pixel, RgbImage,
    mono::{MonoImageRef, MonoPixel},
};
use mediary_yuv::{YuvChromaSubsampling, YuvPlanarImage};

use crate::{
    JpegResult, RawJpeg,
    dct::forward::dct_naive,
    marker::{
        ComponentId, DefineQuantizationTable, ImageData, QuantizationTable,
        QuantizationTableValues, SofComponent, SosComponent, StartOfFrame, StartOfScan,
    },
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

        let mcu_width = width / (8 * luma_horizontal_sampling);
        let mcu_height = height / (8 * luma_vertical_sampling);
        for block_y in 0..mcu_height {
            for block_x in 0..mcu_width {
                let block_width = 8 * luma_horizontal_sampling;
                let block_height = 8 * luma_vertical_sampling;

                for luma_y in 0..luma_vertical_sampling {
                    for luma_x in 0..luma_horizontal_sampling {
                        let sub_block_x = block_x * 8 * luma_x;
                        let sub_block_y = block_y * 8 * luma_y;

                        let y_view = y_plane.view(sub_block_x, sub_block_y, 8, 8).unwrap();
                        let y_block = y_view.to_vec();

                        dct_naive(MonoPixel::as_row_slice(&y_block), &mut coeff_pool);

                        for (coeff, quantization) in
                            coeff_pool.iter_mut().zip(&self.luma_quantization_table.0)
                        {
                            *coeff /= i16::from(*quantization);
                        }
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

    fn encode_block(&self, coeff_pool: &[i32]) {
        for idx in ZIGZAG {
            print!("{}", coeff_pool[idx]);
        }
        println!();
    }
}
