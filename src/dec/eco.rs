use super::bsr::*;
use super::sbac::EvcdSbac;
use super::{EvcdCore, EvcdCtx};
use crate::api::{EvcError, NaluType, SliceType};
use crate::def::*;
use crate::ipred::*;
use crate::itdq::*;
use crate::mc::*;
use crate::picman::*;
use crate::recon::*;
use crate::tbl::*;
use crate::tracer::*;
use crate::util::*;

use std::cell::RefCell;
use std::rc::Rc;

use log::*;

pub(crate) fn evcd_eco_nalu(bs: &mut EvcdBsr, nalu: &mut EvcNalu) -> Result<(), EvcError> {
    //nalu->nal_unit_size = bs.read(32);
    nalu.forbidden_zero_bit = bs.read(1, Some("nalu->forbidden_zero_bit"))? as u8;

    if nalu.forbidden_zero_bit != 0 {
        error!("malformed bitstream: forbidden_zero_bit != 0\n");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nal_unit_type = (bs.read(6, Some("nalu->nal_unit_type_plus1"))? as u8 - 1).into();
    nalu.nuh_temporal_id = bs.read(3, Some("nalu->nuh_temporal_id"))? as u8;
    nalu.nuh_reserved_zero_5bits = bs.read(5, Some("nalu->nuh_reserved_zero_5bits"))? as u8;

    if nalu.nuh_reserved_zero_5bits != 0 {
        error!("malformed bitstream: nuh_reserved_zero_5bits != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nuh_extension_flag = bs.read(1, Some("nalu->nuh_extension_flag"))? != 0;

    if nalu.nuh_extension_flag {
        error!("malformed bitstream: nuh_extension_flag != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    Ok(())
}

pub(crate) fn evcd_eco_sps(bs: &mut EvcdBsr, sps: &mut EvcSps) -> Result<(), EvcError> {
    EVC_TRACE(&mut bs.tracer, "***********************************\n");
    EVC_TRACE(&mut bs.tracer, "************ SPS Start ************\n");

    sps.sps_seq_parameter_set_id = bs.read_ue(Some("sps->sps_seq_parameter_set_id"))? as u8;
    sps.profile_idc = bs.read(8, Some("sps->profile_idc"))? as u8;
    sps.level_idc = bs.read(8, Some("sps->level_idc"))? as u8;
    sps.toolset_idc_h = bs.read(32, Some("sps->toolset_idc_h"))?;
    sps.toolset_idc_l = bs.read(32, Some("sps->toolset_idc_l"))?;
    sps.chroma_format_idc = bs.read_ue(Some("sps->chroma_format_idc"))? as u8;
    sps.pic_width_in_luma_samples = bs.read_ue(Some("sps->pic_width_in_luma_samples"))? as u16;
    sps.pic_height_in_luma_samples = bs.read_ue(Some("sps->pic_height_in_luma_samples"))? as u16;
    sps.bit_depth_luma_minus8 = bs.read_ue(Some("sps->bit_depth_luma_minus8"))? as u8;
    sps.bit_depth_chroma_minus8 = bs.read_ue(Some("sps->bit_depth_chroma_minus8"))? as u8;
    sps.sps_btt_flag = bs.read1(Some("sps->sps_btt_flag"))? != 0;
    sps.sps_suco_flag = bs.read1(Some("sps->sps_suco_flag"))? != 0;
    sps.tool_admvp = bs.read1(Some("sps->tool_admvp"))? != 0;
    sps.tool_eipd = bs.read1(Some("sps->tool_eipd"))? != 0;
    sps.tool_cm_init = bs.read1(Some("sps->tool_cm_init"))? != 0;
    sps.tool_iqt = bs.read1(Some("sps->tool_iqt"))? != 0;
    sps.tool_addb = bs.read1(Some("sps->tool_addb"))? != 0;
    sps.tool_alf = bs.read1(Some("sps->tool_alf"))? != 0;
    sps.tool_htdf = bs.read1(Some("sps->tool_htdf"))? != 0;
    sps.tool_rpl = bs.read1(Some("sps->tool_rpl"))? != 0;
    sps.tool_pocs = bs.read1(Some("sps->tool_pocs"))? != 0;
    sps.dquant_flag = bs.read1(Some("sps->dquant_flag"))? != 0;
    sps.tool_dra = bs.read1(Some("sps->tool_dra"))? != 0;
    if !sps.tool_rpl || !sps.tool_pocs {
        sps.log2_sub_gop_length = bs.read_ue(Some("sps->log2_sub_gop_length"))? as u8;
        if sps.log2_sub_gop_length == 0 {
            sps.log2_ref_pic_gap_length = bs.read_ue(Some("sps->log2_ref_pic_gap_length"))? as u8;
        }
    }
    if !sps.tool_rpl {
        sps.max_num_ref_pics = bs.read_ue(Some("sps->max_num_ref_pics"))? as u8;
    }

    sps.picture_cropping_flag = bs.read1(Some("sps->picture_cropping_flag"))? != 0;
    if sps.picture_cropping_flag {
        sps.picture_crop_left_offset = bs.read_ue(Some("sps->picture_crop_left_offset"))? as u16;
        sps.picture_crop_right_offset = bs.read_ue(Some("sps->picture_crop_right_offset"))? as u16;
        sps.picture_crop_top_offset = bs.read_ue(Some("sps->picture_crop_top_offset"))? as u16;
        sps.picture_crop_bottom_offset =
            bs.read_ue(Some("sps->picture_crop_bottom_offset"))? as u16;
    }

    if sps.chroma_format_idc != 0 {
        sps.chroma_qp_table_struct.chroma_qp_table_present_flag = bs.read1(Some(
            "sps->chroma_qp_table_struct.chroma_qp_table_present_flag",
        ))? != 0;
        if sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            sps.chroma_qp_table_struct.same_qp_table_for_chroma =
                bs.read1(Some("sps->chroma_qp_table_struct.same_qp_table_for_chroma"))? != 0;
            sps.chroma_qp_table_struct.global_offset_flag =
                bs.read1(Some("sps->chroma_qp_table_struct.global_offset_flag"))? != 0;
            for i in 0..if sps.chroma_qp_table_struct.same_qp_table_for_chroma {
                1
            } else {
                2
            } {
                sps.chroma_qp_table_struct.num_points_in_qp_table_minus1[i] = bs.read_ue(Some(
                    "sps->chroma_qp_table_struct.num_points_in_qp_table_minus1[i]",
                ))?
                    as usize;
                for j in 0..=sps.chroma_qp_table_struct.num_points_in_qp_table_minus1[i] {
                    sps.chroma_qp_table_struct.delta_qp_in_val_minus1[i][j] = bs.read(
                        6,
                        Some("sps->chroma_qp_table_struct.delta_qp_in_val_minus1[i][j]"),
                    )?
                        as i8;
                    sps.chroma_qp_table_struct.delta_qp_out_val[i][j] = bs
                        .read_se(Some("sps->chroma_qp_table_struct.delta_qp_out_val[i][j]"))?
                        as i8;
                }
            }
        }
    }

    sps.vui_parameters_present_flag = bs.read1(Some("sps->vui_parameters_present_flag"))? != 0;
    if sps.vui_parameters_present_flag {
        //sps.vui_parameters = evcd_eco_vui(bs)?;
    }

    while !bs.is_byte_aligned() {
        bs.read1(Some("t0"))?;
    }

    EVC_TRACE(&mut bs.tracer, "************ SPS End   ************\n");
    EVC_TRACE(&mut bs.tracer, "***********************************\n");

    Ok(())
}

pub(crate) fn evcd_eco_pps(
    bs: &mut EvcdBsr,
    sps: &EvcSps,
    pps: &mut EvcPps,
) -> Result<(), EvcError> {
    EVC_TRACE(&mut bs.tracer, "***********************************\n");
    EVC_TRACE(&mut bs.tracer, "************ PPS Start ************\n");

    pps.pps_pic_parameter_set_id = bs.read_ue(Some("pps->pps_pic_parameter_set_id"))? as u8;
    pps.pps_seq_parameter_set_id = bs.read_ue(Some("pps->pps_seq_parameter_set_id"))? as u8;
    pps.num_ref_idx_default_active_minus1[0] =
        bs.read_ue(Some("pps->num_ref_idx_default_active_minus1[0]"))? as u8;
    pps.num_ref_idx_default_active_minus1[1] =
        bs.read_ue(Some("pps->num_ref_idx_default_active_minus1[1]"))? as u8;
    pps.additional_lt_poc_lsb_len = bs.read_ue(Some("pps->additional_lt_poc_lsb_len"))? as u8;
    pps.rpl1_idx_present_flag = bs.read1(Some("pps->rpl1_idx_present_flag"))? != 0;
    pps.single_tile_in_pic_flag = bs.read1(Some("pps->single_tile_in_pic_flag"))? != 0;
    assert_eq!(pps.single_tile_in_pic_flag, true);

    pps.tile_id_len_minus1 = bs.read_ue(Some("pps->tile_id_len_minus1"))? as u8;
    pps.explicit_tile_id_flag = bs.read1(Some("pps->explicit_tile_id_flag"))? != 0;
    assert_eq!(pps.explicit_tile_id_flag, false);

    pps.pic_dra_enabled_flag = bs.read1(Some("pps->pic_dra_enabled_flag"))? != 0;
    assert_eq!(pps.pic_dra_enabled_flag, false);

    pps.arbitrary_slice_present_flag = bs.read1(Some("pps->arbitrary_slice_present_flag"))? != 0;
    pps.constrained_intra_pred_flag = bs.read1(Some("pps->constrained_intra_pred_flag"))? != 0;

    pps.cu_qp_delta_enabled_flag = bs.read1(Some("pps->cu_qp_delta_enabled_flag"))? != 0;
    if pps.cu_qp_delta_enabled_flag {
        pps.cu_qp_delta_area = bs.read_ue(Some("pps->cu_qp_delta_area"))? as u8;
        pps.cu_qp_delta_area += 6;
    }

    while !bs.is_byte_aligned() {
        bs.read1(Some("t0"))?;
    }

    EVC_TRACE(&mut bs.tracer, "************ PPS End   ************\n");
    EVC_TRACE(&mut bs.tracer, "***********************************\n");

    Ok(())
}

pub(crate) fn evcd_eco_sh(
    bs: &mut EvcdBsr,
    sps: &EvcSps,
    pps: &EvcPps,
    sh: &mut EvcSh,
    nalu_type: NaluType,
) -> Result<(), EvcError> {
    EVC_TRACE(&mut bs.tracer, "***********************************\n");
    EVC_TRACE(&mut bs.tracer, "************ SH  Start ************\n");

    sh.slice_pic_parameter_set_id = bs.read_ue(Some("sh->slice_pic_parameter_set_id"))? as u8;
    sh.slice_type = (bs.read_ue(Some("sh->slice_type"))? as u8).into();

    if nalu_type == NaluType::EVC_IDR_NUT {
        sh.no_output_of_prior_pics_flag = bs.read1(Some("sh->no_output_of_prior_pics_flag"))? != 0;
    } else {
        if sh.slice_type == SliceType::EVC_ST_P || sh.slice_type == SliceType::EVC_ST_B {
            sh.num_ref_idx_active_override_flag =
                bs.read1(Some("sh->num_ref_idx_active_override_flag"))? != 0;
            if sh.num_ref_idx_active_override_flag {
                sh.rpl_l0.ref_pic_active_num =
                    bs.read_ue(Some("num_ref_idx_active_minus1"))? as u8 + 1;
                if sh.slice_type == SliceType::EVC_ST_B {
                    sh.rpl_l1.ref_pic_active_num =
                        bs.read_ue(Some("num_ref_idx_active_minus1"))? as u8 + 1;
                }
            } else {
                sh.rpl_l0.ref_pic_active_num = pps.num_ref_idx_default_active_minus1[REFP_0] + 1;
                sh.rpl_l1.ref_pic_active_num = pps.num_ref_idx_default_active_minus1[REFP_1] + 1;
            }
        }
    }

    sh.deblocking_filter_on = bs.read1(Some("sh->deblocking_filter_on"))? != 0;
    sh.qp = bs.read(6, Some("sh->qp"))? as u8;
    if sh.qp < 0 || sh.qp > 51 {
        error!("malformed bitstream: slice_qp should be in the range of 0 to 51\n");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    sh.qp_u_offset = bs.read_se(Some("sh->qp_u_offset"))? as i8;
    sh.qp_v_offset = bs.read_se(Some("sh->qp_v_offset"))? as i8;

    sh.qp_u = EVC_CLIP3(-6 * (BIT_DEPTH - 8) as i8, 57, sh.qp as i8 + sh.qp_u_offset) as u8;
    sh.qp_v = EVC_CLIP3(-6 * (BIT_DEPTH - 8) as i8, 57, sh.qp as i8 + sh.qp_v_offset) as u8;

    /* byte align */
    while !bs.is_byte_aligned() {
        let t0 = bs.read1(Some("t0"))?;
        if t0 != 0 {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }
    }

    EVC_TRACE(&mut bs.tracer, "************ SH  End   ************\n");
    EVC_TRACE(&mut bs.tracer, "***********************************\n");

    Ok(())
}

pub(crate) fn evcd_eco_tile_end_flag(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
) -> Result<u32, EvcError> {
    sbac.decode_bin_trm(bs)
}

pub(crate) fn evcd_eco_split_mode(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    cuw: u16,
    cuh: u16,
) -> Result<SplitMode, EvcError> {
    if cuw < 8 && cuh < 8 {
        Ok(SplitMode::NO_SPLIT)
    } else {
        /* split_cu_flag */
        let bin = sbac.decode_bin(bs, &mut sbac_ctx.split_cu_flag[0])?;

        if bin != 0 {
            Ok(SplitMode::SPLIT_QUAD)
        } else {
            Ok(SplitMode::NO_SPLIT)
        }
    }
}

pub(crate) fn evcd_eco_cu_skip_flag(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    ctx_flags: &[u8],
) -> Result<u32, EvcError> {
    let ctx_flag = ctx_flags[CNID_SKIP_FLAG] as usize;
    let cu_skip_flag = sbac.decode_bin(bs, &mut sbac_ctx.skip_flag[ctx_flag])?; /* cu_skip_flag */

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "skip flag ");
    EVC_TRACE(&mut bs.tracer, cu_skip_flag);
    EVC_TRACE(&mut bs.tracer, " ctx ");
    EVC_TRACE(&mut bs.tracer, ctx_flag);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(cu_skip_flag)
}

pub(crate) fn evcd_eco_pred_mode(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    ctx_flags: &[u8],
    slice_type: SliceType,
) -> Result<PredMode, EvcError> {
    if slice_type != SliceType::EVC_ST_I {
        let ctx_flag = ctx_flags[CNID_PRED_MODE] as usize;
        let pred_mode = if sbac.decode_bin(bs, &mut sbac_ctx.pred_mode[ctx_flag])? != 0 {
            PredMode::MODE_INTRA
        } else {
            PredMode::MODE_INTER
        };

        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "pred mode ");
        EVC_TRACE(&mut bs.tracer, pred_mode as u8);
        EVC_TRACE(&mut bs.tracer, " \n");

        Ok(pred_mode)
    } else {
        Ok(PredMode::MODE_INTRA)
    }
}

pub(crate) fn evcd_eco_mvp_idx(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
) -> Result<u8, EvcError> {
    let idx = sbac.read_truncate_unary_sym(bs, &mut sbac_ctx.mvp_idx, 3, 4)? as u8;

    //#if ENC_DEC_TRACE
    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "mvp idx ");
    EVC_TRACE(&mut bs.tracer, idx);
    EVC_TRACE(&mut bs.tracer, " \n");
    //#endif

    Ok(idx)
}

pub(crate) fn evcd_eco_abs_mvd(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    model: &mut SBAC_CTX_MODEL,
) -> Result<u32, EvcError> {
    let mut val = 0;

    let mut code = sbac.decode_bin(bs, model)?; /* use one model */

    if code == 0 {
        let mut len = 0;
        while (code & 1) == 0 {
            if len == 0 {
                code = sbac.decode_bin(bs, model)?;
            } else {
                code = sbac.decode_bin_ep(bs)?;
            }
            len += 1;
        }
        val = (1 << len) - 1;

        while len != 0 {
            if len == 0 {
                code = sbac.decode_bin(bs, model)?;
            } else {
                code = sbac.decode_bin_ep(bs)?;
            }
            len -= 1;
            val += code << len;
        }
    }

    Ok(val)
}

pub(crate) fn evcd_eco_get_mvd(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    mvd: &mut [i16],
) -> Result<(), EvcError> {
    /* MV_X */
    let mut t16 = evcd_eco_abs_mvd(bs, sbac, &mut sbac_ctx.mvd[0])? as i16;

    if t16 == 0 {
        mvd[MV_X] = 0;
    } else {
        /* sign */
        let sign = sbac.decode_bin_ep(bs)?;

        if sign != 0 {
            mvd[MV_X] = -t16;
        } else {
            mvd[MV_X] = t16;
        }
    }

    /* MV_Y */
    t16 = evcd_eco_abs_mvd(bs, sbac, &mut sbac_ctx.mvd[0])? as i16;

    if t16 == 0 {
        mvd[MV_Y] = 0;
    } else {
        /* sign */
        let sign = sbac.decode_bin_ep(bs)?;

        if sign != 0 {
            mvd[MV_Y] = -t16;
        } else {
            mvd[MV_Y] = t16;
        }
    }

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "mvd x ");
    EVC_TRACE(&mut bs.tracer, mvd[MV_X]);
    EVC_TRACE(&mut bs.tracer, " mvd y ");
    EVC_TRACE(&mut bs.tracer, mvd[MV_Y]);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(())
}

pub(crate) fn evcd_eco_direct_mode_flag(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
) -> Result<InterPredDir, EvcError> {
    let inter_dir = if sbac.decode_bin(bs, &mut sbac_ctx.direct_mode_flag[0])? != 0 {
        InterPredDir::PRED_DIR
    } else {
        InterPredDir::PRED_L0
    };

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "direct_mode_flag ");
    EVC_TRACE(&mut bs.tracer, inter_dir as u8);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(inter_dir)
}

