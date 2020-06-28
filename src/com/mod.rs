pub(crate) mod context;
pub(crate) mod ipred;
pub(crate) mod plane;
pub(crate) mod plane_region;
pub(crate) mod tbl;
pub(crate) mod util;

use crate::api::*;

/*****************************************************************************
 * types
 *****************************************************************************/
pub(crate) type pel = i16;
pub(crate) type double_pel = i32;

#[inline]
pub(crate) fn evc_assert_rv(x: bool, r: EvcError) -> Result<(), EvcError> {
    if !x {
        assert!(x);
        return Err(r);
    }
    Ok(())
}

/********* Conditional tools definition ********/

/* number of picture order count lsb bit */
pub(crate) const POC_LSB_BIT: usize = (11);

pub(crate) const BIT_DEPTH: usize = 10;
//pub(crate) const PEL2BYTE(pel)                      ((pel)*((BIT_DEPTH + 7)>>3))

//pub(crate) const STRIDE_IMGB2PIC(s_imgb)            ((s_imgb)>>1)

pub(crate) const Y_C: usize = 0; /* Y luma */
pub(crate) const U_C: usize = 1; /* Cb Chroma */
pub(crate) const V_C: usize = 2; /* Cr Chroma */
pub(crate) const N_C: usize = 3; /* number of color component */

pub(crate) const REFP_0: usize = 0;
pub(crate) const REFP_1: usize = 1;
pub(crate) const REFP_NUM: usize = 2;

/*****************************************************************************
 * reference index
 *****************************************************************************/
pub(crate) const REFI_INVALID: i8 = (-1);

#[inline]
pub(crate) fn REFI_IS_VALID(refi: i8) -> bool {
    refi >= 0
}

/* X direction motion vector indicator */
pub(crate) const MV_X: usize = 0;
/* Y direction motion vector indicator */
pub(crate) const MV_Y: usize = 1;
/* Maximum count (dimension) of motion */
pub(crate) const MV_D: usize = 2;
/* Reference index indicator */
pub(crate) const REFI: usize = 2;

pub(crate) const N_REF: usize = 3; /* left, up, right */
pub(crate) const NUM_NEIB: usize = 4; /* LR: 00, 10, 01, 11*/

pub(crate) const MAX_CU_LOG2: usize = 7;
pub(crate) const MIN_CU_LOG2: usize = 2;
pub(crate) const MAX_CU_SIZE: usize = (1 << MAX_CU_LOG2);
pub(crate) const MIN_CU_SIZE: usize = (1 << MIN_CU_LOG2);
pub(crate) const MAX_CU_DIM: usize = (MAX_CU_SIZE * MAX_CU_SIZE);
pub(crate) const MIN_CU_DIM: usize = (MIN_CU_SIZE * MIN_CU_SIZE);
pub(crate) const MAX_CU_DEPTH: usize = 10; /* 128x128 ~ 4x4 */
pub(crate) const NUM_CU_DEPTH: usize = (MAX_CU_DEPTH + 1);

pub(crate) const MAX_TR_LOG2: usize = 6; /* 64x64 */
pub(crate) const MIN_TR_LOG2: usize = 1; /* 2x2 */
pub(crate) const MAX_TR_SIZE: usize = (1 << MAX_TR_LOG2);
pub(crate) const MIN_TR_SIZE: usize = (1 << MIN_TR_LOG2);
pub(crate) const MAX_TR_DIM: usize = (MAX_TR_SIZE * MAX_TR_SIZE);
pub(crate) const MIN_TR_DIM: usize = (MIN_TR_SIZE * MIN_TR_SIZE);

pub(crate) const MAX_BEF_DATA_NUM: usize = (NUM_NEIB << 1);

/* maximum CB count in a LCB */
pub(crate) const MAX_CU_CNT_IN_LCU: usize = (MAX_CU_DIM / MIN_CU_DIM);
/* pixel position to SCB position */
#[inline]
pub(crate) fn PEL2SCU(p: usize) -> usize {
    p >> MIN_CU_LOG2
}

pub(crate) const PIC_PAD_SIZE_L: usize = (MAX_CU_SIZE + 16);
pub(crate) const PIC_PAD_SIZE_C: usize = (PIC_PAD_SIZE_L >> 1);

/* number of MVP candidates */
pub(crate) const MAX_NUM_MVP: usize = 4;

pub(crate) const COEF_SCAN_ZIGZAG: usize = 0;

/* for GOP 16 test, increase to 32 */
/* maximum reference picture count. Originally, Max. 16 */
/* for GOP 16 test, increase to 32 */

/* DPB Extra size */
pub(crate) const EXTRA_FRAME: usize = MAX_NUM_ACTIVE_REF_FRAME;

/* maximum picture buffer size */
pub(crate) const MAX_PB_SIZE: usize = (MAX_NUM_REF_PICS + EXTRA_FRAME);

