use std::ops::Deref;

use crate::{
    packed::{ImagePacked, ImagePackedMut, ImagePackedRef},
    ImageViewMut, ImageViewRef, Pixel,
};

#[derive(Debug, Clone, Copy)]
pub struct MonoPixel(pub u8);

pub type MonoImage = ImagePacked<MonoPixel>;
pub type MonoImageRef<'a> = ImagePackedRef<'a, MonoPixel>;
pub type MonoImageRefMut<'a> = ImagePackedMut<'a, MonoPixel>;
pub type MonoImageView<'a> = ImageViewRef<'a, MonoImage>;
pub type MonoImageViewMut<'a> = ImageViewMut<'a, MonoImage>;

impl Pixel for MonoPixel {
    const CHANNEL_COUNT: usize = 1;

    unsafe fn from_slice_unchecked(data: &[u8]) -> &Self {
        unsafe { &*(data.as_ptr() as *const MonoPixel) }
    }

    unsafe fn from_slice_mut_unchecked(data: &mut [u8]) -> &mut Self {
        unsafe { &mut *(data.as_mut_ptr() as *mut MonoPixel) }
    }
}

impl Deref for MonoPixel {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
