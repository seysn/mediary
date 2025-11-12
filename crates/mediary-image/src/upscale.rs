use std::ops::Index;

use crate::ImageSource;

pub struct ImageUpscale<'a, T> {
    pub(crate) image: &'a T,

    pub(crate) upscale_x: usize,
    pub(crate) upscale_y: usize,
}

impl<T: ImageSource> ImageSource for ImageUpscale<'_, T> {
    type Pixel = T::Pixel;

    fn width(&self) -> usize {
        self.image.width() * self.upscale_x
    }

    fn height(&self) -> usize {
        self.image.height() * self.upscale_y
    }

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        unsafe {
            self.image
                .get_unchecked(x / self.upscale_x, y / self.upscale_y)
        }
    }
}

impl<T: ImageSource> Index<(usize, usize)> for ImageUpscale<'_, T> {
    type Output = T::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
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
    use crate::{ImageSource, MonoImage};

    #[test]
    fn upscale_double() {
        let image = MonoImage::new(vec![1, 2, 3, 4], 2, 2).unwrap();
        let image_up = image.upscale(2, 2);

        assert_eq!(image.get(0, 0).unwrap().0, 1);
        assert_eq!(image.get(1, 0).unwrap().0, 2);
        assert_eq!(image.get(0, 1).unwrap().0, 3);
        assert_eq!(image.get(1, 1).unwrap().0, 4);

        assert_eq!(image_up.get(0, 0).unwrap().0, 1);
        assert_eq!(image_up.get(1, 0).unwrap().0, 1);
        assert_eq!(image_up.get(0, 1).unwrap().0, 1);
        assert_eq!(image_up.get(1, 1).unwrap().0, 1);

        assert_eq!(image_up.get(2, 0).unwrap().0, 2);
        assert_eq!(image_up.get(3, 0).unwrap().0, 2);
        assert_eq!(image_up.get(2, 1).unwrap().0, 2);
        assert_eq!(image_up.get(3, 1).unwrap().0, 2);

        assert_eq!(image_up.get(0, 2).unwrap().0, 3);
        assert_eq!(image_up.get(1, 2).unwrap().0, 3);
        assert_eq!(image_up.get(0, 3).unwrap().0, 3);
        assert_eq!(image_up.get(1, 3).unwrap().0, 3);

        assert_eq!(image_up.get(2, 2).unwrap().0, 4);
        assert_eq!(image_up.get(3, 2).unwrap().0, 4);
        assert_eq!(image_up.get(2, 3).unwrap().0, 4);
        assert_eq!(image_up.get(3, 3).unwrap().0, 4);
    }
}
