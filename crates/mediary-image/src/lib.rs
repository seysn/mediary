pub mod downscale;
pub mod error;
pub mod mono;
pub mod packed;
pub mod rgb;
pub mod traits;
pub mod upscale;
pub mod view;

use std::marker::PhantomData;

pub use crate::{
    error::ImageResult,
    mono::MonoImage,
    rgb::RgbImage,
    traits::{ImageContainer, ImageProperties, PackedImageRead, PackedImageWrite, Pixel},
    upscale::ImageUpscaleRef,
    view::{ImageViewMut, ImageViewRef},
};

pub struct BaseImage<Px: Pixel, Buf> {
    data: Buf,
    width: usize,
    height: usize,
    _phantom: PhantomData<Px>,
}

impl<Px: Pixel, Buf> BaseImage<Px, Buf> {
    pub const fn as_data(&self) -> &Buf {
        &self.data
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }
}

impl<Px: Pixel, Buf: ImageContainer> BaseImage<Px, Buf> {
    pub fn new(data: Buf, width: usize, height: usize) -> Option<Self> {
        if data.size() == width * height * Px::CHANNEL_COUNT {
            Some(Self::new_unchecked(data, width, height))
        } else {
            None
        }
    }

    pub fn new_unchecked(data: Buf, width: usize, height: usize) -> Self {
        Self {
            data,
            width,
            height,
            _phantom: PhantomData,
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

impl<Buf: AsRef<[u8]>> ImageContainer for Buf {
    fn size(&self) -> usize {
        self.as_ref().len()
    }
}
