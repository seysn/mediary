pub mod error;
pub mod image;
pub mod rgb;
pub mod view;

pub use crate::{
    rgb::{RgbImage, RgbImageViewMut},
    view::{ImageView, ImageViewMut},
};

pub trait Pixel {
    const CHANNEL_COUNT: usize;

    fn from_slice(data: &[u8]) -> Option<&Self>;
    fn from_slice_mut(data: &mut [u8]) -> Option<&mut Self>;
}

pub trait ImageSource: Sized {
    type Pixel: Pixel;

    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get(&self, x: usize, y: usize) -> Option<&Self::Pixel>;
    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<ImageView<'_, Self>>;
}

pub trait ImageSourceMut: ImageSource {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel>;
    fn view_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<ImageViewMut<'_, Self>>;
}
