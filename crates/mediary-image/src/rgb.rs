use crate::{
    packed::{ImagePacked, ImagePackedMut, ImagePackedRef},
    view::{ImageViewMut, ImageViewRef},
    BaseImage, Pixel,
};

#[derive(Debug, Clone, Copy)]
pub struct RgbPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub struct RgbPixelMut<'a> {
    pub r: &'a mut u8,
    pub g: &'a mut u8,
    pub b: &'a mut u8,
}

pub struct RgbPlanarBuffer<Buf> {
    pub r_plane: Buf,
    pub g_plane: Buf,
    pub b_plane: Buf,
}

pub type RgbImage = ImagePacked<RgbPixel>;
pub type RgbImageRef<'a> = ImagePackedRef<'a, RgbPixel>;
pub type RgbImageMut<'a> = ImagePackedMut<'a, RgbPixel>;
pub type RgbPlanar = BaseImage<RgbPixel, RgbPlanarBuffer<Vec<u8>>>;
pub type RgbPlanarRef<'a> = BaseImage<RgbPixel, RgbPlanarBuffer<&'a [u8]>>;
pub type RgbPlanarMut<'a> = BaseImage<RgbPixel, RgbPlanarBuffer<&'a mut [u8]>>;
pub type RgbImageView<'a> = ImageViewRef<'a, RgbImage>;
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

impl<Buf: AsRef<[u8]>> RgbPlanarBuffer<Buf> {
    pub fn new(r_plane: Buf, g_plane: Buf, b_plane: Buf) -> Option<Self> {
        if r_plane.as_ref().len() == g_plane.as_ref().len()
            && g_plane.as_ref().len() == b_plane.as_ref().len()
        {
            Some(Self {
                r_plane,
                g_plane,
                b_plane,
            })
        } else {
            None
        }
    }
}

impl<Buf: AsRef<[u8]>> BaseImage<RgbPixel, RgbPlanarBuffer<Buf>> {
    pub fn new(
        r_plane: Buf,
        g_plane: Buf,
        b_plane: Buf,
        width: usize,
        height: usize,
    ) -> Option<Self> {
        if r_plane.as_ref().len() + g_plane.as_ref().len() + b_plane.as_ref().len()
            == width * height * RgbPixel::CHANNEL_COUNT
        {
            Some(Self {
                data: RgbPlanarBuffer::new(r_plane, g_plane, b_plane)?,
                width,
                height,
                _phantom: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<RgbPixel> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            Some(RgbPixel {
                r: self.data.r_plane.as_ref()[idx],
                g: self.data.g_plane.as_ref()[idx],
                b: self.data.b_plane.as_ref()[idx],
            })
        } else {
            None
        }
    }

    pub fn r_plane(&self) -> &[u8] {
        self.data.r_plane.as_ref()
    }

    pub fn g_plane(&self) -> &[u8] {
        self.data.g_plane.as_ref()
    }

    pub fn b_plane(&self) -> &[u8] {
        self.data.b_plane.as_ref()
    }
}

impl<Buf: AsMut<[u8]>> BaseImage<RgbPixel, RgbPlanarBuffer<Buf>> {
    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> Option<RgbPixelMut<'_>> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            Some(RgbPixelMut {
                r: &mut self.data.r_plane.as_mut()[idx],
                g: &mut self.data.g_plane.as_mut()[idx],
                b: &mut self.data.b_plane.as_mut()[idx],
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ImageMut as _, ImageRef as _};

    use super::{RgbImage, RgbPlanar};

    #[test]
    fn rgb_packed() {
        let rgb = RgbImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 2, 2).unwrap();

        let px = rgb.get_pixel(0, 0).unwrap();
        assert_eq!(px.r, 1);
        assert_eq!(px.g, 2);
        assert_eq!(px.b, 3);

        let px = rgb.get_pixel(1, 0).unwrap();
        assert_eq!(px.r, 4);
        assert_eq!(px.g, 5);
        assert_eq!(px.b, 6);

        let px = rgb.get_pixel(0, 1).unwrap();
        assert_eq!(px.r, 7);
        assert_eq!(px.g, 8);
        assert_eq!(px.b, 9);

        let px = rgb.get_pixel(1, 1).unwrap();
        assert_eq!(px.r, 10);
        assert_eq!(px.g, 11);
        assert_eq!(px.b, 12);
    }

