use crate::{
    error::{ImageError, ImageResult},
    ImageSource, ImageSourceMut,
};

pub struct ImageView<'a, T> {
    pub(crate) rgb: &'a T,

    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub struct ImageViewMut<'a, T> {
    pub(crate) rgb: &'a mut T,

    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl<T: ImageSource> ImageSource for ImageView<'_, T> {
    type Pixel = T::Pixel;

    fn get(&self, x: usize, y: usize) -> Option<T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.get(x, y)
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, Self> {
        todo!()
    }
}

impl<T: ImageSource> ImageSource for ImageViewMut<'_, T> {
    type Pixel = T::Pixel;

    fn get(&self, x: usize, y: usize) -> Option<T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.get(x, y)
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, Self> {
        todo!()
    }
}

impl<T: ImageSourceMut> ImageSourceMut for ImageViewMut<'_, T> {
    fn set(&mut self, x: usize, y: usize, value: Self::Pixel) -> ImageResult<()> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.set(x, y, value)
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
    ) -> ImageViewMut<'_, Self> {
        todo!()
    }
}
