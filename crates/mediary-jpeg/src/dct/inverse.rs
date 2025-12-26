//! Inverse Discrete Cosine Transform (IDCT) functions
#![allow(clippy::needless_range_loop, clippy::approx_constant)]

use std::f64::consts::PI;

use crate::dct::{ALPHA_TABLE, COS_TABLE};

/// Precomputed implementation of Inverse Discrete Cosine Transform (IDCT) using separatable
/// one dimension IDCT table with two pass.
///
/// This is around five times faster than [precomputed](crate::dct::idct_precomputed) implementation.
pub fn idct_two_pass(
    input: &[i16],
    output: &mut [u8],
    stride: usize,
    block_x: usize,
    block_y: usize,
) {
    let mut tmp = [[0f64; 8]; 8];

    // First pass: for each vertical frequency and output column
    for v in 0..8 {
        for col in 0..8 {
            for u in 0..8 {
                let cu = ALPHA_TABLE[u];
                let coeff = input[v * 8 + u] as f64;
                let cos_x = COS_TABLE[col][u];

                tmp[v][col] += cu * coeff * cos_x;
            }
        }
    }

    // Second pass: for each output column and output row
    for row in 0..8 {
        for col in 0..8 {
            let mut sum = 0.0f64;

            for v in 0..8 {
                let cv = ALPHA_TABLE[v];
                let cos_y = COS_TABLE[row][v];

                sum += cv * tmp[v][col] * cos_y;
            }

            // Multiply by normalization factor (1/4)
            sum *= 0.25;

            // Shift by +128 to return to 0..255 range
            let val = (sum.round() + 128.0).clamp(0.0, 255.0) as u8;

            let idx = 8 * stride * block_y + stride * row + 8 * block_x + col;
            output[idx] = val;
        }
    }
}

/// Precomputed implementation of Inverse Discrete Cosine Transform (IDCT)
///
/// This is around three times faster than [naive](crate::dct::idct_naive) implementation
pub fn idct_precomputed(
    input: &[i16],
    output: &mut [u8],
    stride: usize,
    block_x: usize,
    block_y: usize,
) {
    for row in 0..8 {
        for col in 0..8 {
            let mut sum = 0.0f64;

            for v in 0..8 {
                for u in 0..8 {
                    let cu = ALPHA_TABLE[u];
                    let cv = ALPHA_TABLE[v];

                    let coeff = input[v * 8 + u] as f64;
                    let cos_x = COS_TABLE[col][u];
                    let cos_y = COS_TABLE[row][v];

                    sum += cu * cv * coeff * cos_x * cos_y;
                }
            }

            // Multiply by normalization factor (1/4)
            sum *= 0.25;

            // Shift by +128 to return to 0..255 range
            let val = (sum.round() + 128.0).clamp(0.0, 255.0) as u8;

            let idx = 8 * stride * block_y + stride * row + 8 * block_x + col;
            output[idx] = val;
        }
    }
}

/// Naive implementation of Inverse Discrete Cosine Transform (IDCT)
pub fn idct_naive(input: &[i16], output: &mut [u8], stride: usize, block_x: usize, block_y: usize) {
    for row in 0..8 {
        for col in 0..8 {
            let mut sum = 0.0;

            for v in 0..8 {
                for u in 0..8 {
                    let cu = if u == 0 { 1.0 / f64::sqrt(2.0) } else { 1.0 };
                    let cv = if v == 0 { 1.0 / f64::sqrt(2.0) } else { 1.0 };

                    let coeff = input[v * 8 + u] as f64;
                    let cos_x = f64::cos(((2 * col + 1) as f64 * u as f64 * PI) / 16.0);
                    let cos_y = f64::cos(((2 * row + 1) as f64 * v as f64 * PI) / 16.0);

                    sum += cu * cv * coeff * cos_x * cos_y;
                }
            }

            // Multiply by normalization factor (1/4)
            sum *= 0.25;

            // Shift by +128 to return to 0..255 range
            let val = (sum.round() + 128.0).clamp(0.0, 255.0) as u8;

            let idx = 8 * stride * block_y + stride * row + 8 * block_x + col;
            output[idx] = val;
        }
    }
}
