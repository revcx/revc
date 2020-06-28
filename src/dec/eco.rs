use super::bsr::*;
use super::sbac::EvcdSbac;
use super::{EvcdCore, EvcdCtx};

use crate::api::{EvcError, NaluType, SliceType};
use crate::com::util::*;
use crate::com::*;

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
    let ctx_flag = ctx_flags[CtxNevIdx::CNID_SKIP_FLAG as usize] as usize;
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
    tree_cons: &TREE_CONS,
) -> Result<PredMode, EvcError> {
    let mut pred_mode_flag = false;
    let pred_mode_constraint = tree_cons.mode_cons; //TODO: Tim changed place

    if pred_mode_constraint == MODE_CONS::eAll {
        let ctx_flag = ctx_flags[CtxNevIdx::CNID_PRED_MODE as usize] as usize;
        pred_mode_flag = sbac.decode_bin(bs, &mut sbac_ctx.pred_mode[ctx_flag])? != 0;

        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "pred mode ");
        EVC_TRACE(
            &mut bs.tracer,
            if pred_mode_flag {
                PredMode::MODE_INTRA
            } else {
                PredMode::MODE_INTER
            } as u8,
        );
        EVC_TRACE(&mut bs.tracer, " \n");
    }

    let pred_mode = if pred_mode_constraint == MODE_CONS::eOnlyInter {
        PredMode::MODE_INTER
    } else if pred_mode_constraint == MODE_CONS::eOnlyIntra {
        PredMode::MODE_INTRA
    } else {
        if pred_mode_flag {
            PredMode::MODE_INTRA
        } else {
            PredMode::MODE_INTER
        }
    };

    Ok(pred_mode)
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

pub(crate) fn evcd_eco_direct_mode_flag(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
) -> Result<PredDir, EvcError> {
    let inter_dir = if sbac.decode_bin(bs, &mut sbac_ctx.direct_mode_flag[0])? != 0 {
        PredDir::PRED_DIR
    } else {
        PredDir::PRED_L0
    };

    EVC_TRACE_COUNTER(&mut bs.tracer);
    EVC_TRACE(&mut bs.tracer, "direct_mode_flag ");
    EVC_TRACE(&mut bs.tracer, inter_dir as u8);
    EVC_TRACE(&mut bs.tracer, " \n");

    Ok(inter_dir)
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

pub(crate) fn eco_cbf(
    bs: &mut EvcdBsr,
    sbac: &mut EvcdSbac,
    sbac_ctx: &mut EvcSbacCtx,
    pred_mode: PredMode,
    cbf: &mut [u8],
    b_no_cbf: bool,
    is_sub: bool,
    sub_pos: u8,
    cbf_all: &mut bool,
    tree_cons: &TREE_CONS,
) -> Result<(), EvcError> {
    /* decode allcbf */
    if pred_mode != PredMode::MODE_INTRA && tree_cons.tree_type == TREE_TYPE::TREE_LC {
        if b_no_cbf == false && sub_pos == 0 {
            if sbac.decode_bin(bs, &mut sbac_ctx.cbf_all[0])? == 0 {
                *cbf_all = false;
                cbf[Y_C] = 0;
                cbf[U_C] = 0;
                cbf[V_C] = 0;

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

        cbf[U_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cb[0])? as u8;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf U ");
        EVC_TRACE(&mut bs.tracer, cbf[U_C]);
        EVC_TRACE(&mut bs.tracer, " \n");

        cbf[V_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cr[0])? as u8;
        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "cbf V ");
        EVC_TRACE(&mut bs.tracer, cbf[V_C]);
        EVC_TRACE(&mut bs.tracer, " \n");

        if cbf[U_C] + cbf[V_C] == 0 && !is_sub {
            cbf[Y_C] = 1;
        } else {
            cbf[Y_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_luma[0])? as u8;
            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "cbf Y ");
            EVC_TRACE(&mut bs.tracer, cbf[Y_C]);
            EVC_TRACE(&mut bs.tracer, " \n");
        }
    } else {
        if evc_check_chroma(tree_cons) {
            cbf[U_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cb[0])? as u8;
            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "cbf U ");
            EVC_TRACE(&mut bs.tracer, cbf[U_C]);
            EVC_TRACE(&mut bs.tracer, " \n");

            cbf[V_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_cr[0])? as u8;
            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "cbf V ");
            EVC_TRACE(&mut bs.tracer, cbf[V_C]);
            EVC_TRACE(&mut bs.tracer, " \n");
        } else {
            cbf[U_C] = 0;
            cbf[V_C] = 0;
        }
        if evc_check_luma(tree_cons) {
            cbf[Y_C] = sbac.decode_bin(bs, &mut sbac_ctx.cbf_luma[0])? as u8;
            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "cbf Y ");
            EVC_TRACE(&mut bs.tracer, cbf[Y_C]);
            EVC_TRACE(&mut bs.tracer, " \n");
        } else {
            cbf[Y_C] = 0;
        }
    }

    Ok(())
}
