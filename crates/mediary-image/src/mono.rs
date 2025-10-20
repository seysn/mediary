use crate::{
    image::{ImageBuffer, ImageRef, ImageRefMut},
    ImageView, ImageViewMut, Pixel,
};

#[derive(Debug, Clone, Copy)]
pub struct MonoPixel(pub u8);

pub type MonoImage = ImageBuffer<MonoPixel>;
pub type MonoImageRef<'a> = ImageRef<'a, MonoPixel>;
pub type MonoImageRefMut<'a> = ImageRefMut<'a, MonoPixel>;
pub type MonoImageView<'a> = ImageView<'a, MonoImage>;
pub type MonoImageViewMut<'a> = ImageViewMut<'a, MonoImage>;

impl Pixel for MonoPixel {
    const CHANNEL_COUNT: usize = 1;

    fn from_slice(data: &[u8]) -> Option<&Self> {
        if data.len() == 1 {
            Some(unsafe { &*(data.as_ptr() as *const MonoPixel) })
        } else {
            None
        }
    }

    fn from_slice_mut(data: &mut [u8]) -> Option<&mut Self> {
        if data.len() == 1 {
            Some(unsafe { &mut *(data.as_mut_ptr() as *mut MonoPixel) })
        } else {
            None
        }
    }
}
