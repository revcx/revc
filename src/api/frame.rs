use crate::com::plane::*;
use crate::com::region::*;
use crate::com::*;

use super::util::*;
use super::*;

use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Frame<T: Pixel> {
    pub planes: [Plane<T>; N_C],
    pub chroma_sampling: ChromaSampling,
    pub crop_l: i16,
    pub crop_r: i16,
    pub crop_t: i16,
    pub crop_b: i16,
}

impl<T: Pixel> Frame<T> {
    pub fn new(width: usize, height: usize, chroma_sampling: ChromaSampling) -> Self {
        //TODO: support Monochrome
        Frame {
            planes: [
                Plane::new(width, height, 0, 0, PIC_PAD_SIZE_L, MIN_CU_LOG2),
                Plane::new(
                    (width + 1) >> 1,
                    (height + 1) >> 1,
                    1,
                    1,
                    PIC_PAD_SIZE_C,
                    MIN_CU_LOG2 - 1,
                ),
                Plane::new(
                    (width + 1) >> 1,
                    (height + 1) >> 1,
                    1,
                    1,
                    PIC_PAD_SIZE_C,
                    MIN_CU_LOG2 - 1,
                ),
            ],
            chroma_sampling,
            crop_l: 0,
            crop_r: 0,
            crop_t: 0,
            crop_b: 0,
        }
    }

    pub fn pad(&mut self) {
        for p in self.planes.iter_mut() {
            p.pad();
        }
    }

    /// Returns a `PixelIter` containing the data of this frame's planes in YUV format.
    /// Each point in the `PixelIter` is a triple consisting of a Y, U, and V component.
    /// The `PixelIter` is laid out as contiguous rows, e.g. to get a given 0-indexed row
    /// you could use `data.skip(width * row_idx).take(width)`.
    ///
    /// This data retains any padding, e.g. it uses the width and height specifed in
    /// the Y-plane's `cfg` struct, and not the display width and height specied in
    /// `FrameInvariants`.
    pub fn iter(&self) -> PixelIter<'_, T> {
        PixelIter::new(&self.planes)
    }
}

#[derive(Debug)]
pub struct PixelIter<'a, T: Pixel> {
    planes: &'a [Plane<T>; 3],
    y: usize,
    x: usize,
}

impl<'a, T: Pixel> PixelIter<'a, T> {
    pub fn new(planes: &'a [Plane<T>; 3]) -> Self {
        PixelIter { planes, y: 0, x: 0 }
    }

    fn width(&self) -> usize {
        self.planes[0].cfg.width
    }

    fn height(&self) -> usize {
        self.planes[0].cfg.height
    }
}

impl<'a, T: Pixel> Iterator for PixelIter<'a, T> {
    type Item = (T, T, T);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.y == self.height() - 1 && self.x == self.width() - 1 {
            return None;
        }
        let pixel = (
            self.planes[0].p(self.x, self.y),
            self.planes[1].p(
                self.x >> self.planes[1].cfg.xdec,
                self.y >> self.planes[1].cfg.ydec,
            ),
            self.planes[2].p(
                self.x >> self.planes[2].cfg.xdec,
                self.y >> self.planes[2].cfg.ydec,
            ),
        );
        if self.x == self.width() - 1 {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }
        Some(pixel)
    }
}