pub(crate) const MAX_NUM_TILES_ROW: usize = 22;
pub(crate) const MAX_NUM_TILES_COL: usize = 20;

/* Neighboring block availability flag bits */
pub(crate) const AVAIL_BIT_UP: usize = 0;
pub(crate) const AVAIL_BIT_LE: usize = 1;
pub(crate) const AVAIL_BIT_RI: usize = 3;
pub(crate) const AVAIL_BIT_LO: usize = 4;
pub(crate) const AVAIL_BIT_UP_LE: usize = 5;
pub(crate) const AVAIL_BIT_UP_RI: usize = 6;
pub(crate) const AVAIL_BIT_LO_LE: usize = 7;
pub(crate) const AVAIL_BIT_LO_RI: usize = 8;
pub(crate) const AVAIL_BIT_RI_UP: usize = 9;
pub(crate) const AVAIL_BIT_UP_LE_LE: usize = 10;
pub(crate) const AVAIL_BIT_UP_RI_RI: usize = 11;

/* Neighboring block availability flags */
pub(crate) const AVAIL_UP: usize = (1 << AVAIL_BIT_UP);
pub(crate) const AVAIL_LE: usize = (1 << AVAIL_BIT_LE);
pub(crate) const AVAIL_RI: usize = (1 << AVAIL_BIT_RI);
pub(crate) const AVAIL_LO: usize = (1 << AVAIL_BIT_LO);
pub(crate) const AVAIL_UP_LE: usize = (1 << AVAIL_BIT_UP_LE);
pub(crate) const AVAIL_UP_RI: usize = (1 << AVAIL_BIT_UP_RI);
pub(crate) const AVAIL_LO_LE: usize = (1 << AVAIL_BIT_LO_LE);
pub(crate) const AVAIL_LO_RI: usize = (1 << AVAIL_BIT_LO_RI);
pub(crate) const AVAIL_RI_UP: usize = (1 << AVAIL_BIT_RI_UP);
pub(crate) const AVAIL_UP_LE_LE: usize = (1 << AVAIL_BIT_UP_LE_LE);
pub(crate) const AVAIL_UP_RI_RI: usize = (1 << AVAIL_BIT_UP_RI_RI);

/* MB availability check macro */
//pub(crate) const IS_AVAIL(avail, pos)  : usize =        (((avail)&(pos)) == (pos))
/* MB availability set macro */
//pub(crate) const SET_AVAIL(avail, pos) : usize =          (avail) |= (pos)
/* MB availability remove macro */
//pub(crate) const REM_AVAIL(avail, pos)    : usize =         (avail) &= (~(pos))
/* MB availability into bit flag */
//pub(crate) const GET_AVAIL_FLAG(avail, bit)      (((avail)>>(bit)) & 0x1)

#[inline]
pub(crate) fn GET_QP(qp: i8, dqp: i8) -> i8 {
    ((qp + dqp + 52) % 52)
}
#[inline]
pub(crate) fn GET_LUMA_QP(qp: i8) -> i8 {
    (qp + 6 * (BIT_DEPTH - 8) as i8)
}
/*****************************************************************************
 * prediction mode
 *****************************************************************************/
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PredMode {
    MODE_INTRA = 0,
    MODE_INTER = 1,
    MODE_SKIP = 2,
    MODE_DIR = 3,
    MODE_SKIP_MMVD = 4,
    MODE_DIR_MMVD = 5,
    MODE_IBC = 6,
}

impl Default for PredMode {
    fn default() -> Self {
        PredMode::MODE_INTRA
    }
}

/*****************************************************************************
 * prediction direction
 *****************************************************************************/
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PredDir {
    /* inter pred direction, look list0 side */
    PRED_L0 = 0,
    /* inter pred direction, look list1 side */
    PRED_L1 = 1,
    /* inter pred direction, look both list0, list1 side */
    PRED_BI = 2,
    /* inter pred direction, look both list0, list1 side */
    PRED_SKIP = 3,
    /* inter pred direction, look both list0, list1 side */
    PRED_DIR = 4,
}

impl Default for PredDir {
    fn default() -> Self {
        PredDir::PRED_L0
    }
}

/*****************************************************************************
 * intra prediction direction
 *****************************************************************************/
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum IntraPredDir {
    IPD_UNKNOWN = -1,
    IPD_DC_B = 0,
    IPD_HOR_B = 1, /* Luma, Horizontal */
    IPD_VER_B = 2, /* Luma, Vertical */
    IPD_UL_B = 3,
    IPD_UR_B = 4,
    IPD_CNT_B = 5,
}

impl Default for IntraPredDir {
    fn default() -> Self {
        IntraPredDir::IPD_UNKNOWN
    }
}

