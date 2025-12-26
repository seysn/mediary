use std::ops::Deref;

use crate::{
    ImageViewMut, ImageViewRef, Pixel,
    packed::{ImagePacked, ImagePackedMut, ImagePackedRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonoPixel(pub u8);

pub type MonoImage = ImagePacked<MonoPixel>;
pub type MonoImageRef<'a> = ImagePackedRef<'a, MonoPixel>;
pub type MonoImageRefMut<'a> = ImagePackedMut<'a, MonoPixel>;
pub type MonoImageView<'a> = ImageViewRef<'a, MonoImage>;
pub type MonoImageViewMut<'a> = ImageViewMut<'a, MonoImage>;

impl Pixel for MonoPixel {
    const CHANNEL_COUNT: usize = 1;
}

impl Deref for MonoPixel {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{MonoImage, PackedImageRead, mono::MonoPixel, traits::RowsRead};

    #[test]
    fn get_pixel() {
        let mono = MonoImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3).unwrap();
        assert_eq!(mono.get_pixel(0, 0).unwrap(), &MonoPixel(1));
        assert_eq!(mono.get_pixel(1, 0).unwrap(), &MonoPixel(2));
        assert_eq!(mono.get_pixel(2, 0).unwrap(), &MonoPixel(3));
        assert_eq!(mono.get_pixel(0, 1).unwrap(), &MonoPixel(4));
        assert_eq!(mono.get_pixel(1, 1).unwrap(), &MonoPixel(5));
        assert_eq!(mono.get_pixel(2, 1).unwrap(), &MonoPixel(6));
        assert_eq!(mono.get_pixel(0, 2).unwrap(), &MonoPixel(7));
        assert_eq!(mono.get_pixel(1, 2).unwrap(), &MonoPixel(8));
        assert_eq!(mono.get_pixel(2, 2).unwrap(), &MonoPixel(9));
        assert_eq!(mono.get_pixel(0, 3), None);
    }

    #[test]
    fn get_rows() {
        let mono = MonoImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3).unwrap();
        assert_eq!(
            mono.get_row(0).unwrap(),
            &[MonoPixel(1), MonoPixel(2), MonoPixel(3)]
        );
        assert_eq!(
            mono.get_row(1).unwrap(),
            &[MonoPixel(4), MonoPixel(5), MonoPixel(6)]
        );
        assert_eq!(
            mono.get_row(2).unwrap(),
            &[MonoPixel(7), MonoPixel(8), MonoPixel(9)]
        );
        assert_eq!(mono.get_row(3), None);
    }

    #[test]
    fn get_pixel_view() {
        let mono = MonoImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3).unwrap();
        let mono_view = mono.view(1, 1, 2, 2).unwrap();
        assert_eq!(mono_view.get_pixel(0, 0).unwrap(), &MonoPixel(5));
        assert_eq!(mono_view.get_pixel(1, 0).unwrap(), &MonoPixel(6));
        assert_eq!(mono_view.get_pixel(2, 0), None);
        assert_eq!(mono_view.get_pixel(0, 1).unwrap(), &MonoPixel(8));
        assert_eq!(mono_view.get_pixel(1, 1).unwrap(), &MonoPixel(9));
        assert_eq!(mono_view.get_pixel(2, 1), None);
    }

    #[test]
    fn get_rows_view() {
        let mono = MonoImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3).unwrap();
        let mono_view = mono.view(1, 1, 2, 2).unwrap();
        assert_eq!(mono_view.get_row(0).unwrap(), &[MonoPixel(5), MonoPixel(6)]);
        assert_eq!(mono_view.get_row(1).unwrap(), &[MonoPixel(8), MonoPixel(9)]);
        assert_eq!(mono_view.get_row(2), None);
    }
}
