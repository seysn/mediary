pub mod error;
pub mod rgb;
pub mod view;

pub use crate::{
    error::ImageResult,
    rgb::{RgbImage, RgbImageViewMut},
    view::{ImageView, ImageViewMut},
};

pub trait ImageSource: Sized {
    type Pixel;

    fn get(&self, x: usize, y: usize) -> Option<Self::Pixel>;
    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, Self>;
}

pub trait ImageSourceMut: ImageSource {
    fn set(&mut self, x: usize, y: usize, value: Self::Pixel) -> ImageResult<()>;
    fn view_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, Self>;
}