pub(crate) fn evcd_eco_inter_pred_idc(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    slice_type: SliceType,
) -> Result<InterPredDir, EvcError> {
    let mut tmp = true;
    if check_bi_applicability(slice_type) {
        tmp = sbac.decode_bin(bs, &mut sbac_ctx.inter_dir[0])? != 0;
    }

    let inter_dir = if !tmp {
        InterPredDir::PRED_BI
    } else {
        tmp = sbac.decode_bin(bs, &mut sbac_ctx.inter_dir[1])? != 0;
        if tmp {
            InterPredDir::PRED_L1
        } else {
            InterPredDir::PRED_L0
        }
    };

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "inter dir ");
    EVC_TRACE(&mut bs.tracer, inter_dir as u8);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(inter_dir)
}

pub(crate) fn evcd_eco_refi(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    num_refp: u8,
) -> Result<u8, EvcError> {
    //#if 1 //yuliu debug
    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "num_refp: ");
    EVC_TRACE(&mut bs.tracer, num_refp);
    EVC_TRACE(&mut bs.tracer, " \n");
    //#endif

    let mut ref_num = 0;
    if num_refp > 1 {
        if sbac.decode_bin(bs, &mut sbac_ctx.refi[0])? != 0 {
            ref_num += 1;
            if num_refp > 2 && sbac.decode_bin(bs, &mut sbac_ctx.refi[1])? != 0 {
                ref_num += 1;
                while ref_num < num_refp - 1 {
                    if sbac.decode_bin_ep(bs)? == 0 {
                        break;
                    }
                    ref_num += 1;
                }
                return Ok(ref_num);
            }
        }
    }

    Ok(ref_num)
}

