use super::bsw::*;
use super::sbac::*;
use crate::api::*;
use crate::def::*;
use crate::util::*;

pub(crate) fn evce_eco_tile_end_flag(bs: &mut EvceBsw, sbac: &mut EvceSbac, flag: u32) {
    sbac.encode_bin_trm(bs, flag);
}

pub(crate) fn evce_eco_split_mode(
    bs: &mut EvceBsw,
    sbac: &mut EvceSbac,
    sbac_ctx: &mut EvcSbacCtx,
    cud: u16,
    cup: u16,
    cuw: u16,
    cuh: u16,
    lcu_s: u16,
    split_mode_buf: &LcuSplitMode,
) {
    let mut split_mode = SplitMode::NO_SPLIT;
    let mut ctx = 0;
    let order_idx = if cuw >= cuh { 0 } else { 1 };

    let mut split_allow = vec![false; MAX_SPLIT_NUM];

    if cuw < 8 && cuh < 8 {
        return;
    }

    //evc_assert(evce_check_luma(c, core));
    split_mode = evc_get_split_mode(cud, cup, cuw, cuh, lcu_s, split_mode_buf);

    sbac.encode_bin(
        bs,
        &mut sbac_ctx.split_cu_flag[0],
        if split_mode != SplitMode::NO_SPLIT {
            1
        } else {
            0
        },
    ); /* split_cu_flag */

    /*EVC_TRACE_COUNTER;
    EVC_TRACE_STR("x pos ");
    EVC_TRACE_INT(core->x_pel + ((cup % (c->max_cuwh >> MIN_CU_LOG2)) << MIN_CU_LOG2));
    EVC_TRACE_STR("y pos ");
    EVC_TRACE_INT(core->y_pel + ((cup / (c->max_cuwh >> MIN_CU_LOG2)) << MIN_CU_LOG2));
    EVC_TRACE_STR("width ");
    EVC_TRACE_INT(cuw);
    EVC_TRACE_STR("height ");
    EVC_TRACE_INT(cuh);
    EVC_TRACE_STR("depth ");
    EVC_TRACE_INT(cud);
    EVC_TRACE_STR("split mode ");
    EVC_TRACE_INT(split_mode);
    EVC_TRACE_STR("\n");*/
}

pub(crate) fn evce_eco_intra_dir_b(
    bs: &mut EvceBsw,
    sbac: &mut EvceSbac,
    sbac_ctx: &mut EvcSbacCtx,
    ipm: u8,
    mpm: &[u8],
) {
    sbac.write_unary_sym(bs, &mut sbac_ctx.intra_dir, mpm[ipm as usize] as u32, 2);
    /* EVC_TRACE_COUNTER;
    #if TRACE_ADDITIONAL_FLAGS
        EVC_TRACE_STR("mpm list: ");
        for (int i = 0; i < IPD_CNT_B; i++)
        {
            EVC_TRACE_INT(mpm[i]);
        }
    #endif
        EVC_TRACE_STR("ipm Y ");
        EVC_TRACE_INT(ipm);
        EVC_TRACE_STR("\n");*/
}
