use std::ops::Deref;

use mediary_image::{
    traits::PlanarImageRead, BaseImage, ImageContainer, ImageProperties, RgbImage,
};

use crate::{YuvChromaSubsampling, YuvPixel};

pub struct YuvPlanarBuffer<Buf> {
    pub y: Buf,
    pub u: Buf,
    pub v: Buf,

    pub subsampling: YuvChromaSubsampling,
}

pub type YuvPlanarImageInner = BaseImage<YuvPixel, YuvPlanarBuffer<Vec<u8>>>;

pub struct YuvPlanarImage {
    inner: YuvPlanarImageInner,
}

impl<Buf: AsRef<[u8]>> ImageContainer for YuvPlanarBuffer<Buf> {
    fn size(&self) -> usize {
        self.y.as_ref().len() + self.u.as_ref().len() + self.v.as_ref().len()
    }
}

impl YuvPlanarImage {
    #[inline(always)]
    pub fn y(&self) -> &[u8] {
        &self.inner.as_data().y
    }

    #[inline(always)]
    pub fn u(&self) -> &[u8] {
        &self.inner.as_data().u
    }

    #[inline(always)]
    pub fn v(&self) -> &[u8] {
        &self.inner.as_data().v
    }

    #[inline(always)]
    pub fn subsampling(&self) -> YuvChromaSubsampling {
        self.inner.as_data().subsampling
    }

    pub fn new_yuv420_from_rgb(rgb: &RgbImage) -> Self {
        let width = rgb.width();
        let height = rgb.height();
        let pixel_count = width * height;
        let chroma_pixel_count = pixel_count / 4;
        dbg!(width, height, pixel_count, chroma_pixel_count);

        let mut y = vec![0; pixel_count];
        let mut u = vec![0; chroma_pixel_count];
        let mut v = vec![0; chroma_pixel_count];

        for (((rgb, y), u), v) in rgb
            .as_data()
            .chunks_exact(width * 2)
            .zip(y.chunks_exact_mut(width * 2))
            .zip(u.chunks_exact_mut(width))
            .zip(v.chunks_exact_mut(width))
        {
            let (y0, y1) = y.split_at_mut(width);
            let (rgb0, rgb1) = rgb.split_at(width);
            // println!("> {rgb0:?}  |  {rgb1:?}");

            for (((((rgb0, rgb1), y0), y1), u), v) in rgb0
                .chunks_exact(6)
                .zip(rgb1.chunks_exact(6))
                .zip(y0.chunks_exact_mut(2))
                .zip(y1.chunks_exact_mut(2))
                .zip(u.iter_mut())
                .zip(v.iter_mut())
            {
                // println!("> {rgb0:?}  |  {rgb1:?}");
                let r00 = rgb0[0] as f32;
                let g00 = rgb0[1] as f32;
                let b00 = rgb0[2] as f32;

                let r01 = rgb0[3] as f32;
                let g01 = rgb0[4] as f32;
                let b01 = rgb0[5] as f32;

                let r10 = rgb1[0] as f32;
                let g10 = rgb1[1] as f32;
                let b10 = rgb1[2] as f32;

                let r11 = rgb1[3] as f32;
                let g11 = rgb1[4] as f32;
                let b11 = rgb1[5] as f32;

                y0[0] = (0.299 * r00 + 0.587 * g00 + 0.114 * b00) as u8;
                y0[1] = (0.299 * r01 + 0.587 * g01 + 0.114 * b01) as u8;
                y1[0] = (0.299 * r10 + 0.587 * g10 + 0.114 * b10) as u8;
                y1[1] = (0.299 * r11 + 0.587 * g11 + 0.114 * b11) as u8;

                let r = (r00 + r01 + r10 + r11) / 4.0;
                let g = (g00 + g01 + g10 + g11) / 4.0;
                let b = (b00 + b01 + b10 + b11) / 4.0;

                *u = (-0.168736 * r - 0.331264 * g + 0.5 * b + 128.0) as u8;
                *v = (0.5 * r - 0.418688 * g - 0.081312 * b + 128.0) as u8;
            }
        }

        let data = YuvPlanarBuffer {
            y,
            u,
            v,
            subsampling: YuvChromaSubsampling::Yuv420,
        };

        Self {
            inner: YuvPlanarImageInner::new_unchecked(data, width, height),
        }
    }
}

impl ImageProperties for YuvPlanarImage {
    type Pixel = YuvPixel;

    fn width(&self) -> usize {
        self.inner.width()
    }

    fn height(&self) -> usize {
        self.inner.height()
    }
}

impl PlanarImageRead for YuvPlanarImage {
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Self::Pixel {
        let idx = y * self.width() + x;

        match self.subsampling() {
            YuvChromaSubsampling::Yuv444 => YuvPixel {
                y: self.y()[idx],
                u: self.u()[idx],
                v: self.v()[idx],
            },
            YuvChromaSubsampling::Yuv420 => YuvPixel {
                y: self.y()[idx],
                u: self.u()[idx / 2],
                v: self.v()[idx / 2],
            },
        }
    }
}

impl Deref for YuvPlanarImage {
    type Target = YuvPlanarImageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