impl From<u8> for IntraPredDir {
    fn from(val: u8) -> Self {
        use self::IntraPredDir::*;
        match val {
            0 => IPD_DC_B,
            1 => IPD_HOR_B,
            2 => IPD_VER_B,
            3 => IPD_UL_B,
            4 => IPD_UR_B,
            5 => IPD_CNT_B,
            _ => IPD_UNKNOWN,
        }
    }
}

pub(crate) enum CtxNevIdx {
    CNID_SKIP_FLAG = 0,
    CNID_PRED_MODE = 1,
    CNID_MODE_CONS = 2,
    CNID_AFFN_FLAG = 3,
    CNID_IBC_FLAG = 4,
    NUM_CNID = 5,
}

/*************************************************

*/
/*****************************************************************************
* macros for CU map

- [ 0: 6] : slice number (0 ~ 128)
- [ 7:14] : reserved
- [15:15] : 1 -> intra CU, 0 -> inter CU
- [16:22] : QP
- [23:23] : skip mode flag
- [24:24] : luma cbf
- [25:25] : dmvr_flag
- [26:26] : IBC mode flag
- [27:30] : reserved
- [31:31] : 0 -> no encoded/decoded CU, 1 -> encoded/decoded CU
*****************************************************************************/
#[derive(Default, Clone, Copy)]
pub(crate) struct MCU(u32);

impl MCU {
    /*
        /* set slice number to map */
        # define MCU_SET_SN(m, sn)       (m) = (((m) & 0xFFFFFF80) | ((sn) & 0x7F))
        /* get slice number from map */
        # define MCU_GET_SN(m)           (int)((m) & 0x7F)
    */
    /* set intra CU flag to map */
    #[inline]
    pub(crate) fn SET_IF(&mut self) {
        self.0 = self.0 | (1 << 15)
    }
    /* get intra CU flag from map */
    #[inline]
    pub(crate) fn GET_IF(&self) -> u32 {
        (self.0 >> 15) & 1
    }
    /* clear intra CU flag in map */
    #[inline]
    pub(crate) fn CLR_IF(&mut self) {
        self.0 = self.0 & 0xFFFF7FFF
    }
    /*
    /* set QP to map */
    # define MCU_SET_QP(m, qp)       (m) = ((m) | ((qp) & 0x7F) < < 16)
    /* get QP from map */
    # define MCU_GET_QP(m)           (int)(((m) > > 16) & 0x7F)
    # if DQP
    # define MCU_RESET_QP(m)         (m) = ((m) & (~((127) < < 16)))
    # endif

    /* set skip mode flag */
    # define MCU_SET_SF(m)           (m) = ((m) | (1 < < 23))
    /* get skip mode flag */
    # define MCU_GET_SF(m)           (int)(((m) > > 23) & 1)
    /* clear skip mode flag */
    # define MCU_CLR_SF(m)           (m) = ((m) & (~(1 < < 23)))

    /* set luma cbf flag */
    # define MCU_SET_CBFL(m)         (m) = ((m) | (1 < < 24))
    /* get luma cbf flag */
    # define MCU_GET_CBFL(m)         (int)(((m) > > 24) & 1)
    /* clear luma cbf flag */
    # define MCU_CLR_CBFL(m)         (m) = ((m) & (~(1 < < 24)))

    # if DMVR_FLAG
    /* set dmvr flag */
    # define MCU_SET_DMVRF(m)         (m) = ((m) | (1 < < 25))
    /* get dmvr flag */
    # define MCU_GET_DMVRF(m)         (int)(((m) > > 25) & 1)
    /* clear dmvr flag */
    # define MCU_CLR_DMVRF(m)         (m) = ((m) & (~(1 < < 25)))
    # endif
    /* set ibc mode flag */
    # define MCU_SET_IBC(m)          (m) = ((m) | (1 < < 26))
    /* get ibc mode flag */
    # define MCU_GET_IBC(m)          (int)(((m) > > 26) & 1)
    /* clear ibc mode flag */
    # define MCU_CLR_IBC(m)          (m) = ((m) & (~(1 < < 26)))
     */
    /* set encoded/decoded CU to map */
    #[inline]
    pub(crate) fn SET_COD(&mut self) {
        self.0 = self.0 | (1 << 31);
    }
    /* get encoded/decoded CU flag from map */
    #[inline]
    pub(crate) fn GET_COD(&self) -> u32 {
        (self.0 >> 31) & 1
    }
    /* clear encoded/decoded CU flag to map */
    #[inline]
    pub(crate) fn CLR_COD(&mut self) {
        self.0 = self.0 & 0x7FFFFFFF;
    }

    /*
    /* multi bit setting: intra flag, encoded/decoded flag, slice number */
    # define MCU_SET_IF_COD_SN_QP(m, i, sn, qp) \
    (m) = (((m) & 0xFF807F80) | ((sn) & 0x7F) | ((qp) < < 16) | ((i)< < 15) | (1 < < 31))

