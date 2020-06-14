use super::EvcdBsr;
use crate::api::EvcError;
use crate::com::*;

use log::*;

pub(crate) fn evcd_eco_nalu(bs: &mut EvcdBsr) -> Result<EvcNalu, EvcError> {
    let mut nalu = EvcNalu::default();

    //nalu->nal_unit_size = bs.read(32);
    nalu.forbidden_zero_bit = bs.read(1) as u8;

    if nalu.forbidden_zero_bit != 0 {
        error!("malformed bitstream: forbidden_zero_bit != 0\n");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nal_unit_type = bs.read(6) as u8 - 1;
    nalu.nuh_temporal_id = bs.read(3) as u8;
    nalu.nuh_reserved_zero_5bits = bs.read(5) as u8;

    if nalu.nuh_reserved_zero_5bits != 0 {
        error!("malformed bitstream: nuh_reserved_zero_5bits != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    nalu.nuh_extension_flag = bs.read(1) != 0;

    if nalu.nuh_extension_flag {
        error!("malformed bitstream: nuh_extension_flag != 0");
        return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
    }

    Ok(nalu)
}

pub(crate) fn evcd_eco_sps(bs: &mut EvcdBsr, sps: &mut EvcSps) -> Result<(), EvcError> {
    /*#if TRACE_HLS
        EVC_TRACE_STR("***********************************\n");
        EVC_TRACE_STR("************ SPS Start ************\n");
    #endif*/
    sps.sps_seq_parameter_set_id = bs.read_ue() as u8;
    sps.profile_idc = bs.read(8) as u8;
    sps.level_idc = bs.read(8) as u8;
    sps.toolset_idc_h = bs.read(32);
    sps.toolset_idc_l = bs.read(32);
    sps.chroma_format_idc = bs.read_ue() as u8;
    sps.pic_width_in_luma_samples = bs.read_ue() as u16;
    sps.pic_height_in_luma_samples = bs.read_ue() as u16;
    sps.bit_depth_luma_minus8 = bs.read_ue() as u8;
    sps.bit_depth_chroma_minus8 = bs.read_ue() as u8;
    let _sps_btt_flag = bs.read1();
    let _sps_suco_flag = bs.read1();
    let _tool_admvp = bs.read1();
    let _tool_eipd = bs.read1();
    let _tool_cm_init = bs.read1();
    let _tool_iqt = bs.read1();
    let _tool_addb = bs.read1();
    let _tool_dra = bs.read1();
    let _tool_alf = bs.read1();
    let _tool_htdf = bs.read1();
    let _tool_rpl = bs.read1() != 0;
    let _tool_pocs = bs.read1() != 0;
    let _dquant_flag = bs.read1();
    let _tool_dra = bs.read1();
    if !_tool_rpl || !_tool_pocs {
        sps.log2_sub_gop_length = bs.read_ue() as u8;
        if sps.log2_sub_gop_length == 0 {
            sps.log2_ref_pic_gap_length = bs.read_ue() as u8;
        }
    }
    if !_tool_rpl {
        sps.max_num_ref_pics = bs.read_ue() as u8;
    }

    sps.picture_cropping_flag = bs.read1() != 0;
    if sps.picture_cropping_flag {
        sps.picture_crop_left_offset = bs.read_ue() as u16;
        sps.picture_crop_right_offset = bs.read_ue() as u16;
        sps.picture_crop_top_offset = bs.read_ue() as u16;
        sps.picture_crop_bottom_offset = bs.read_ue() as u16;
    }

    if (sps.chroma_format_idc != 0) {
        sps.chroma_qp_table_struct.chroma_qp_table_present_flag = bs.read1() != 0;
        if sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            sps.chroma_qp_table_struct.same_qp_table_for_chroma = bs.read1() != 0;
            sps.chroma_qp_table_struct.global_offset_flag = bs.read1() != 0;
            for i in 0..if sps.chroma_qp_table_struct.same_qp_table_for_chroma {
                1
            } else {
                2
            } {
                sps.chroma_qp_table_struct.num_points_in_qp_table_minus1[i] = bs.read_ue() as usize;
                for j in 0..=sps.chroma_qp_table_struct.num_points_in_qp_table_minus1[i] {
                    sps.chroma_qp_table_struct.delta_qp_in_val_minus1[i][j] = bs.read(6) as i8;
                    sps.chroma_qp_table_struct.delta_qp_out_val[i][j] = bs.read_se() as i8;
                }
            }
        }
    }

    sps.vui_parameters_present_flag = bs.read1() != 0;
    if sps.vui_parameters_present_flag {
        //sps.vui_parameters = evcd_eco_vui(bs)?;
    }

    while !bs.EVC_BSR_IS_BYTE_ALIGN() {
        bs.read1();
    }
    //#if TRACE_HLS
    //    EVC_TRACE_STR("************ SPS End   ************\n");
    //    EVC_TRACE_STR("***********************************\n");
    //#endif

    Ok(())
}
