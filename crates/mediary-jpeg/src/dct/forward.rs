use std::f64::consts::PI;

pub fn dct_naive(input: &[u8], output: &mut [i16]) {
    for v in 0..8 {
        for u in 0..8 {
            let mut sum = 0.0f64;

            for row in 0..8 {
                for col in 0..8 {
                    let pixel = input[row * 8 + col] as f64;
                    let cos_u = f64::cos(((2 * col + 1) as f64 * u as f64 * PI) / 16.0);
                    let cos_v = f64::cos(((2 * row + 1) as f64 * v as f64 * PI) / 16.0);
                    sum += pixel * cos_u * cos_v;
                }
            }

            let cu = if u == 0 { 1.0 / f64::sqrt(2.0) } else { 1.0 };
            let cv = if v == 0 { 1.0 / f64::sqrt(2.0) } else { 1.0 };

            let coeff = 0.25 * cu * cv * sum;
            output[v * 8 + u] = coeff
                .round()
                .clamp(f64::from(i16::MIN), f64::from(i16::MAX))
                as i16;
        }
    }
}