    # define MCU_IS_COD_NIF(m)      ((((m)> > 15) & 0x10001) == 0x10000)
     */
}
/*****************************************************************************
 * NALU header
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcNalu {
    pub(crate) nal_unit_size: u32,
    pub(crate) forbidden_zero_bit: u8,
    pub(crate) nal_unit_type: NaluType,
    pub(crate) nuh_temporal_id: u8,
    pub(crate) nuh_reserved_zero_5bits: u8,
    pub(crate) nuh_extension_flag: bool,
}

pub(crate) const EXTENDED_SAR: usize = 255;
pub(crate) const NUM_CPB: usize = 32;

/*****************************************************************************
* Hypothetical Reference Decoder (HRD) parameters, part of VUI
*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcHrd {
    pub(crate) cpb_cnt_minus1: u8,
    pub(crate) bit_rate_scale: u8,
    pub(crate) cpb_size_scale: u8,
    pub(crate) bit_rate_value_minus1: [u32; NUM_CPB],
    pub(crate) cpb_size_value_minus1: [u32; NUM_CPB],
    pub(crate) cbr_flag: [bool; NUM_CPB],
    pub(crate) initial_cpb_removal_delay_length_minus1: u8,
    pub(crate) cpb_removal_delay_length_minus1: u8,
    pub(crate) dpb_output_delay_length_minus1: u8,
    pub(crate) time_offset_length: u8,
}

/*****************************************************************************
* video usability information (VUI) part of SPS
*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcVui {
    pub(crate) aspect_ratio_info_present_flag: bool,
    pub(crate) aspect_ratio_idc: u8,
    pub(crate) sar_width: u16,
    pub(crate) sar_height: u16,
    pub(crate) overscan_info_present_flag: bool,
    pub(crate) overscan_appropriate_flag: bool,
    pub(crate) video_signal_type_present_flag: bool,
    pub(crate) video_format: u8,
    pub(crate) video_full_range_flag: bool,
    pub(crate) colour_description_present_flag: bool,
    pub(crate) colour_primaries: u8,
    pub(crate) transfer_characteristics: u8,
    pub(crate) matrix_coefficients: u8,
    pub(crate) chroma_loc_info_present_flag: bool,
    pub(crate) chroma_sample_loc_type_top_field: u8,
    pub(crate) chroma_sample_loc_type_bottom_field: u8,
    pub(crate) neutral_chroma_indication_flag: bool,
    pub(crate) field_seq_flag: bool,
    pub(crate) timing_info_present_flag: bool,
    pub(crate) num_units_in_tick: u32,
    pub(crate) time_scale: u32,
    pub(crate) fixed_pic_rate_flag: bool,
    pub(crate) nal_hrd_parameters_present_flag: bool,
    pub(crate) vcl_hrd_parameters_present_flag: bool,
    pub(crate) low_delay_hrd_flag: bool,
    pub(crate) pic_struct_present_flag: bool,
    pub(crate) bitstream_restriction_flag: bool,
    pub(crate) motion_vectors_over_pic_boundaries_flag: bool,
    pub(crate) max_bytes_per_pic_denom: u8,
    pub(crate) max_bits_per_mb_denom: u8,
    pub(crate) log2_max_mv_length_horizontal: u8,
    pub(crate) log2_max_mv_length_vertical: u8,
    pub(crate) num_reorder_pics: u8,
    pub(crate) max_dec_pic_buffering: u8,

    pub(crate) hrd_parameters: EvcHrd,
}

/*****************************************************************************
 * sequence parameter set
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcSps {
    pub(crate) sps_seq_parameter_set_id: u8,
    pub(crate) profile_idc: u8,
    pub(crate) level_idc: u8,
    pub(crate) toolset_idc_h: u32,
    pub(crate) toolset_idc_l: u32,
    pub(crate) chroma_format_idc: u8,
    pub(crate) pic_width_in_luma_samples: u16,
    pub(crate) pic_height_in_luma_samples: u16,
    pub(crate) bit_depth_luma_minus8: u8,
    pub(crate) bit_depth_chroma_minus8: u8,
    pub(crate) sps_btt_flag: bool,
    pub(crate) sps_suco_flag: bool,

    pub(crate) log2_ctu_size_minus5: u8,
    pub(crate) log2_min_cb_size_minus2: u8,
    pub(crate) log2_diff_ctu_max_14_cb_size: u8,
    pub(crate) log2_diff_ctu_max_tt_cb_size: u8,
    pub(crate) log2_diff_min_cb_min_tt_cb_size_minus2: u8,

    pub(crate) tool_amvr: bool,
    pub(crate) tool_mmvd: bool,
    pub(crate) tool_affine: bool,
    pub(crate) tool_dmvr: bool,
    pub(crate) tool_addb: bool,
    pub(crate) tool_alf: bool,
    pub(crate) tool_htdf: bool,
    pub(crate) tool_admvp: bool,
    pub(crate) tool_hmvp: bool,
    pub(crate) tool_eipd: bool,
    pub(crate) tool_iqt: bool,
    pub(crate) tool_cm_init: bool,
    pub(crate) tool_ats: bool,
    pub(crate) tool_rpl: bool,
    pub(crate) tool_pocs: bool,
    pub(crate) tool_adcc: bool,

    pub(crate) log2_sub_gop_length: u8,
    pub(crate) log2_ref_pic_gap_length: u8,
    pub(crate) max_num_ref_pics: u8,

    pub(crate) picture_cropping_flag: bool,
    pub(crate) picture_crop_left_offset: u16,
    pub(crate) picture_crop_right_offset: u16,
    pub(crate) picture_crop_top_offset: u16,
    pub(crate) picture_crop_bottom_offset: u16,

    pub(crate) dquant_flag: bool,
    pub(crate) chroma_qp_table_struct: EvcChromaTable,

    pub(crate) tool_dra: bool,

    pub(crate) vui_parameters_present_flag: bool,
    pub(crate) vui_parameters: EvcVui,
}

/*****************************************************************************
* picture parameter set
*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPps {
    pub(crate) pps_pic_parameter_set_id: u8,
    pub(crate) pps_seq_parameter_set_id: u8,
    pub(crate) num_ref_idx_default_active_minus1: [u8; 2],
    pub(crate) additional_lt_poc_lsb_len: u8,
    pub(crate) rpl1_idx_present_flag: bool,
    pub(crate) single_tile_in_pic_flag: bool,
    pub(crate) tile_id_len_minus1: u8,
    pub(crate) explicit_tile_id_flag: bool,
    pub(crate) pic_dra_enabled_flag: bool,
    pub(crate) arbitrary_slice_present_flag: bool,
    pub(crate) constrained_intra_pred_flag: bool,
    pub(crate) cu_qp_delta_enabled_flag: bool,
    pub(crate) cu_qp_delta_area: u8,
}

/*****************************************************************************
 * slice header
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcSh {
    pub(crate) slice_pic_parameter_set_id: u8,
    pub(crate) slice_type: SliceType,
    pub(crate) no_output_of_prior_pics_flag: bool,

    pub(crate) poc_lsb: i32,

    /*   HLS_RPL */
    pub(crate) ref_pic_list_sps_flag: [u32; 2],
    pub(crate) rpl_l0_idx: isize, //-1 means this slice does not use RPL candidate in SPS for RPL0
    pub(crate) rpl_l1_idx: isize, //-1 means this slice does not use RPL candidate in SPS for RPL1

    pub(crate) rpl_l0: EvcRpl,
    pub(crate) rpl_l1: EvcRpl,

    pub(crate) num_ref_idx_active_override_flag: bool,
    pub(crate) deblocking_filter_on: bool,

    pub(crate) qp: u8,
    pub(crate) qp_u: u8,
    pub(crate) qp_v: u8,
    pub(crate) qp_u_offset: i8,
    pub(crate) qp_v_offset: i8,

    pub(crate) num_ctb: u16,
}

