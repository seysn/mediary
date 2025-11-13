pub mod error;
pub mod mono;
pub mod packed;
pub mod rgb;
pub mod upscale;
pub mod view;

use std::marker::PhantomData;

pub use crate::{
    error::ImageResult,
    mono::MonoImage,
    rgb::RgbImage,
    upscale::ImageUpscaleRef,
    view::{ImageViewMut, ImageViewRef},
};

pub struct BaseImage<Px: Pixel, Buf> {
    data: Buf,
    width: usize,
    height: usize,
    _phantom: PhantomData<Px>,
}

pub trait Pixel {
    const CHANNEL_COUNT: usize;

    fn from_slice(data: &[u8]) -> Option<&Self> {
        if data.len() == Self::CHANNEL_COUNT {
            Some(unsafe { Self::from_slice_unchecked(data) })
        } else {
            None
        }
    }

    fn from_slice_mut(data: &mut [u8]) -> Option<&mut Self> {
        if data.len() == Self::CHANNEL_COUNT {
            Some(unsafe { Self::from_slice_mut_unchecked(data) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn from_slice_unchecked(data: &[u8]) -> &Self;

    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn from_slice_mut_unchecked(data: &mut [u8]) -> &mut Self;
}

pub trait ImageProperties {
    type Pixel: Pixel;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait ImageRef: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> &Self::Pixel;

    fn get_pixel(&self, x: usize, y: usize) -> Option<&Self::Pixel> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked(x, y) })
        } else {
            None
        }
    }

    fn view(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<ImageViewRef<'_, Self>> {
        if x + width <= self.width() && y + height <= self.height() {
            Some(ImageViewRef {
                image: self,
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }

    fn upscale(&self, upscale_x: usize, upscale_y: usize) -> ImageUpscaleRef<'_, Self> {
        ImageUpscaleRef {
            image: self,
            upscale_x,
            upscale_y,
        }
    }
}

pub trait ImageMut: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if data has incorrect size
    unsafe fn get_pixel_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Self::Pixel;

    fn get_pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_mut_unchecked(x, y) })
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
    ) -> Option<ImageViewMut<'_, Self>> {
        if x + width <= self.width() && y + height <= self.height() {
            Some(ImageViewMut {
                image: self,
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

impl<Px: Pixel, Buf> BaseImage<Px, Buf> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl<Px: Pixel, Buf: AsRef<[u8]>> BaseImage<Px, Buf> {
    pub fn new(data: Buf, width: usize, height: usize) -> Option<Self> {
        if data.as_ref().len() == width * height * Px::CHANNEL_COUNT {
            Some(Self {
                data,
                width,
                height,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<Px: Pixel, Buf> ImageProperties for BaseImage<Px, Buf> {
    type Pixel = Px;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}
