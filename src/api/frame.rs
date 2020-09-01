use super::*;
use crate::def::*;
use crate::plane::*;
use crate::region::*;

use std::fmt;

use num_traits::*;
use std::fmt::{Debug, Display};
use std::mem::size_of;
use std::mem::MaybeUninit;
use std::{cmp, io};

#[derive(Clone)]
#[repr(align(32))]
pub struct Align32;

/// A 16 byte aligned array.
#[derive(Clone, Default)]
pub struct AlignedArray<ARRAY> {
    _alignment: [Align32; 0],
    pub array: ARRAY,
}

#[allow(non_snake_case)]
pub fn AlignedArray<ARRAY>(array: ARRAY) -> AlignedArray<ARRAY> {
    AlignedArray {
        _alignment: [],
        array,
    }
}

#[allow(non_snake_case)]
pub fn UninitializedAlignedArray<ARRAY>() -> AlignedArray<ARRAY> {
    AlignedArray(unsafe { MaybeUninit::uninit().assume_init() })
}

#[test]
fn sanity() {
    let a: AlignedArray<_> = AlignedArray([0u8; 3]);
    assert!(is_aligned(a.array.as_ptr(), 4));
}

pub trait Fixed {
    fn floor_log2(&self, n: usize) -> usize;
    fn ceil_log2(&self, n: usize) -> usize;
    fn align_power_of_two(&self, n: usize) -> usize;
    fn align_power_of_two_and_shift(&self, n: usize) -> usize;
}

impl Fixed for usize {
    #[inline]
    fn floor_log2(&self, n: usize) -> usize {
        self & !((1 << n) - 1)
    }
    #[inline]
    fn ceil_log2(&self, n: usize) -> usize {
        (self + (1 << n) - 1).floor_log2(n)
    }
    #[inline]
    fn align_power_of_two(&self, n: usize) -> usize {
        self.ceil_log2(n)
    }
    #[inline]
    fn align_power_of_two_and_shift(&self, n: usize) -> usize {
        (self + (1 << n) - 1) >> n
    }
}

/// Check alignment.
pub fn is_aligned<T>(ptr: *const T, n: usize) -> bool {
    ((ptr as usize) & ((1 << n) - 1)) == 0
}

pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

pub trait CastFromPrimitive<T>: Copy + 'static {
    fn cast_from(v: T) -> Self;
}

macro_rules! impl_cast_from_primitive {
  ( $T:ty => $U:ty ) => {
    impl CastFromPrimitive<$U> for $T {
      #[inline(always)]
      fn cast_from(v: $U) -> Self { v as Self }
    }
  };
  ( $T:ty => { $( $U:ty ),* } ) => {
    $( impl_cast_from_primitive!($T => $U); )*
  };
}

// casts to { u8, u16 } are implemented separately using Pixel, so that the
// compiler understands that CastFromPrimitive<T: Pixel> is always implemented
impl_cast_from_primitive!(u8 => { u32, u64, usize });
impl_cast_from_primitive!(u8 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(u16 => { u32, u64, usize });
impl_cast_from_primitive!(u16 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(i16 => { u32, u64, usize });
impl_cast_from_primitive!(i16 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(i32 => { u32, u64, usize });
impl_cast_from_primitive!(i32 => { i8, i16, i32, i64, isize });

pub trait Pixel:
    PrimInt
    + Into<u32>
    + Into<i32>
    + AsPrimitive<u8>
    + AsPrimitive<i16>
    + AsPrimitive<u16>
    + AsPrimitive<i32>
    + AsPrimitive<u32>
    + AsPrimitive<usize>
    + CastFromPrimitive<u8>
    + CastFromPrimitive<i16>
    + CastFromPrimitive<u16>
    + CastFromPrimitive<i32>
    + CastFromPrimitive<u32>
    + CastFromPrimitive<usize>
    + Debug
    + Display
    + Send
    + Sync
    + 'static
{
}

impl Pixel for u8 {}
impl Pixel for u16 {}

macro_rules! impl_cast_from_pixel_to_primitive {
    ( $T:ty ) => {
        impl<T: Pixel> CastFromPrimitive<T> for $T {
            #[inline(always)]
            fn cast_from(v: T) -> Self {
                v.as_()
            }
        }
    };
}

impl_cast_from_pixel_to_primitive!(u8);
impl_cast_from_pixel_to_primitive!(i16);
impl_cast_from_pixel_to_primitive!(u16);
impl_cast_from_pixel_to_primitive!(i32);
impl_cast_from_pixel_to_primitive!(u32);

pub trait ILog: PrimInt {
    fn ilog(self) -> Self {
        Self::from(size_of::<Self>() * 8 - self.leading_zeros() as usize).unwrap()
    }
}

impl<T> ILog for T where T: PrimInt {}

#[inline(always)]
pub fn msb(x: i32) -> i32 {
    debug_assert!(x > 0);
    31 ^ (x.leading_zeros() as i32)
}

#[inline(always)]
pub fn round_shift(value: i32, bit: usize) -> i32 {
    (value + (1 << bit >> 1) as i32) >> bit as i32
}

#[inline(always)]
pub fn clip<T: PartialOrd>(v: T, min: T, max: T) -> T {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

#[inline(always)]
pub fn check_error(condition: bool, msg: &str) -> io::Result<()> {
    if condition {
        Err(io::Error::new(io::ErrorKind::InvalidInput, msg))
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn tile_log2(sz: i32, tgt: i32) -> i32 {
    let mut k = 0;
    while (sz << k) < tgt {
        k += 1;
    }
    k
}

#[derive(Debug, Clone, Default)]
pub struct Frame<T: Pixel> {
    pub planes: [Plane<T>; N_C],
    pub chroma_sampling: ChromaSampling,
    pub ts: u64,
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
                Plane::new(width, height, 0, 0, PIC_PAD_SIZE_L, PIC_PAD_SIZE_L),
                Plane::new(
                    (width + 1) >> 1,
                    (height + 1) >> 1,
                    1,
                    1,
                    PIC_PAD_SIZE_C,
                    PIC_PAD_SIZE_C,
                ),
                Plane::new(
                    (width + 1) >> 1,
                    (height + 1) >> 1,
                    1,
                    1,
                    PIC_PAD_SIZE_C,
                    PIC_PAD_SIZE_C,
                ),
            ],
            chroma_sampling,
            ts: 0,
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