/*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPoc {
    /* current picture order count value */
    pub(crate) poc_val: i32,
    /* the picture order count of the previous Tid0 picture */
    pub(crate) prev_poc_val: u32,
    /* the decoding order count of the previous picture */
    pub(crate) prev_doc_offset: i32,
}

/*****************************************************************************
 * for binary and triple tree structure
 *****************************************************************************/
#[derive(PartialEq, Clone, Copy)]
pub(crate) enum SplitMode {
    NO_SPLIT = 0,
    SPLIT_QUAD = 5,
}

impl SplitMode {
    #[inline]
    pub(crate) fn part_count(&self) -> usize {
        if self == &SplitMode::NO_SPLIT {
            0
        } else {
            4
        }
    }

    #[inline]
    pub(crate) fn part_size(&self, length: usize) -> usize {
        if self == &SplitMode::NO_SPLIT {
            length
        } else {
            length >> 1
        }
    }

    #[inline]
    pub(crate) fn part_size_idx(&self, length_idx: usize) -> usize {
        if self == &SplitMode::NO_SPLIT {
            length_idx
        } else {
            length_idx - 1
        }
    }

    /* Partitioning (START) */
    #[inline]
    pub(crate) fn inc_qt_depth(&self, qtd: u8) -> u8 {
        if self == &SplitMode::NO_SPLIT {
            qtd
        } else {
            qtd + 1
        }
    }
}

pub(crate) enum SplitDir {
    SPLIT_VER = 0,
    SPLIT_HOR = 1,
}

pub(crate) enum BlockShape {
    NON_SQUARE_14 = 0,
    NON_SQUARE_12 = 1,
    SQUARE = 2,
    NON_SQUARE_21 = 3,
    NON_SQUARE_41 = 4,
    NUM_BLOCK_SHAPE = 5,
}

