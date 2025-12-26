pub mod planar;

use mediary_image::{Pixel, rgb::RgbPixel};
pub use planar::YuvPlanarImage;

#[derive(Debug, Clone, Copy)]
pub enum YuvChromaSubsampling {
    Yuv444,
    Yuv420,
}

#[derive(Debug, Clone)]
pub struct YuvPixel {
    pub y: u8,
    pub u: u8,
    pub v: u8,
}

impl Pixel for YuvPixel {
    const CHANNEL_COUNT: usize = 3;
}

impl YuvPixel {
    pub fn write_rgb(&self, rgb: &mut RgbPixel) {
        let y = f32::from(self.y);
        let cr = f32::from(self.u) - 128.0;
        let cb = f32::from(self.v) - 128.0;

        rgb.r = (y + 1.402 * cr).round().clamp(0.0, 255.0) as u8;
        rgb.g = (y - 0.344136 * cb - 0.714136 * cr)
            .round()
            .clamp(0.0, 255.0) as u8;
        rgb.b = (y + 1.772 * cb).round().clamp(0.0, 255.0) as u8;
    }
}
