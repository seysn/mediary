use std::ops::Index;

use crate::{ImageProperties, PackedImageRead};

pub struct ImageUpscale<Img> {
    pub(crate) image: Img,

    pub(crate) upscale_x: usize,
    pub(crate) upscale_y: usize,
}

pub type ImageUpscaleRef<'a, Img> = ImageUpscale<&'a Img>;

impl<'a, Img: ImageProperties> ImageProperties for ImageUpscaleRef<'a, Img> {
    type Pixel = Img::Pixel;

    fn width(&self) -> usize {
        self.image.width() * self.upscale_x
    }

    fn height(&self) -> usize {
        self.image.height() * self.upscale_y
    }
}

impl<'a, Img: PackedImageRead> PackedImageRead for ImageUpscaleRef<'a, Img> {
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        unsafe {
            self.image
                .get_pixel_unchecked(x / self.upscale_x, y / self.upscale_y)
        }
    }
}

impl<'a, Img: PackedImageRead> Index<(usize, usize)> for ImageUpscaleRef<'a, Img> {
    type Output = Img::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get_pixel(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width(),
                self.height()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{PackedImageRead as _, MonoImage};

    #[test]
    fn upscale_double() {
        let image = MonoImage::new(vec![1, 2, 3, 4], 2, 2).unwrap();
        let image_up = image.upscale(2, 2);

        assert_eq!(image.get_pixel(0, 0).unwrap().0, 1);
        assert_eq!(image.get_pixel(1, 0).unwrap().0, 2);
        assert_eq!(image.get_pixel(0, 1).unwrap().0, 3);
        assert_eq!(image.get_pixel(1, 1).unwrap().0, 4);

        assert_eq!(image_up.get_pixel(0, 0).unwrap().0, 1);
        assert_eq!(image_up.get_pixel(1, 0).unwrap().0, 1);
        assert_eq!(image_up.get_pixel(0, 1).unwrap().0, 1);
        assert_eq!(image_up.get_pixel(1, 1).unwrap().0, 1);

        assert_eq!(image_up.get_pixel(2, 0).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(3, 0).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(2, 1).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(3, 1).unwrap().0, 2);

        assert_eq!(image_up.get_pixel(0, 2).unwrap().0, 3);
        assert_eq!(image_up.get_pixel(1, 2).unwrap().0, 3);
        assert_eq!(image_up.get_pixel(0, 3).unwrap().0, 3);
        assert_eq!(image_up.get_pixel(1, 3).unwrap().0, 3);

        assert_eq!(image_up.get_pixel(2, 2).unwrap().0, 4);
        assert_eq!(image_up.get_pixel(3, 2).unwrap().0, 4);
        assert_eq!(image_up.get_pixel(2, 3).unwrap().0, 4);
        assert_eq!(image_up.get_pixel(3, 3).unwrap().0, 4);
    }
}