pub(crate) enum ModeCons {
    eOnlyIntra,
    eOnlyInter,
    eAll,
}

/* MMVD (START) */
pub(crate) const MMVD_BASE_MV_NUM: usize = 4;
pub(crate) const MMVD_DIST_NUM: usize = 8;
pub(crate) const MMVD_MAX_REFINE_NUM: usize = (MMVD_DIST_NUM * 4);
pub(crate) const MMVD_SKIP_CON_NUM: usize = 4;
pub(crate) const MMVD_GRP_NUM: usize = 3;
pub(crate) const MMVD_THRESHOLD: f32 = 1.5;
/* MMVD (END) */

pub(crate) const AFF_MAX_NUM_MVP: usize = 2; // maximum affine inter candidates
pub(crate) const AFF_MAX_CAND: usize = 5; // maximum affine merge candidates

pub(crate) type SBAC_CTX_MODEL = u16;

/* CABAC (START) */
pub(crate) const PROB_INIT: SBAC_CTX_MODEL = (512); /* 1/2 of initialization with mps = 0 */
/* CABAC (END) */

pub(crate) const NUM_CTX_MMVD_FLAG: usize = 1;
pub(crate) const NUM_CTX_MMVD_GROUP_IDX: usize = (MMVD_GRP_NUM - 1);
pub(crate) const NUM_CTX_MMVD_MERGE_IDX: usize = (MMVD_BASE_MV_NUM - 1);
pub(crate) const NUM_CTX_MMVD_DIST_IDX: usize = (MMVD_DIST_NUM - 1);
pub(crate) const NUM_CTX_MMVD_DIRECTION_IDX: usize = 2;
pub(crate) const NUM_CTX_AFFINE_MVD_FLAG: usize = 2; /* number of context models for affine_mvd_flag_l0 and affine_mvd_flag_l1 (1st one is for affine_mvd_flag_l0 and 2nd one if for affine_mvd_flag_l1) */
pub(crate) const NUM_CTX_SKIP_FLAG: usize = 2;
pub(crate) const NUM_CTX_IBC_FLAG: usize = 2;
pub(crate) const NUM_CTX_BTT_SPLIT_FLAG: usize = 15;
pub(crate) const NUM_CTX_BTT_SPLIT_DIR: usize = 5;
pub(crate) const NUM_CTX_BTT_SPLIT_TYPE: usize = 1;
pub(crate) const NUM_CTX_SUCO_FLAG: usize = 14;
pub(crate) const NUM_CTX_CBF_LUMA: usize = 1;
pub(crate) const NUM_CTX_CBF_CB: usize = 1;
pub(crate) const NUM_CTX_CBF_CR: usize = 1;
pub(crate) const NUM_CTX_CBF_ALL: usize = 1;
pub(crate) const NUM_CTX_PRED_MODE: usize = 3;
pub(crate) const NUM_CTX_MODE_CONS: usize = 3;
pub(crate) const NUM_CTX_INTER_PRED_IDC: usize = 2; /* number of context models for inter prediction direction */
pub(crate) const NUM_CTX_DIRECT_MODE_FLAG: usize = 1;
pub(crate) const NUM_CTX_MERGE_MODE_FLAG: usize = 1;
pub(crate) const NUM_CTX_REF_IDX: usize = 2;
pub(crate) const NUM_CTX_MERGE_IDX: usize = 5;
pub(crate) const NUM_CTX_MVP_IDX: usize = 3;
pub(crate) const NUM_CTX_AMVR_IDX: usize = 4;
pub(crate) const NUM_CTX_BI_PRED_IDX: usize = 2;
pub(crate) const NUM_CTX_MVD: usize = 1; /* number of context models for motion vector difference */
pub(crate) const NUM_CTX_INTRA_PRED_MODE: usize = 2;
pub(crate) const NUM_CTX_INTRA_LUMA_PRED_MPM_FLAG: usize = 1;
pub(crate) const NUM_CTX_INTRA_LUMA_PRED_MPM_IDX: usize = 1;
pub(crate) const NUM_CTX_INTRA_CHROMA_PRED_MODE: usize = 1;
pub(crate) const NUM_CTX_AFFINE_FLAG: usize = 2;
pub(crate) const NUM_CTX_AFFINE_MODE: usize = 1;
pub(crate) const NUM_CTX_AFFINE_MRG: usize = AFF_MAX_CAND;
pub(crate) const NUM_CTX_AFFINE_MVP_IDX: usize = (AFF_MAX_NUM_MVP - 1);
pub(crate) const NUM_CTX_CC_RUN: usize = 24;
pub(crate) const NUM_CTX_CC_LAST: usize = 2;
pub(crate) const NUM_CTX_CC_LEVEL: usize = 24;
pub(crate) const NUM_CTX_ALF_CTB_FLAG: usize = 1;
pub(crate) const NUM_CTX_SPLIT_CU_FLAG: usize = 1;
pub(crate) const NUM_CTX_DELTA_QP: usize = 1;
pub(crate) const NUM_CTX_ATS_INTRA_CU_FLAG: usize = 1;
pub(crate) const NUM_CTX_ATS_MODE_FLAG: usize = 1;
pub(crate) const NUM_CTX_ATS_INTER_FLAG: usize = 2;
pub(crate) const NUM_CTX_ATS_INTER_QUAD_FLAG: usize = 1;
pub(crate) const NUM_CTX_ATS_INTER_HOR_FLAG: usize = 3;
pub(crate) const NUM_CTX_ATS_INTER_POS_FLAG: usize = 1;

