use std::ops::{Index, IndexMut};

use crate::{BaseImage, ImageMut, ImageRef, Pixel};

pub type ImagePacked<Px> = BaseImage<Px, Vec<u8>>;
pub type ImagePackedRef<'a, Px> = BaseImage<Px, &'a [u8]>;
pub type ImagePackedMut<'a, Px> = BaseImage<Px, &'a mut [u8]>;

impl<Px: Pixel> ImagePacked<Px> {
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    pub fn as_data(&self) -> &[u8] {
        &self.data
    }
}

impl<Px: Pixel, Buf: AsRef<[u8]>> ImageRef for BaseImage<Px, Buf> {
    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Px {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_unchecked(&self.data.as_ref()[range]) }
    }
}

impl<Px: Pixel, Buf: AsMut<[u8]>> ImageMut for BaseImage<Px, Buf> {
    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn get_pixel_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Px {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_mut_unchecked(&mut self.data.as_mut()[range]) }
    }
}

impl<Px: Pixel, Buf: AsRef<[u8]>> Index<(usize, usize)> for BaseImage<Px, Buf> {
    type Output = Px;

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

impl<Px: Pixel, Buf: AsRef<[u8]> + AsMut<[u8]>> IndexMut<(usize, usize)> for BaseImage<Px, Buf> {
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
