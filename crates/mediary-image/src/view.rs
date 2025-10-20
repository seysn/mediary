use crate::{ImageSource, ImageSourceMut};

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

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.get(x, y)
        } else {
            None
        }
    }

    fn view(
        &self,
        _x: usize,
        _y: usize,
        _width: usize,
        _height: usize,
    ) -> Option<ImageView<'_, Self>> {
        todo!()
    }
}

impl<T: ImageSource> ImageSource for ImageViewMut<'_, T> {
    type Pixel = T::Pixel;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.get(x, y)
        } else {
            None
        }
    }

    fn view(
        &self,
        _x: usize,
        _y: usize,
        _width: usize,
        _height: usize,
    ) -> Option<ImageView<'_, Self>> {
        todo!()
    }
}

impl<T: ImageSourceMut> ImageSourceMut for ImageViewMut<'_, T> {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.rgb.get_mut(x, y)
        } else {
            None
        }
    }

    fn view_mut(
        &mut self,
        _x: usize,
        _y: usize,
        _width: usize,
        _height: usize,
    ) -> Option<ImageViewMut<'_, Self>> {
        todo!()
    }
}
