use super::tbl::*;

pub(crate) fn CONV_LOG2(v: usize) -> u8 {
    evc_tbl_log2[v]
}
