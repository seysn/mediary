use crate::{
    error::{ImageError, ImageResult},
    view::{ImageView, ImageViewMut},
    ImageSource, ImageSourceMut,
};

pub struct RgbImage {
    pub data: Vec<u8>,

    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub type RgbImageView<'a> = ImageView<'a, RgbImage>;
pub type RgbImageViewMut<'a> = ImageViewMut<'a, RgbImage>;

impl ImageSource for RgbImage {
    type Pixel = RgbPixel;

    fn get(&self, x: usize, y: usize) -> ImageResult<RgbPixel> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            Ok(RgbPixel {
                r: self.data[idx * 3],
                g: self.data[idx * 3 + 1],
                b: self.data[idx * 3 + 2],
            })
        } else {
            Err(ImageError::OutOfBounds)
        }
    }

    fn view(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageResult<RgbImageView<'_>> {
        if x + width <= self.width && y + height <= self.height {
            Ok(RgbImageView {
                rgb: self,
                x,
                y,
                width,
                height,
            })
        } else {
            Err(ImageError::OutOfBounds)
        }
    }
}

impl ImageSourceMut for RgbImage {
    fn set(&mut self, x: usize, y: usize, value: RgbPixel) -> ImageResult<()> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.data[idx * 3] = value.r;
            self.data[idx * 3 + 1] = value.g;
            self.data[idx * 3 + 2] = value.b;

            Ok(())
        } else {
            Err(ImageError::OutOfBounds)
        }
    }

    fn view_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageResult<RgbImageViewMut<'_>> {
        if x + width <= self.width && y + height <= self.height {
            Ok(RgbImageViewMut {
                rgb: self,
                x,
                y,
                width,
                height,
            })
        } else {
            Err(ImageError::OutOfBounds)
        }
    }
}
