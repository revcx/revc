use super::tbl::*;

use std::cmp::*;

/* clipping within min and max */
pub(crate) fn EVC_CLIP3<T: Ord>(min_x: T, max_x: T, value: T) -> T {
    max(min_x, min(max_x, value))
}

pub(crate) fn CONV_LOG2(v: usize) -> u8 {
    evc_tbl_log2[v]
}
