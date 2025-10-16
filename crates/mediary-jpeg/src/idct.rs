//! Inverse Discrete Cosine Transform (IDCT) functions
#![allow(clippy::needless_range_loop, clippy::approx_constant)]

use std::f64::consts::PI;

/// Precomputed cosine table
///
/// This is the equivalent of running :
/// ```no_run
/// for y in 0..8 {
///     for x in 0..8 {
///         table[y][x] = f64::cos(((2 * y + 1) as f64 * x as f64 * PI) / 16.0);
///     }
/// }
/// ```
#[rustfmt::skip]
const COS_TABLE: [[f64; 8]; 8] = [
    [1.0, 0.9807852804032304, 0.9238795325112867, 0.8314696123025452, 0.7071067811865476, 0.5555702330196023, 0.38268343236508984, 0.19509032201612833],
    [1.0, 0.8314696123025452, 0.38268343236508984, -0.1950903220161282, -0.7071067811865475, -0.9807852804032304, -0.9238795325112868, -0.5555702330196022],
    [1.0, 0.5555702330196023, -0.3826834323650897, -0.9807852804032304, -0.7071067811865477, 0.1950903220161283, 0.9238795325112865, 0.8314696123025455],
    [1.0, 0.19509032201612833, -0.9238795325112867, -0.5555702330196022, 0.7071067811865474, 0.8314696123025455, -0.3826834323650899, -0.9807852804032307],
    [1.0, -0.1950903220161282, -0.9238795325112868, 0.5555702330196018, 0.7071067811865477, -0.8314696123025451, -0.38268343236509056, 0.9807852804032304],
    [1.0, -0.555570233019602, -0.38268343236509034, 0.9807852804032304, -0.7071067811865467, -0.19509032201612803, 0.9238795325112867, -0.831469612302545],
    [1.0, -0.8314696123025453, 0.38268343236509, 0.19509032201612878, -0.7071067811865471, 0.9807852804032307, -0.9238795325112864, 0.5555702330196015],
    [1.0, -0.9807852804032304, 0.9238795325112865, -0.8314696123025451, 0.7071067811865466, -0.5555702330196015, 0.38268343236508956, -0.19509032201612858],
];

/// Precomputed alpha table
///
/// First element is the result of `1.0 / f64::sqrt(2.0)`
const ALPHA: [f64; 8] = [0.7071067811865475, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];

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
                let cu = ALPHA[u];
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
                let cv = ALPHA[v];
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
                    let cu = ALPHA[u];
                    let cv = ALPHA[v];

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