    #[test]
    fn rgb_packed_mut() {
        let mut image = RgbImage::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 2, 2).unwrap();

        let px = image.get_pixel_mut(0, 0).unwrap();
        assert_eq!(px.r, 1);
        assert_eq!(px.g, 2);
        assert_eq!(px.b, 3);
        px.g += 10;

        let px = image.get_pixel_mut(1, 0).unwrap();
        assert_eq!(px.r, 4);
        assert_eq!(px.g, 5);
        assert_eq!(px.b, 6);
        px.b += 20;

        let px = image.get_pixel_mut(0, 1).unwrap();
        assert_eq!(px.r, 7);
        assert_eq!(px.g, 8);
        assert_eq!(px.b, 9);
        px.r += 3;

        let px = image.get_pixel_mut(1, 1).unwrap();
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

    #[test]
    fn rgb_planar() {
        let rgb = RgbPlanar::new(
            vec![1, 4, 7, 10],
            vec![2, 5, 8, 11],
            vec![3, 6, 9, 12],
            2,
            2,
        )
        .unwrap();

        let px = rgb.get_pixel(0, 0).unwrap();
        assert_eq!(px.r, 1);
        assert_eq!(px.g, 2);
        assert_eq!(px.b, 3);

        let px = rgb.get_pixel(1, 0).unwrap();
        assert_eq!(px.r, 4);
        assert_eq!(px.g, 5);
        assert_eq!(px.b, 6);

        let px = rgb.get_pixel(0, 1).unwrap();
        assert_eq!(px.r, 7);
        assert_eq!(px.g, 8);
        assert_eq!(px.b, 9);

        let px = rgb.get_pixel(1, 1).unwrap();
        assert_eq!(px.r, 10);
        assert_eq!(px.g, 11);
        assert_eq!(px.b, 12);
    }

    #[test]
    fn rgb_planar_mut() {
        let mut rgb = RgbPlanar::new(
            vec![1, 4, 7, 10],
            vec![2, 5, 8, 11],
            vec![3, 6, 9, 12],
            2,
            2,
        )
        .unwrap();

        let px = rgb.get_pixel_mut(0, 0).unwrap();
        assert_eq!(*px.r, 1);
        assert_eq!(*px.g, 2);
        assert_eq!(*px.b, 3);
        *px.g += 10;
        assert_eq!(*px.g, 12);

        let px = rgb.get_pixel_mut(1, 0).unwrap();
        assert_eq!(*px.r, 4);
        assert_eq!(*px.g, 5);
        assert_eq!(*px.b, 6);
        *px.b += 20;
        assert_eq!(*px.b, 26);

        let px = rgb.get_pixel_mut(0, 1).unwrap();
        assert_eq!(*px.r, 7);
        assert_eq!(*px.g, 8);
        assert_eq!(*px.b, 9);
        *px.r += 3;
        assert_eq!(*px.r, 10);

        let px = rgb.get_pixel_mut(1, 1).unwrap();
        assert_eq!(*px.r, 10);
        assert_eq!(*px.g, 11);
        assert_eq!(*px.b, 12);
        *px.r += 1;
        *px.g = 2;
        *px.b *= 3;
        assert_eq!(*px.r, 11);
        assert_eq!(*px.g, 2);
        assert_eq!(*px.b, 36);

        assert_eq!(rgb.r_plane(), &[1, 4, 10, 11]);
        assert_eq!(rgb.g_plane(), &[12, 5, 8, 2]);
        assert_eq!(rgb.b_plane(), &[3, 26, 9, 36]);
    }
}
