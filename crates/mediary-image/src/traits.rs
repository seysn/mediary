use crate::{ImageUpscaleRef, ImageViewMut, ImageViewRef, downscale::ImageDownscaleRef};

/// A generic pixel unit used both from planar and packed images
pub trait Pixel: Sized + Clone {
    /// Number of bytes for one Pixel
    const CHANNEL_COUNT: usize;

    /// Return a pixel reference and checking if the slice has correct size
    fn from_slice(data: &[u8]) -> Option<&Self> {
        if data.len() == Self::CHANNEL_COUNT {
            Some(unsafe { Self::from_slice_unchecked(data) })
        } else {
            None
        }
    }

    /// Return a pixel mutable reference and checking if the slice has correct size
    fn from_slice_mut(data: &mut [u8]) -> Option<&mut Self> {
        if data.len() == Self::CHANNEL_COUNT {
            Some(unsafe { Self::from_slice_mut_unchecked(data) })
        } else {
            None
        }
    }

    /// Return a pixel row reference from bytes
    fn from_row_slice(data: &[u8]) -> &[Self] {
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const Self, data.len()) }
    }

    /// Return bytes from a pixel row reference
    fn as_row_slice(data: &[Self]) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        }
    }

    /// Return a pixel reference
    ///
    /// # Safety
    ///
    /// This results in undefined behaviour if data size is incorrect
    unsafe fn from_slice_unchecked(data: &[u8]) -> &Self {
        unsafe { &*(data.as_ptr() as *const Self) }
    }

    /// Return a pixel mutable reference
    ///
    /// # Safety
    ///
    /// This results in undefined behaviour if data size incorrect
    unsafe fn from_slice_mut_unchecked(data: &mut [u8]) -> &mut Self {
        unsafe { &mut *(data.as_mut_ptr() as *mut Self) }
    }
}

/// Container used to store image data
pub trait ImageContainer {
    fn size(&self) -> usize;
}

/// General properties of an image
pub trait ImageProperties {
    type Pixel: Pixel;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

/// Trait to read on packed image
pub trait PackedImageRead: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if coordinates are incorrect
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

    fn downscale(&self, downscale_x: usize, downscale_y: usize) -> ImageDownscaleRef<'_, Self> {
        ImageDownscaleRef {
            image: self,
            downscale_x,
            downscale_y,
        }
    }
}

/// Trait to write on packed image
pub trait PackedImageWrite: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if coordinates are incorrect
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

/// Trait to read on planar image
pub trait PlanarImageRead: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if coordinates are incorrect
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Self::Pixel;

    fn get_pixel(&self, x: usize, y: usize) -> Option<Self::Pixel> {
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
}

/// Trait to read rows on image
pub trait RowsRead: ImageProperties {
    /// # Safety
    ///
    /// This results in undefined behaviour if y coordinate is incorrect
    unsafe fn get_row_unchecked(&self, y: usize) -> &[Self::Pixel];

    fn get_row(&self, y: usize) -> Option<&[Self::Pixel]> {
        if y < self.height() {
            Some(unsafe { self.get_row_unchecked(y) })
        } else {
            None
        }
    }
}
