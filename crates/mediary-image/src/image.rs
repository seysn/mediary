use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{ImageSource, ImageSourceMut, Pixel};

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

    pub fn as_data(&self) -> &[u8] {
        &self.data
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

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_unchecked(&self.data[range]) }
    }
}

impl<Px: Pixel> ImageSourceMut for ImageBuffer<Px> {
    unsafe fn get_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Self::Pixel {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_mut_unchecked(&mut self.data[range]) }
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

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_unchecked(&self.data[range]) }
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

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> &Self::Pixel {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_unchecked(&self.data[range]) }
    }
}

impl<Px: Pixel> ImageSourceMut for ImageRefMut<'_, Px> {
    unsafe fn get_mut_unchecked(&mut self, x: usize, y: usize) -> &mut Self::Pixel {
        let idx = (y * self.width + x) * Px::CHANNEL_COUNT;
        let range = idx..idx + Px::CHANNEL_COUNT;

        unsafe { Px::from_slice_mut_unchecked(&mut self.data[range]) }
    }
}

impl<Px: Pixel> Index<(usize, usize)> for ImageBuffer<Px> {
    type Output = Px;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<Px: Pixel> IndexMut<(usize, usize)> for ImageBuffer<Px> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let width = self.width;
        let height = self.height;

        match self.get_mut(x, y) {
            Some(p) => p,
            None => panic!("Position ({x}, {y}) is out of bounds ({width}, {height})",),
        }
    }
}

impl<Px: Pixel> Index<(usize, usize)> for ImageRef<'_, Px> {
    type Output = Px;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<Px: Pixel> Index<(usize, usize)> for ImageRefMut<'_, Px> {
    type Output = Px;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        match self.get(x, y) {
            Some(p) => p,
            None => panic!(
                "Position ({x}, {y}) is out of bounds ({}, {})",
                self.width, self.height
            ),
        }
    }
}

impl<Px: Pixel> IndexMut<(usize, usize)> for ImageRefMut<'_, Px> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let width = self.width;
        let height = self.height;

        match self.get_mut(x, y) {
            Some(p) => p,
            None => panic!("Position ({x}, {y}) is out of bounds ({width}, {height})",),
        }
    }
}
