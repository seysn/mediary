//! Discrete Cosine Transform (DCT) functions
#![allow(clippy::needless_range_loop)]

use std::{f64::consts::PI, sync::LazyLock};

/// Lazy-computed cosine table
static COS_TABLE: LazyLock<[[f64; 8]; 8]> = LazyLock::new(|| {
    let mut table = [[0.0f64; 8]; 8];

    for y in 0..8 {
        for x in 0..8 {
            table[y][x] = f64::cos(((2 * y + 1) as f64 * x as f64 * PI) / 16.0);
        }
    }

    table
});

/// Lazy-computed alpha table
static ALPHA: LazyLock<[f64; 8]> = LazyLock::new(|| {
    let mut table = [1.0f64; 8];
    table[0] = 1.0 / f64::sqrt(2.0);
    table
});

/// Precomputed implementation of Inverse Discrete Cosine Transform (IDCT) using separatable
/// one dimension IDCT table with two passes.
pub fn idct_two_pass(
    input: &mut [i16],
    output: &mut [u8],
    stride: usize,
    block_x: usize,
    block_y: usize,
) {
    let cos = &*COS_TABLE;
    let alpha = &*ALPHA;
    let mut tmp = [[0f64; 8]; 8];

    // First pass: for each vertical frequency and output column
    for v in 0..8 {
        for col in 0..8 {
            for u in 0..8 {
                let cu = alpha[u];
                let coeff = input[v * 8 + u] as f64;
                let cos_x = cos[col][u];

                tmp[v][col] += cu * coeff * cos_x;
            }
        }
    }

    // Second pass: for each output column and output row
    for row in 0..8 {
        for col in 0..8 {
            let mut sum = 0.0f64;

            for v in 0..8 {
                let cv = alpha[v];
                let cos_y = cos[row][v];

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
    input: &mut [i16],
    output: &mut [u8],
    stride: usize,
    block_x: usize,
    block_y: usize,
) {
    let cos = &*COS_TABLE;
    let alpha = &*ALPHA;

    for row in 0..8 {
        for col in 0..8 {
            let mut sum = 0.0f64;

            for v in 0..8 {
                for u in 0..8 {
                    let cu = alpha[u];
                    let cv = alpha[v];

                    let coeff = input[v * 8 + u] as f64;
                    let cos_x = cos[col][u];
                    let cos_y = cos[row][v];

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
pub fn idct_naive(
    input: &mut [i16],
    output: &mut [u8],
    stride: usize,
    block_x: usize,
    block_y: usize,
) {
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
