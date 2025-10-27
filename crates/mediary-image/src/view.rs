use std::ops::{Index, IndexMut};

use crate::{ImageSource, ImageSourceMut};

pub struct ImageView<'a, T> {
    pub(crate) image: &'a T,

    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub struct ImageViewMut<'a, T> {
    pub(crate) image: &'a mut T,

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

    fn get(&self, x: usize, y: usize) -> Option<&T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.image.get(x, y)
        } else {
            None
        }
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

    fn get(&self, x: usize, y: usize) -> Option<&T::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.image.get(x, y)
        } else {
            None
        }
    }
}

impl<T: ImageSourceMut> ImageSourceMut for ImageViewMut<'_, T> {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel> {
        if x < self.width && y < self.height {
            let x = self.x + x;
            let y = self.y + y;

            self.image.get_mut(x, y)
        } else {
            None
        }
    }
}

impl<T: ImageSource> Index<(usize, usize)> for ImageView<'_, T> {
    type Output = T::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<T: ImageSource> Index<(usize, usize)> for ImageViewMut<'_, T> {
    type Output = T::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<T: ImageSourceMut> IndexMut<(usize, usize)> for ImageViewMut<'_, T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let width = self.width;
        let height = self.height;

        match self.get_mut(x, y) {
            Some(p) => p,
            None => panic!("Position ({x}, {y}) is out of bounds ({width}, {height})",),
        }
    }
}