pub(crate) fn evcd_eco_intra_dir_b(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    mpm: &[u8],
) -> Result<u8, EvcError> {
    let mut ipm = 0;
    let t0 = sbac.read_unary_sym(bs, &mut sbac_ctx.intra_dir, 2)?;

    EVC_TRACE_COUNTER(&mut bs.tracer);
    //#if TRACE_ADDITIONAL_FLAGS
    //    EVC_TRACE_STR("mpm list: ");
    //#endif
    for i in 0..IntraPredDir::IPD_CNT_B as usize {
        if t0 == mpm[i] as u32 {
            ipm = i;
        }
        //#if TRACE_ADDITIONAL_FLAGS
        //        EVC_TRACE_INT(mpm[i]);
        //#endif
    }
    EVC_TRACE(&mut bs.tracer, "ipm Y ");
    EVC_TRACE(&mut bs.tracer, ipm);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(ipm as u8)
}

pub(crate) fn evcd_eco_cbf(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    pred_mode: PredMode,
    cbf: &mut [bool],
    b_no_cbf: bool,
    is_sub: bool,
    sub_pos: u8,
    cbf_all: &mut bool,
) -> Result<(), EvcError> {
    /* decode allcbf */
    if pred_mode != PredMode::MODE_INTRA {
        if b_no_cbf == false && sub_pos == 0 {
            if sbac.decode_bin(bs, &mut sbac_ctx.cbf_all[0])? == 0 {
                *cbf_all = false;
                cbf[Y_C] = false;
                cbf[U_C] = false;
                cbf[V_C] = false;

                EVC_TRACE_COUNTER(&mut bs.tracer);
                EVC_TRACE(&mut bs.tracer, "all_cbf ");
                EVC_TRACE(&mut bs.tracer, 0);
                EVC_TRACE(&mut bs.tracer, " \n");

                return Ok(());
            } else {
                EVC_TRACE_COUNTER(&mut bs.tracer);
                EVC_TRACE(&mut bs.tracer, "all_cbf ");
                EVC_TRACE(&mut bs.tracer, 1);
                EVC_TRACE(&mut bs.tracer, " \n");
            }
        }

        cbf[U_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cb[0])? != 0;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf U ");
        EVC_TRACE(&mut bs.tracer, cbf[U_C] as u8);
        EVC_TRACE(&mut bs.tracer, " \n");

        cbf[V_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cr[0])? != 0;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf V ");
        EVC_TRACE(&mut bs.tracer, cbf[V_C] as u8);
        EVC_TRACE(&mut bs.tracer, " \n");

        if cbf[U_C] == false && cbf[V_C] == false && !is_sub {
            cbf[Y_C] = true;
        } else {
            cbf[Y_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_luma[0])? != 0;
            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "cbf Y ");
            EVC_TRACE(&mut bs.tracer, cbf[Y_C] as u8);
            EVC_TRACE(&mut bs.tracer, " \n");
        }
    } else {
        cbf[U_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cb[0])? != 0;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf U ");
        EVC_TRACE(&mut bs.tracer, cbf[U_C] as u8);
        EVC_TRACE(&mut bs.tracer, " \n");

        cbf[V_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cr[0])? != 0;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf V ");
        EVC_TRACE(&mut bs.tracer, cbf[V_C] as u8);
        EVC_TRACE(&mut bs.tracer, " \n");

        cbf[Y_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_luma[0])? != 0;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf Y ");
        EVC_TRACE(&mut bs.tracer, cbf[Y_C] as u8);
        EVC_TRACE(&mut bs.tracer, " \n");
    }

    Ok(())
}

