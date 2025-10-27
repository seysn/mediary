use crate::{
    image::{ImageBuffer, ImageRefMut},
    view::{ImageView, ImageViewMut},
    Pixel,
};

#[derive(Debug, Clone, Copy)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub type RgbImage = ImageBuffer<RgbPixel>;
pub type RgbImageRef<'a> = ImageRefMut<'a, RgbPixel>;
pub type RgbImageView<'a> = ImageView<'a, RgbImage>;
pub type RgbImageViewMut<'a> = ImageViewMut<'a, RgbImage>;

impl Pixel for RgbPixel {
    const CHANNEL_COUNT: usize = 3;

    unsafe fn from_slice_unchecked(data: &[u8]) -> &Self {
        unsafe { &*(data.as_ptr() as *const RgbPixel) }
    }

    unsafe fn from_slice_mut_unchecked(data: &mut [u8]) -> &mut Self {
        unsafe { &mut *(data.as_mut_ptr() as *mut RgbPixel) }
    }
}

#[cfg(test)]
mod tests {
    use super::RgbImage;
    use crate::ImageSourceMut;

    #[test]
    fn rgb_pixel_get_mut() {
        let mut image = RgbImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 2, 2).unwrap();

        let px = image.get_mut(0, 0).unwrap();
        assert_eq!(px.r, 1);
        assert_eq!(px.g, 2);
        assert_eq!(px.b, 3);
        px.g += 10;

        let px = image.get_mut(1, 0).unwrap();
        assert_eq!(px.r, 4);
        assert_eq!(px.g, 5);
        assert_eq!(px.b, 6);
        px.b += 20;

        let px = image.get_mut(0, 1).unwrap();
        assert_eq!(px.r, 7);
        assert_eq!(px.g, 8);
        assert_eq!(px.b, 9);
        px.r += 3;

        let px = image.get_mut(1, 1).unwrap();
        assert_eq!(px.r, 10);
        assert_eq!(px.g, 11);
        assert_eq!(px.b, 12);
        px.r += 1;
        px.g = 2;
        px.b *= 3;

        assert_eq!(
            image.into_data(),
            vec![1, 12, 3, 4, 5, 26, 10, 8, 9, 11, 2, 36]
        );
    }
}
