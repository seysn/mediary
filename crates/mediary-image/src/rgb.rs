use crate::{
    view::{ImageView, ImageViewMut},
    ImageSource, ImageSourceMut,
};

pub struct RgbImage {
    data: Vec<u8>,

    width: usize,
    height: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub type RgbImageView<'a> = ImageView<'a, RgbImage>;
pub type RgbImageViewMut<'a> = ImageViewMut<'a, RgbImage>;

impl RgbImage {
    pub fn new(data: Vec<u8>, width: usize, height: usize) -> Option<Self> {
        if data.len() == width * height * 3 {
            Some(Self {
                data,
                width,
                height,
            })
        } else {
            None
        }
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

impl ImageSource for RgbImage {
    type Pixel = RgbPixel;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<RgbPixel> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            Some(RgbPixel {
                r: self.data[idx * 3],
                g: self.data[idx * 3 + 1],
                b: self.data[idx * 3 + 2],
            })
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<RgbImageView<'_>> {
        if x + width <= self.width && y + height <= self.height {
            Some(RgbImageView {
                rgb: self,
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }
}

impl ImageSourceMut for RgbImage {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut RgbPixel> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            // SAFETY: Dimensions are checked when creating RgbImage
            unsafe {
                let ptr = self.data.as_mut_ptr().add(idx * 3) as *mut RgbPixel;
                Some(&mut *ptr)
            }
        } else {
            None
        }
    }

    fn view_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<RgbImageViewMut<'_>> {
        if x + width <= self.width && y + height <= self.height {
            Some(RgbImageViewMut {
                rgb: self,
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_pixel_get_mut() {
        let mut image = RgbImage {
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            width: 2,
            height: 2,
        };

        let px = image.get_mut(0, 0).unwrap();
        assert_eq!(px.r, 1);
        assert_eq!(px.g, 2);
        assert_eq!(px.b, 3);
        px.g += 10;

        let px = image.get_mut(1, 0).unwrap();
        assert_eq!(px.r, 4);
        assert_eq!(px.g, 5);
        assert_eq!(px.b, 6);
        px.b += 20;

        let px = image.get_mut(0, 1).unwrap();
        assert_eq!(px.r, 7);
        assert_eq!(px.g, 8);
        assert_eq!(px.b, 9);
        px.r += 3;

        let px = image.get_mut(1, 1).unwrap();
        assert_eq!(px.r, 10);
        assert_eq!(px.g, 11);
        assert_eq!(px.b, 12);
        px.r += 1;
        px.g = 2;
        px.b *= 3;

        assert_eq!(image.data, vec![1, 12, 3, 4, 5, 26, 10, 8, 9, 11, 2, 36]);
    }
}