pub(crate) fn evcd_eco_dqp(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
) -> Result<i8, EvcError> {
    let mut dqp = sbac.read_unary_sym(bs, &mut sbac_ctx.delta_qp, NUM_CTX_DELTA_QP as u32)? as i8;

    if dqp > 0 {
        let sign = sbac.decode_bin_ep(bs)?;
        dqp = if sign != 0 { -dqp } else { dqp };
    }

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "dqp ");
    EVC_TRACE(&mut bs.tracer, dqp);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(dqp)
}

pub(crate) fn evcd_eco_xcoef(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    coef: &mut [i16],
    log2_w: u8,
    log2_h: u8,
    ch_type: usize,
) -> Result<(), EvcError> {
    evcd_eco_run_length_cc(bs, sbac, sbac_ctx, coef, log2_w, log2_h, ch_type)?;

    TRACE_COEF(
        &mut bs.tracer,
        ch_type,
        1 << log2_w as usize,
        1 << log2_h as usize,
        coef,
    );

    Ok(())
}

pub(crate) fn evcd_eco_run_length_cc(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    coef: &mut [i16],
    log2_w: u8,
    log2_h: u8,
    ch_type: usize,
) -> Result<(), EvcError> {
    let scanp = &evc_scan_tbl[log2_w as usize - 1];
    let num_coeff = 1 << (log2_w + log2_h) as u32;
    let mut scan_pos_offset = 0;
    let mut prev_level = 6;
    let mut coef_cnt = 0;

    let mut last_flag = false;
    while !last_flag {
        let t0 = if ch_type == Y_C { 0 } else { 2 };

        /* Run parsing */
        let run = sbac.read_unary_sym(bs, &mut sbac_ctx.run[t0..], 2)?;
        for i in scan_pos_offset..scan_pos_offset + run {
            coef[scanp[i as usize] as usize] = 0;
        }
        scan_pos_offset += run;

        /* Level parsing */
        let level = sbac.read_unary_sym(bs, &mut sbac_ctx.level[t0..], 2)? + 1;
        prev_level = level;

        /* Sign parsing */
        let sign = sbac.decode_bin_ep(bs)?;
        coef[scanp[scan_pos_offset as usize] as usize] = if sign != 0 {
            -(level as i16)
        } else {
            level as i16
        };

        coef_cnt += 1;

        if scan_pos_offset >= num_coeff - 1 {
            break;
        }
        scan_pos_offset += 1;

        /* Last flag parsing */
        let ctx_last = if ch_type == Y_C { 0 } else { 1 };
        last_flag = sbac.decode_bin(bs, &mut sbac_ctx.last[ctx_last])? != 0;
    }

    //#if ENC_DEC_TRACE
    /*EVC_TRACE(&mut bs.tracer, "coef luma ");
    for scan_pos_offset in 0..num_coeff as usize {
        EVC_TRACE(&mut bs.tracer, coef[scan_pos_offset]);
        EVC_TRACE(&mut bs.tracer, " ");
    }
    EVC_TRACE(&mut bs.tracer, "\n");*/
    //#endif

    Ok(())
}

