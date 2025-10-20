use std::marker::PhantomData;

use crate::{ImageSource, ImageSourceMut, ImageUpscale, ImageView, ImageViewMut, Pixel};

pub struct ImageBuffer<Px: Pixel> {
    data: Vec<u8>,
    width: usize,
    height: usize,
    _phantom: PhantomData<Px>,
}

pub struct ImageRef<'a, Px: Pixel> {
    data: &'a [u8],
    width: usize,
    height: usize,
    _phantom: PhantomData<Px>,
}

pub struct ImageRefMut<'a, Px: Pixel> {
    data: &'a mut [u8],
    width: usize,
    height: usize,
    _phantom: PhantomData<Px>,
}

impl<Px: Pixel> ImageBuffer<Px> {
    pub fn new(data: Vec<u8>, width: usize, height: usize) -> Option<Self> {
        if data.len() == width * height * Px::CHANNEL_COUNT {
            Some(Self {
                data,
                width,
                height,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

impl<'a, Px: Pixel> ImageRef<'a, Px> {
    pub fn new(data: &'a [u8], width: usize, height: usize) -> Option<Self> {
        if data.len() == width * height * Px::CHANNEL_COUNT {
            Some(Self {
                data,
                width,
                height,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<'a, Px: Pixel> ImageRefMut<'a, Px> {
    pub fn new(data: &'a mut [u8], width: usize, height: usize) -> Option<Self> {
        if data.len() == width * height * Px::CHANNEL_COUNT {
            Some(Self {
                data,
                width,
                height,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<Px: Pixel> ImageSource for ImageBuffer<Px> {
    type Pixel = Px;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<&Self::Pixel> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
            let range = idx..idx + Px::CHANNEL_COUNT;

            Px::from_slice(&self.data[range])
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<ImageView<'_, Self>> {
        if x + width <= self.width && y + height <= self.height {
            Some(ImageView {
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

    fn upscale(&self, upscale_x: usize, upscale_y: usize) -> ImageUpscale<'_, Self> {
        ImageUpscale {
            image: self,
            upscale_x,
            upscale_y,
        }
    }
}

impl<Px: Pixel> ImageSourceMut for ImageBuffer<Px> {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
            let range = idx..idx + Px::CHANNEL_COUNT;

            Px::from_slice_mut(&mut self.data[range])
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
        if x + width <= self.width && y + height <= self.height {
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

impl<Px: Pixel> ImageSource for ImageRef<'_, Px> {
    type Pixel = Px;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<&Self::Pixel> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
            let range = idx..idx + Px::CHANNEL_COUNT;

            Px::from_slice(&self.data[range])
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<ImageView<'_, Self>> {
        if x + width <= self.width && y + height <= self.height {
            Some(ImageView {
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

    fn upscale(&self, upscale_x: usize, upscale_y: usize) -> ImageUpscale<'_, Self> {
        ImageUpscale {
            image: self,
            upscale_x,
            upscale_y,
        }
    }
}

impl<Px: Pixel> ImageSource for ImageRefMut<'_, Px> {
    type Pixel = Px;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get(&self, x: usize, y: usize) -> Option<&Self::Pixel> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
            let range = idx..idx + Px::CHANNEL_COUNT;

            Px::from_slice(&self.data[range])
        } else {
            None
        }
    }

    fn view(&self, x: usize, y: usize, width: usize, height: usize) -> Option<ImageView<'_, Self>> {
        if x + width <= self.width && y + height <= self.height {
            Some(ImageView {
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

    fn upscale(&self, upscale_x: usize, upscale_y: usize) -> ImageUpscale<'_, Self> {
        ImageUpscale {
            image: self,
            upscale_x,
            upscale_y,
        }
    }
}

impl<Px: Pixel> ImageSourceMut for ImageRefMut<'_, Px> {
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Pixel> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
            let range = idx..idx + Px::CHANNEL_COUNT;

            Px::from_slice_mut(&mut self.data[range])
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
        if x + width <= self.width && y + height <= self.height {
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
