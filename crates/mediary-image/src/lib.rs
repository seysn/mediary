pub mod error;
pub mod rgb;
pub mod view;

pub use crate::{
    rgb::{RgbImage, RgbImageViewMut},
    view::{ImageView, ImageViewMut},
};

pub trait ImageSource: Sized {
    type Pixel;

    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get(&self, x: usize, y: usize) -> Option<Self::Pixel>;
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