fn evcd_eco_coef(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    core: &mut EvcdCore,
    log2_cuw: u8,
    log2_cuh: u8,
    sps_dquant_flag: bool,
    pps_cu_qp_delta_enabled_flag: bool,
    sh_qp_u_offset: i8,
    sh_qp_v_offset: i8,
) -> Result<(), EvcError> {
    let mut cbf = [false; N_C];
    let mut b_no_cbf = false;

    let mut tmp_coef = [0; N_C];
    let is_sub = false;
    let mut cbf_all = true;

    if cbf_all {
        evcd_eco_cbf(
            bs,
            sbac,
            sbac_ctx,
            core.pred_mode,
            &mut cbf,
            b_no_cbf,
            is_sub,
            0,
            &mut cbf_all,
        )?;
    } else {
        cbf[Y_C] = false;
        cbf[U_C] = false;
        cbf[V_C] = false;
    }

    let mut dqp = 0;
    if pps_cu_qp_delta_enabled_flag
        && (((!(sps_dquant_flag) || (core.cu_qp_delta_code == 1 && !core.cu_qp_delta_is_coded))
            && (cbf[Y_C] || cbf[U_C] || cbf[V_C]))
            || (core.cu_qp_delta_code == 2 && !core.cu_qp_delta_is_coded))
    {
        dqp = evcd_eco_dqp(bs, sbac, sbac_ctx)?;
        core.cu_qp_delta_is_coded = true;
    } else {
        dqp = 0;
    }
    core.qp = GET_QP(core.qp as i8, dqp) as u8;
    core.qp_y = GET_LUMA_QP(core.qp as i8) as u8;

    let qp_i_cb = EVC_CLIP3(
        -6 * (BIT_DEPTH as i8 - 8),
        57,
        (core.qp as i8 + sh_qp_u_offset) as i8,
    );
    let qp_i_cr = EVC_CLIP3(
        -6 * (BIT_DEPTH as i8 - 8),
        57,
        (core.qp as i8 + sh_qp_v_offset) as i8,
    );
    core.qp_u = (core.evc_tbl_qp_chroma_dynamic_ext[0]
        [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
        + (6 * (BIT_DEPTH - 8)) as i8) as u8;
    core.qp_v = (core.evc_tbl_qp_chroma_dynamic_ext[1]
        [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
        + (6 * (BIT_DEPTH - 8)) as i8) as u8;

    for c in 0..N_C {
        if cbf[c] {
            let chroma = if c > 0 { 1 } else { 0 };
            evcd_eco_xcoef(
                bs,
                sbac,
                sbac_ctx,
                &mut core.coef.data[c],
                log2_cuw - chroma,
                log2_cuh - chroma,
                c,
            )?;

            tmp_coef[c] += 1;
        } else {
            tmp_coef[c] += 0;
        }
    }

    for c in 0..N_C {
        core.is_coef[c] = if tmp_coef[c] != 0 { true } else { false };
    }

    Ok(())
}

fn evcd_eco_cu(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    core: &mut EvcdCore,
    x: u16,
    y: u16,
    log2_cuw: u8,
    log2_cuh: u8,
    w_scu: u16,
    map_scu: &[MCU],
    map_ipm: &[IntraPredDir],
    dpm: &Option<EvcPm>,
    sps_dquant_flag: bool,
    pps_cu_qp_delta_enabled_flag: bool,
    sh_slice_type: SliceType,
    sh_qp: u8,
    sh_qp_u_offset: i8,
    sh_qp_v_offset: i8,
) -> Result<(), EvcError> {
    //CU position X in a frame in SCU unit
    let x_scu = PEL2SCU(x as usize) as u16;
    //CU position Y in a frame in SCU unit
    let y_scu = PEL2SCU(y as usize) as u16;
    //CU position in current frame in SCU unit
    let scup = x_scu as u32 + y_scu as u32 * w_scu as u32;

    core.refi[REFP_0] = 0;
    core.refi[REFP_1] = 0;
    core.mv[REFP_0][MV_X] = 0;
    core.mv[REFP_0][MV_Y] = 0;
    core.mv[REFP_1][MV_X] = 0;
    core.mv[REFP_1][MV_Y] = 0;

    core.pred_mode = PredMode::MODE_INTRA;
    core.mvp_idx[REFP_0] = 0;
    core.mvp_idx[REFP_1] = 0;
    core.inter_dir = InterPredDir::PRED_L0;
    for i in 0..REFP_NUM {
        for j in 0..MV_D {
            core.mvd[i][j] = 0;
        }
    }

    let cuw = 1 << log2_cuw;
    let cuh = 1 << log2_cuh;
    core.avail_lr = evc_check_nev_avail(x_scu, y_scu, cuw, w_scu, map_scu);

    if sh_slice_type != SliceType::EVC_ST_I {
        /* CU skip flag */
        let cu_skip_flag = evcd_eco_cu_skip_flag(bs, sbac, sbac_ctx, &core.ctx_flags)?;
        if cu_skip_flag != 0 {
            core.pred_mode = PredMode::MODE_SKIP;
        }
    }

    /* parse prediction info */
    if core.pred_mode == PredMode::MODE_SKIP {
        core.mvp_idx[REFP_0] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
        if sh_slice_type == SliceType::EVC_ST_B {
            core.mvp_idx[REFP_1] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
        }

        core.is_coef[Y_C] = false;
        core.is_coef[U_C] = false;
        core.is_coef[V_C] = false;

        core.qp = sh_qp;
        core.qp_y = GET_LUMA_QP(core.qp as i8) as u8;
        let qp_i_cb = EVC_CLIP3(
            -6 * (BIT_DEPTH - 8) as i8,
            57,
            core.qp as i8 + sh_qp_u_offset,
        );
        let qp_i_cr = EVC_CLIP3(
            -6 * (BIT_DEPTH - 8) as i8,
            57,
            core.qp as i8 + sh_qp_v_offset,
        );

        core.qp_u = (core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        core.qp_v = (core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;
    } else {
        core.pred_mode = evcd_eco_pred_mode(bs, sbac, sbac_ctx, &core.ctx_flags, sh_slice_type)?;

        if core.pred_mode == PredMode::MODE_INTER {
            //TODO: bugfix? missing SLICE_TYPE==B for direct_mode_flag?
            core.inter_dir = evcd_eco_direct_mode_flag(bs, sbac, sbac_ctx)?;

            if core.inter_dir != InterPredDir::PRED_DIR {
                /* inter_pred_idc */
                core.inter_dir = evcd_eco_inter_pred_idc(bs, sbac, sbac_ctx, sh_slice_type)?;

                for inter_dir_idx in 0..2 {
                    /* 0: forward, 1: backward */
                    if (((core.inter_dir as usize + 1) >> inter_dir_idx) & 1) != 0 {
                        core.refi[inter_dir_idx] = evcd_eco_refi(
                            bs,
                            sbac,
                            sbac_ctx,
                            dpm.as_ref().unwrap().num_refp[inter_dir_idx],
                        )? as i8;
                        core.mvp_idx[inter_dir_idx] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
                        evcd_eco_get_mvd(bs, sbac, sbac_ctx, &mut core.mvd[inter_dir_idx])?;
                    }
                }
            }
        } else if core.pred_mode == PredMode::MODE_INTRA {
            core.mpm_b_list = evc_get_mpm_b(x_scu, y_scu, map_scu, map_ipm, scup, w_scu);

            let mut luma_ipm = IntraPredDir::IPD_DC_B;
            core.ipm[0] = evcd_eco_intra_dir_b(bs, sbac, sbac_ctx, core.mpm_b_list)?.into();
            luma_ipm = core.ipm[0];
            core.ipm[1] = luma_ipm;

            core.refi[REFP_0] = REFI_INVALID;
            core.refi[REFP_1] = REFI_INVALID;
            core.mv[REFP_0][MV_X] = 0;
            core.mv[REFP_0][MV_Y] = 0;
            core.mv[REFP_1][MV_X] = 0;
            core.mv[REFP_1][MV_Y] = 0;
        } else {
            evc_assert_rv(false, EvcError::EVC_ERR_MALFORMED_BITSTREAM)?;
        }

        /* clear coefficient buffer */
        for i in 0..(cuw * cuh) as usize {
            core.coef.data[Y_C][i] = 0;
        }
        for i in 0..((cuw >> 1) * (cuh >> 1)) as usize {
            core.coef.data[U_C][i] = 0;
            core.coef.data[V_C][i] = 0;
        }

        /* parse coefficients */
        evcd_eco_coef(
            bs,
            sbac,
            sbac_ctx,
            core,
            log2_cuw,
            log2_cuh,
            sps_dquant_flag,
            pps_cu_qp_delta_enabled_flag,
            sh_qp_u_offset,
            sh_qp_v_offset,
        )?;
    }

    Ok(())
}

pub(crate) fn evcd_eco_unit(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    core: &mut EvcdCore,
    x: u16,
    y: u16,
    log2_cuw: u8,
    log2_cuh: u8,
    w_scu: u16,
    h_scu: u16,
    w: u16,
    h: u16,
    map_mv: &Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    refp: &Vec<Vec<EvcRefP>>,
    map_scu: &[MCU],
    map_ipm: &[IntraPredDir],
    dpm: &Option<EvcPm>,
    poc_val: i32,
    pic: &Option<Rc<RefCell<EvcPic>>>,
    sps_dquant_flag: bool,
    pps_cu_qp_delta_enabled_flag: bool,
    pps_constrained_intra_pred_flag: bool,
    sh_slice_type: SliceType,
    sh_qp: u8,
    sh_qp_u_offset: i8,
    sh_qp_v_offset: i8,
) -> Result<(), EvcError> {
    let cuw = 1 << log2_cuw;
    let cuh = 1 << log2_cuh;

    //CU position X in a frame in SCU unit
    let x_scu = PEL2SCU(x as usize) as u16;
    //CU position Y in a frame in SCU unit
    let y_scu = PEL2SCU(y as usize) as u16;
    //CU position in current frame in SCU unit
    let scup = x_scu as u32 + y_scu as u32 * w_scu as u32;

    //entropy decoding
    {
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "poc: ");
        EVC_TRACE(&mut bs.tracer, poc_val);
        EVC_TRACE(&mut bs.tracer, " x pos ");
        EVC_TRACE(&mut bs.tracer, x);
        EVC_TRACE(&mut bs.tracer, " y pos ");
        EVC_TRACE(&mut bs.tracer, y);
        EVC_TRACE(&mut bs.tracer, " width ");
        EVC_TRACE(&mut bs.tracer, cuw);
        EVC_TRACE(&mut bs.tracer, " height ");
        EVC_TRACE(&mut bs.tracer, cuh);
        EVC_TRACE(&mut bs.tracer, " \n");

        /* parse CU info */
        evcd_eco_cu(
            bs,
            sbac,
            sbac_ctx,
            core,
            x,
            y,
            log2_cuw,
            log2_cuh,
            w_scu,
            map_scu,
            map_ipm,
            dpm,
            sps_dquant_flag,
            pps_cu_qp_delta_enabled_flag,
            sh_slice_type,
            sh_qp,
            sh_qp_u_offset,
            sh_qp_v_offset,
        )?;
    }

    /* inverse transform and dequantization */
    if core.pred_mode != PredMode::MODE_SKIP {
        evc_sub_block_itdq(
            &mut bs.tracer,
            &mut core.coef.data,
            log2_cuw,
            log2_cuh,
            core.qp_y,
            core.qp_u,
            core.qp_v,
            &core.is_coef,
        );
    }

    /* prediction */
    if core.pred_mode != PredMode::MODE_INTRA {
        core.avail_cu = evc_get_avail_inter(
            x_scu as usize,
            y_scu as usize,
            w_scu as usize,
            h_scu as usize,
            scup as usize,
            cuw as usize,
            cuh as usize,
            map_scu,
        );
        if core.pred_mode == PredMode::MODE_SKIP {
            evcd_get_skip_motion(core, cuw, cuh, w_scu, scup, map_mv, refp, sh_slice_type);
        } else {
            if core.inter_dir == InterPredDir::PRED_DIR {
                evc_get_mv_dir(
                    &refp[0],
                    poc_val,
                    scup as usize
                        + ((1 << (log2_cuw as usize - MIN_CU_LOG2)) - 1)
                        + ((1 << (log2_cuh as usize - MIN_CU_LOG2)) - 1) * w_scu as usize,
                    scup as usize,
                    w_scu,
                    h_scu,
                    &mut core.mv,
                );
                core.refi[REFP_0] = 0;
                core.refi[REFP_1] = 0;
            } else {
                evcd_get_inter_motion(core, cuw, cuh, w_scu, scup, map_mv, refp);
            }
        }

        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "Inter: ");
        EVC_TRACE(&mut bs.tracer, core.inter_dir as isize);
        EVC_TRACE(&mut bs.tracer, " , mv[REFP_0]:( ");
        EVC_TRACE(&mut bs.tracer, core.mv[REFP_0][MV_X]);
        EVC_TRACE(&mut bs.tracer, " , ");
        EVC_TRACE(&mut bs.tracer, core.mv[REFP_0][MV_Y]);
        EVC_TRACE(&mut bs.tracer, " ), mv[REFP_1]:( ");
        EVC_TRACE(&mut bs.tracer, core.mv[REFP_1][MV_X]);
        EVC_TRACE(&mut bs.tracer, " , ");
        EVC_TRACE(&mut bs.tracer, core.mv[REFP_1][MV_Y]);
        EVC_TRACE(&mut bs.tracer, " )\n");

        evc_mc(
            x as i16,
            y as i16,
            w as i16,
            h as i16,
            cuw as i16,
            cuh as i16,
            &core.refi,
            &core.mv,
            refp,
            &mut core.pred,
            poc_val,
        );
    } else {
        core.avail_cu = evc_get_avail_intra(
            x_scu as usize,
            y_scu as usize,
            w_scu as usize,
            h_scu as usize,
            scup as usize,
            log2_cuw,
            log2_cuh,
            map_scu,
        );
        get_nbr_yuv(
            core,
            x,
            y,
            cuw,
            cuh,
            w_scu,
            h_scu,
            scup,
            map_scu,
            pic,
            pps_constrained_intra_pred_flag,
        );

        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "Intra: ");
        EVC_TRACE(&mut bs.tracer, core.ipm[0] as isize);
        EVC_TRACE(&mut bs.tracer, " , ");
        EVC_TRACE(&mut bs.tracer, core.ipm[1] as isize);
        EVC_TRACE(&mut bs.tracer, " \n");

        evc_ipred_b(
            &core.nb.data[Y_C][0][2..],
            &core.nb.data[Y_C][1][cuh as usize..],
            core.nb.data[Y_C][1][cuh as usize - 1],
            &mut core.pred[0].data[Y_C],
            core.ipm[0],
            cuw as usize,
            cuh as usize,
        );

        evc_ipred_b(
            &core.nb.data[U_C][0][2..],
            &core.nb.data[U_C][1][(cuh >> 1) as usize..],
            core.nb.data[U_C][1][(cuh >> 1) as usize - 1],
            &mut core.pred[0].data[U_C],
            core.ipm[1],
            cuw as usize >> 1,
            cuh as usize >> 1,
        );
        evc_ipred_b(
            &core.nb.data[V_C][0][2..],
            &core.nb.data[V_C][1][(cuh >> 1) as usize..],
            core.nb.data[V_C][1][(cuh >> 1) as usize - 1],
            &mut core.pred[0].data[V_C],
            core.ipm[1],
            cuw as usize >> 1,
            cuh as usize >> 1,
        );
    }

    TRACE_PRED(
        &mut bs.tracer,
        Y_C,
        cuw as usize,
        cuh as usize,
        &core.pred[0].data[Y_C],
    );
    TRACE_PRED(
        &mut bs.tracer,
        U_C,
        cuw as usize >> 1,
        cuh as usize >> 1,
        &core.pred[0].data[U_C],
    );
    TRACE_PRED(
        &mut bs.tracer,
        V_C,
        cuw as usize >> 1,
        cuh as usize >> 1,
        &core.pred[0].data[V_C],
    );

    /* reconstruction */
    if let Some(p) = &pic {
        evc_recon_yuv(
            &mut bs.tracer,
            x as usize,
            y as usize,
            cuw as usize,
            cuh as usize,
            &core.coef.data,
            &core.pred[0].data,
            &core.is_coef,
            &mut p.borrow().frame.borrow_mut().planes,
        );
    }

    Ok(())
}

fn evcd_get_skip_motion(
    core: &mut EvcdCore,
    cuw: u8,
    cuh: u8,
    w_scu: u16,
    scup: u32,
    map_mv: &Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    refp: &Vec<Vec<EvcRefP>>,
    sh_slice_type: SliceType,
) {
    let mut srefi = [[0i8; MAX_NUM_MVP]; REFP_NUM];
    let mut smvp = [[[0i16; MV_D]; MAX_NUM_MVP]; REFP_NUM];

    let map_mv = map_mv.as_ref().unwrap().borrow();

    evc_get_motion(
        scup as usize,
        REFP_0,
        &*map_mv,
        refp,
        cuw as usize,
        cuh as usize,
        w_scu as usize,
        core.avail_cu,
        &mut srefi[REFP_0],
        &mut smvp[REFP_0],
    );

    core.refi[REFP_0] = srefi[REFP_0][core.mvp_idx[REFP_0] as usize];

    core.mv[REFP_0][MV_X] = smvp[REFP_0][core.mvp_idx[REFP_0] as usize][MV_X];
    core.mv[REFP_0][MV_Y] = smvp[REFP_0][core.mvp_idx[REFP_0] as usize][MV_Y];

    if sh_slice_type == SliceType::EVC_ST_P {
        core.refi[REFP_1] = REFI_INVALID;
        core.mv[REFP_1][MV_X] = 0;
        core.mv[REFP_1][MV_Y] = 0;
    } else {
        evc_get_motion(
            scup as usize,
            REFP_1,
            &*map_mv,
            refp,
            cuw as usize,
            cuh as usize,
            w_scu as usize,
            core.avail_cu,
            &mut srefi[REFP_1],
            &mut smvp[REFP_1],
        );

        core.refi[REFP_1] = srefi[REFP_1][core.mvp_idx[REFP_1] as usize];
        core.mv[REFP_1][MV_X] = smvp[REFP_1][core.mvp_idx[REFP_1] as usize][MV_X];
        core.mv[REFP_1][MV_Y] = smvp[REFP_1][core.mvp_idx[REFP_1] as usize][MV_Y];
    }
}

fn evcd_get_inter_motion(
    core: &mut EvcdCore,
    cuw: u8,
    cuh: u8,
    w_scu: u16,
    scup: u32,
    map_mv: &Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    refp: &Vec<Vec<EvcRefP>>,
) {
    let mut mvp = [[0i16; MV_D]; MAX_NUM_MVP];
    let mut refi = [0i8; MAX_NUM_MVP];

    let map_mv = map_mv.as_ref().unwrap().borrow();

    for inter_dir_idx in 0..2 {
        /* 0: forward, 1: backward */
        if (((core.inter_dir as usize + 1) >> inter_dir_idx) & 1) != 0 {
            evc_get_motion(
                scup as usize,
                inter_dir_idx,
                &*map_mv,
                refp,
                cuw as usize,
                cuh as usize,
                w_scu as usize,
                core.avail_cu,
                &mut refi,
                &mut mvp,
            );
            core.mv[inter_dir_idx][MV_X] =
                mvp[core.mvp_idx[inter_dir_idx] as usize][MV_X] + core.mvd[inter_dir_idx][MV_X];
            core.mv[inter_dir_idx][MV_Y] =
                mvp[core.mvp_idx[inter_dir_idx] as usize][MV_Y] + core.mvd[inter_dir_idx][MV_Y];
        } else {
            core.refi[inter_dir_idx] = REFI_INVALID;
            core.mv[inter_dir_idx][MV_X] = 0;
            core.mv[inter_dir_idx][MV_Y] = 0;
        }
    }
}

fn get_nbr_yuv(
    core: &mut EvcdCore,
    mut x: u16,
    mut y: u16,
    mut cuw: u8,
    mut cuh: u8,
    w_scu: u16,
    h_scu: u16,
    scup: u32,
    map_scu: &[MCU],
    pic: &Option<Rc<RefCell<EvcPic>>>,
    pps_constrained_intra_pred_flag: bool,
) {
    let constrained_intra_flag =
        core.pred_mode == PredMode::MODE_INTRA && pps_constrained_intra_pred_flag;

    if let Some(pic) = &pic {
        let frame = &pic.borrow().frame;
        let planes = &frame.borrow().planes;
        /* Y */
        evc_get_nbr_b(
            x as usize,
            y as usize,
            cuw as usize,
            cuh as usize,
            &planes[Y_C].as_region(),
            core.avail_cu,
            &mut core.nb.data[Y_C],
            scup as usize,
            map_scu,
            w_scu as usize,
            h_scu as usize,
            Y_C,
            constrained_intra_flag,
        );

        cuw >>= 1;
        cuh >>= 1;
        x >>= 1;
        y >>= 1;

        /* U */
        evc_get_nbr_b(
            x as usize,
            y as usize,
            cuw as usize,
            cuh as usize,
            &planes[U_C].as_region(),
            core.avail_cu,
            &mut core.nb.data[U_C],
            scup as usize,
            map_scu,
            w_scu as usize,
            h_scu as usize,
            U_C,
            constrained_intra_flag,
        );

        /* V */
        evc_get_nbr_b(
            x as usize,
            y as usize,
            cuw as usize,
            cuh as usize,
            &planes[V_C].as_region(),
            core.avail_cu,
            &mut core.nb.data[V_C],
            scup as usize,
            map_scu,
            w_scu as usize,
            h_scu as usize,
            V_C,
            constrained_intra_flag,
        );
    }
}

pub(crate) fn evcd_set_dec_info(
    core: &mut EvcdCore,
    x: u16,
    y: u16,
    log2_cuw: u8,
    log2_cuh: u8,
    w_scu: usize,
    pps_cu_qp_delta_enabled_flag: bool,
    slice_num: u16,
    map_refi: &mut Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    map_mv: &mut Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    map_scu: &mut [MCU],
    map_cu_mode: &mut [MCU],
    map_ipm: &mut [IntraPredDir],
) {
    //CU position X in a frame in SCU unit
    let x_scu = PEL2SCU(x as usize) as usize;
    //CU position Y in a frame in SCU unit
    let y_scu = PEL2SCU(y as usize) as usize;
    //CU position in current frame in SCU unit
    let scup = x_scu + y_scu * w_scu;

    let w_cu = (1 << log2_cuw) >> MIN_CU_LOG2;
    let h_cu = (1 << log2_cuh) >> MIN_CU_LOG2;
    let flag = if core.pred_mode == PredMode::MODE_INTRA {
        1
    } else {
        0
    };

    if let (Some(map_refi), Some(map_mv)) = (map_refi, map_mv) {
        let (mut refis, mut mvs) = (map_refi.borrow_mut(), map_mv.borrow_mut());

        for i in 0..h_cu {
            let map_scu = &mut map_scu[scup + i * w_scu..];
            let map_ipm = &mut map_ipm[scup + i * w_scu..];
            let map_cu_mode = &mut map_cu_mode[scup + i * w_scu..];
            let refi = &mut refis[scup + i * w_scu..];
            let mv = &mut mvs[scup + i * w_scu..];

            for j in 0..w_cu {
                if core.pred_mode == PredMode::MODE_SKIP {
                    map_scu[j].SET_SF();
                } else {
                    map_scu[j].CLR_SF();
                }
                if core.is_coef[Y_C] {
                    map_scu[j].SET_CBFL();
                } else {
                    map_scu[j].CLR_CBFL();
                }

                map_cu_mode[j].SET_LOGW(log2_cuw as u32);
                map_cu_mode[j].SET_LOGH(log2_cuh as u32);

                if pps_cu_qp_delta_enabled_flag {
                    map_scu[j].RESET_QP();
                }
                map_scu[j].SET_IF_COD_SN_QP(flag, slice_num as u32, core.qp);

                map_ipm[j] = core.ipm[0];

                refi[j][REFP_0] = core.refi[REFP_0];
                refi[j][REFP_1] = core.refi[REFP_1];
                mv[j][REFP_0][MV_X] = core.mv[REFP_0][MV_X];
                mv[j][REFP_0][MV_Y] = core.mv[REFP_0][MV_Y];
                mv[j][REFP_1][MV_X] = core.mv[REFP_1][MV_X];
                mv[j][REFP_1][MV_Y] = core.mv[REFP_1][MV_Y];
            }
        }
    }
}