pub(crate) const NUM_CTX_LAST_SIG_COEFF_LUMA: usize = 18;
pub(crate) const NUM_CTX_LAST_SIG_COEFF_CHROMA: usize = 3;
pub(crate) const NUM_CTX_LAST_SIG_COEFF: usize =
    (NUM_CTX_LAST_SIG_COEFF_LUMA + NUM_CTX_LAST_SIG_COEFF_CHROMA);
pub(crate) const NUM_CTX_SIG_COEFF_LUMA: usize = 39; /* number of context models for luma sig coeff flag */
pub(crate) const NUM_CTX_SIG_COEFF_CHROMA: usize = 8; /* number of context models for chroma sig coeff flag */
pub(crate) const NUM_CTX_SIG_COEFF_LUMA_TU: usize = 13; /* number of context models for luma sig coeff flag per TU */
pub(crate) const NUM_CTX_SIG_COEFF_FLAG: usize =
    (NUM_CTX_SIG_COEFF_LUMA + NUM_CTX_SIG_COEFF_CHROMA); /* number of context models for sig coeff flag */
pub(crate) const NUM_CTX_GTX_LUMA: usize = 13;
pub(crate) const NUM_CTX_GTX_CHROMA: usize = 5;
pub(crate) const NUM_CTX_GTX: usize = (NUM_CTX_GTX_LUMA + NUM_CTX_GTX_CHROMA); /* number of context models for gtA/B flag */

