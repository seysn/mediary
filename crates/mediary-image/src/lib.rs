pub mod error;
pub mod image;
pub mod mono;
pub mod rgb;
pub mod upscale;
pub mod view;

pub use crate::{
    error::ImageResult,
    mono::MonoImage,
    rgb::RgbImage,
    upscale::ImageUpscale,
    view::{ImageView, ImageViewMut},
};

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

pub trait ImageSource: Sized {
    type Pixel: Pixel;

    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get(&self, x: usize, y: usize) -> Option<&Self::Pixel>;

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<ImageView<'_, Self>> {
        if x + width <= self.width() && y + height <= self.height() {
            Some(ImageView {
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

    fn upscale(&self, upscale_x: usize, upscale_y: usize) -> ImageUpscale<'_, Self> {
        ImageUpscale {
            image: self,
            upscale_x,
            upscale_y,
        }
    }
}

pub trait ImageSourceMut: ImageSource {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel>;

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
