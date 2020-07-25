use super::bsw::*;
use super::sbac::*;

pub(crate) fn evce_eco_tile_end_flag(bs: &mut EvceBsw, sbac: &mut EvceSbac, flag: u32) {
    sbac.encode_bin_trm(bs, flag);
}
