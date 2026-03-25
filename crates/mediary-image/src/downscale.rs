use std::ops::Index;

use crate::{ImageProperties, PackedImageRead};

pub struct ImageDownscale<Img> {
    pub(crate) image: Img,

    pub(crate) downscale_x: usize,
    pub(crate) downscale_y: usize,
}

pub type ImageDownscaleRef<'a, Img> = ImageDownscale<&'a Img>;

impl<'a, Img: ImageProperties> ImageProperties for ImageDownscaleRef<'a, Img> {
    type Pixel = Img::Pixel;

    fn width(&self) -> usize {
        self.image.width() / self.downscale_x
    }

    fn height(&self) -> usize {
        self.image.height() / self.downscale_y
    }
}

impl<'a, Img: PackedImageRead> PackedImageRead for ImageDownscaleRef<'a, Img> {
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        unsafe {
            self.image
                .get_pixel_unchecked(x * self.downscale_x, y * self.downscale_y)
        }
    }
}

impl<'a, Img: PackedImageRead> Index<(usize, usize)> for ImageDownscaleRef<'a, Img> {
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
    use crate::{MonoImage, PackedImageRead as _};

    #[test]
    fn downscale_double() {
        #[rustfmt::skip]
        let data = vec![
            0, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 0, 1, 2, 3, 4, 5,
            6, 7, 8, 9, 0, 1, 2, 3,
            4, 5, 6, 7, 8, 9, 0, 1,
            2, 3, 4, 5, 6, 7, 8, 9,
            0, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 0, 1, 2, 3, 4, 5,
            6, 7, 8, 9, 0, 1, 2, 3,
        ];
        let image = MonoImage::new(data, 8, 8).unwrap();
        let image_up = image.downscale(2, 2);

        assert_eq!(image.get_pixel(0, 0).unwrap().0, 0);
        assert_eq!(image.get_pixel(1, 0).unwrap().0, 1);
        assert_eq!(image.get_pixel(0, 1).unwrap().0, 8);
        assert_eq!(image.get_pixel(1, 1).unwrap().0, 9);

        assert_eq!(image_up.get_pixel(0, 0).unwrap().0, 0);
        assert_eq!(image_up.get_pixel(1, 0).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(0, 1).unwrap().0, 6);
        assert_eq!(image_up.get_pixel(1, 1).unwrap().0, 8);

        assert_eq!(image_up.get_pixel(2, 0).unwrap().0, 4);
        assert_eq!(image_up.get_pixel(3, 0).unwrap().0, 6);
        assert_eq!(image_up.get_pixel(2, 1).unwrap().0, 0);
        assert_eq!(image_up.get_pixel(3, 1).unwrap().0, 2);

        assert_eq!(image_up.get_pixel(0, 2).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(1, 2).unwrap().0, 4);
        assert_eq!(image_up.get_pixel(0, 3).unwrap().0, 8);
        assert_eq!(image_up.get_pixel(1, 3).unwrap().0, 0);

        assert_eq!(image_up.get_pixel(2, 2).unwrap().0, 6);
        assert_eq!(image_up.get_pixel(3, 2).unwrap().0, 8);
        assert_eq!(image_up.get_pixel(2, 3).unwrap().0, 2);
        assert_eq!(image_up.get_pixel(3, 3).unwrap().0, 4);
    }
}
