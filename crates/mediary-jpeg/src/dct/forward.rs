use std::f64::consts::PI;

use mediary_image::{BaseImage, PackedImageRead, mono::MonoPixel, view::ImageView};

pub trait DctInput {
    fn get(&self, row: usize, col: usize) -> f64;
}

impl DctInput for [u8] {
    fn get(&self, row: usize, col: usize) -> f64 {
        self[row * 8 + col] as f64
    }
}

impl DctInput for ImageView<&BaseImage<MonoPixel, &[u8]>> {
    fn get(&self, row: usize, col: usize) -> f64 {
        let px = unsafe { self.get_pixel_unchecked(col, row) };
        px.0 as f64
    }
}

pub fn dct_naive<I>(input: &I, output: &mut [i16])
where
    I: DctInput + ?Sized,
{
    for v in 0..8 {
        for u in 0..8 {
            let mut sum = 0.0f64;

            for row in 0..8 {
                for col in 0..8 {
                    let pixel = input.get(row, col) - 128.0;
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

#[cfg(test)]
mod tests {
    use mediary_image::{PackedImageRead, mono::MonoImageRef};

    use crate::dct::forward::DctInput;

    #[test]
    fn array_input() {
        let mut array = [0; 64];
        array[0] = 11;
        array[2] = 22;
        array[16] = 33;
        array[18] = 44;
        assert_eq!(DctInput::get(array.as_slice(), 0, 0), 11.0);
        assert_eq!(DctInput::get(array.as_slice(), 0, 2), 22.0);
        assert_eq!(DctInput::get(array.as_slice(), 2, 0), 33.0);
        assert_eq!(DctInput::get(array.as_slice(), 2, 2), 44.0);
    }

    #[test]
    fn view_input() {
        let mut array = [0; 64];
        array[0] = 11;
        array[2] = 22;
        array[16] = 33;
        array[18] = 44;

        let image_ref = MonoImageRef::new(&array, 8, 8).unwrap();
        let view = image_ref.view(0, 0, 8, 8).unwrap();

        assert_eq!(DctInput::get(&view, 0, 0), 11.0);
        assert_eq!(DctInput::get(&view, 0, 2), 22.0);
        assert_eq!(DctInput::get(&view, 2, 0), 33.0);
        assert_eq!(DctInput::get(&view, 2, 2), 44.0);
    }
}
