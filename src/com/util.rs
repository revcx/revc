use super::tbl::*;
use super::*;

use std::cmp::*;

/* clipping within min and max */
pub(crate) fn EVC_CLIP3<T: Ord>(min_x: T, max_x: T, value: T) -> T {
    max(min_x, min(max_x, value))
}

pub(crate) fn CONV_LOG2(v: usize) -> u8 {
    evc_tbl_log2[v]
}

pub(crate) fn evc_poc_derivation(sps: &EvcSps, tid: u8, poc: &mut EvcPoc) {
    let sub_gop_length: i32 = (1 << sps.log2_sub_gop_length) as i32;
    let mut expected_tid = 0;

    if tid == 0 {
        poc.poc_val = poc.prev_poc_val as i32 + sub_gop_length;
        poc.prev_doc_offset = 0;
        poc.prev_poc_val = poc.poc_val as u32;
        return;
    }
    let mut doc_offset = (poc.prev_doc_offset + 1) % sub_gop_length;
    if doc_offset == 0 {
        poc.prev_poc_val += sub_gop_length as u32;
    } else {
        expected_tid = 1 + (doc_offset as f32).log2() as u8;
    }
    while tid != expected_tid {
        doc_offset = (doc_offset + 1) % sub_gop_length as i32;
        if doc_offset == 0 {
            expected_tid = 0;
        } else {
            expected_tid = 1 + (doc_offset as f32).log2() as u8;
        }
    }
    let poc_offset: i32 = sub_gop_length * ((2 * doc_offset + 1) / (1 << tid as i32) - 2);
    poc.poc_val = poc.prev_poc_val as i32 + poc_offset;
    poc.prev_doc_offset = doc_offset;
}
