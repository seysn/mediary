use std::ops::{Index, IndexMut};

use crate::{ImageMut, ImageProperties, ImageRef};

pub struct ImageView<Img> {
    pub(crate) image: Img,

    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub type ImageViewRef<'a, Img> = ImageView<&'a Img>;
pub type ImageViewMut<'a, Img> = ImageView<&'a mut Img>;

impl<'a, Img: ImageProperties> ImageProperties for ImageViewRef<'a, Img> {
    type Pixel = Img::Pixel;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<'a, Img: ImageProperties> ImageProperties for ImageViewMut<'a, Img> {
    type Pixel = Img::Pixel;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<'a, Img: ImageRef> ImageRef for ImageViewRef<'a, Img> {
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        unsafe { self.image.get_pixel_unchecked(self.x + x, self.y + y) }
    }
}

impl<'a, Img: ImageRef> ImageRef for ImageViewMut<'a, Img> {
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        unsafe { self.image.get_pixel_unchecked(self.x + x, self.y + y) }
    }
}

impl<'a, Img: ImageMut> ImageMut for ImageViewMut<'a, Img> {
    unsafe fn get_pixel_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Self::Pixel {
        unsafe { self.image.get_pixel_mut_unchecked(self.x + x, self.y + y) }
    }
}

impl<'a, Img: ImageRef> Index<(usize, usize)> for ImageViewRef<'a, Img> {
    type Output = Img::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get_pixel(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<'a, Img: ImageRef> Index<(usize, usize)> for ImageViewMut<'a, Img> {
    type Output = Img::Pixel;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get_pixel(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<'a, Img: ImageRef + ImageMut> IndexMut<(usize, usize)> for ImageViewMut<'a, Img> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let width = self.width;
        let height = self.height;

        match self.get_pixel_mut(x, y) {
            Some(p) => p,
            None => panic!("Position ({x}, {y}) is out of bounds ({width}, {height})",),
        }
    }
}
