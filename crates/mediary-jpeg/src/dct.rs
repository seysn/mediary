//! Discrete Cosine Transform (DCT) functions

use std::f64::consts::PI;

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