/* context models for arithemetic coding */
#[derive(Default)]
pub(crate) struct EvcSbacCtx {
    pub(crate) skip_flag: [SBAC_CTX_MODEL; NUM_CTX_SKIP_FLAG],
    pub(crate) ibc_flag: [SBAC_CTX_MODEL; NUM_CTX_IBC_FLAG],
    pub(crate) mmvd_flag: [SBAC_CTX_MODEL; NUM_CTX_MMVD_FLAG],
    pub(crate) mmvd_merge_idx: [SBAC_CTX_MODEL; NUM_CTX_MMVD_MERGE_IDX],
    pub(crate) mmvd_distance_idx: [SBAC_CTX_MODEL; NUM_CTX_MMVD_DIST_IDX],
    pub(crate) mmvd_direction_idx: [SBAC_CTX_MODEL; NUM_CTX_MMVD_DIRECTION_IDX],
    pub(crate) mmvd_group_idx: [SBAC_CTX_MODEL; NUM_CTX_MMVD_GROUP_IDX],
    pub(crate) direct_mode_flag: [SBAC_CTX_MODEL; NUM_CTX_DIRECT_MODE_FLAG],
    pub(crate) merge_mode_flag: [SBAC_CTX_MODEL; NUM_CTX_MERGE_MODE_FLAG],
    pub(crate) inter_dir: [SBAC_CTX_MODEL; NUM_CTX_INTER_PRED_IDC],
    pub(crate) intra_dir: [SBAC_CTX_MODEL; NUM_CTX_INTRA_PRED_MODE],
    pub(crate) intra_luma_pred_mpm_flag: [SBAC_CTX_MODEL; NUM_CTX_INTRA_LUMA_PRED_MPM_FLAG],
    pub(crate) intra_luma_pred_mpm_idx: [SBAC_CTX_MODEL; NUM_CTX_INTRA_LUMA_PRED_MPM_IDX],
    pub(crate) intra_chroma_pred_mode: [SBAC_CTX_MODEL; NUM_CTX_INTRA_CHROMA_PRED_MODE],
    pub(crate) pred_mode: [SBAC_CTX_MODEL; NUM_CTX_PRED_MODE],
    pub(crate) mode_cons: [SBAC_CTX_MODEL; NUM_CTX_MODE_CONS],
    pub(crate) refi: [SBAC_CTX_MODEL; NUM_CTX_REF_IDX],
    pub(crate) merge_idx: [SBAC_CTX_MODEL; NUM_CTX_MERGE_IDX],
    pub(crate) mvp_idx: [SBAC_CTX_MODEL; NUM_CTX_MVP_IDX],
    pub(crate) affine_mvp_idx: [SBAC_CTX_MODEL; NUM_CTX_AFFINE_MVP_IDX],
    pub(crate) mvr_idx: [SBAC_CTX_MODEL; NUM_CTX_AMVR_IDX],
    pub(crate) bi_idx: [SBAC_CTX_MODEL; NUM_CTX_BI_PRED_IDX],
    pub(crate) mvd: [SBAC_CTX_MODEL; NUM_CTX_MVD],
    pub(crate) cbf_all: [SBAC_CTX_MODEL; NUM_CTX_CBF_ALL],
    pub(crate) cbf_luma: [SBAC_CTX_MODEL; NUM_CTX_CBF_LUMA],
    pub(crate) cbf_cb: [SBAC_CTX_MODEL; NUM_CTX_CBF_CB],
    pub(crate) cbf_cr: [SBAC_CTX_MODEL; NUM_CTX_CBF_CR],
    pub(crate) run: [SBAC_CTX_MODEL; NUM_CTX_CC_RUN],
    pub(crate) last: [SBAC_CTX_MODEL; NUM_CTX_CC_LAST],
    pub(crate) level: [SBAC_CTX_MODEL; NUM_CTX_CC_LEVEL],
    //pub(crate) sig_coeff_flag: [SBAC_CTX_MODEL; NUM_CTX_SIG_COEFF_FLAG],
    pub(crate) coeff_abs_level_greaterAB_flag: [SBAC_CTX_MODEL; NUM_CTX_GTX],
    pub(crate) last_sig_coeff_x_prefix: [SBAC_CTX_MODEL; NUM_CTX_LAST_SIG_COEFF],
    pub(crate) last_sig_coeff_y_prefix: [SBAC_CTX_MODEL; NUM_CTX_LAST_SIG_COEFF],
    pub(crate) btt_split_flag: [SBAC_CTX_MODEL; NUM_CTX_BTT_SPLIT_FLAG],
    pub(crate) btt_split_dir: [SBAC_CTX_MODEL; NUM_CTX_BTT_SPLIT_DIR],
    pub(crate) btt_split_type: [SBAC_CTX_MODEL; NUM_CTX_BTT_SPLIT_TYPE],
    pub(crate) affine_flag: [SBAC_CTX_MODEL; NUM_CTX_AFFINE_FLAG],
    pub(crate) affine_mode: [SBAC_CTX_MODEL; NUM_CTX_AFFINE_MODE],
    pub(crate) affine_mrg: [SBAC_CTX_MODEL; NUM_CTX_AFFINE_MRG],
    pub(crate) affine_mvd_flag: [SBAC_CTX_MODEL; NUM_CTX_AFFINE_MVD_FLAG],
    pub(crate) suco_flag: [SBAC_CTX_MODEL; NUM_CTX_SUCO_FLAG],
    pub(crate) alf_ctb_flag: [SBAC_CTX_MODEL; NUM_CTX_ALF_CTB_FLAG],
    pub(crate) split_cu_flag: [SBAC_CTX_MODEL; NUM_CTX_SPLIT_CU_FLAG],
    pub(crate) delta_qp: [SBAC_CTX_MODEL; NUM_CTX_DELTA_QP],
    pub(crate) ats_mode: [SBAC_CTX_MODEL; NUM_CTX_ATS_MODE_FLAG],
    pub(crate) ats_cu_inter_flag: [SBAC_CTX_MODEL; NUM_CTX_ATS_INTER_FLAG],
    pub(crate) ats_cu_inter_quad_flag: [SBAC_CTX_MODEL; NUM_CTX_ATS_INTER_QUAD_FLAG],
    pub(crate) ats_cu_inter_hor_flag: [SBAC_CTX_MODEL; NUM_CTX_ATS_INTER_HOR_FLAG],
    pub(crate) ats_cu_inter_pos_flag: [SBAC_CTX_MODEL; NUM_CTX_ATS_INTER_POS_FLAG],
}

pub(crate) const MAX_SUB_TB_NUM: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TREE_TYPE {
    TREE_LC = 0,
    TREE_L = 1,
    TREE_C = 2,
}

impl Default for TREE_TYPE {
    fn default() -> Self {
        TREE_TYPE::TREE_LC
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MODE_CONS {
    eOnlyIntra,
    eOnlyInter,
    eAll,
}

impl Default for MODE_CONS {
    fn default() -> Self {
        MODE_CONS::eOnlyIntra
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct TREE_CONS {
    pub(crate) changed: bool,
    pub(crate) tree_type: TREE_TYPE,
    pub(crate) mode_cons: MODE_CONS,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct TREE_CONS_NEW {
    pub(crate) tree_type: TREE_TYPE,
    pub(crate) mode_cons: MODE_CONS,
}
