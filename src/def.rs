use crate::api::*;

/*****************************************************************************
 * types
 *****************************************************************************/

#[inline]
pub(crate) fn evc_assert_rv(x: bool, r: EvcError) -> Result<(), EvcError> {
    if !x {
        assert!(x);
        return Err(r);
    }
    Ok(())
}

/********* Conditional tools definition ********/
pub type pel = u16;

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

pub(crate) const MAX_CU_LOG2: usize = 6; // baseline: 64x64
pub(crate) const MIN_CU_LOG2: usize = 2;
pub(crate) const MAX_CU_SIZE: usize = (1 << MAX_CU_LOG2);
pub(crate) const MIN_CU_SIZE: usize = (1 << MIN_CU_LOG2);
pub(crate) const MAX_CU_DIM: usize = (1 << (MAX_CU_LOG2 + MAX_CU_LOG2));
pub(crate) const MIN_CU_DIM: usize = (1 << (MIN_CU_LOG2 + MIN_CU_LOG2));
pub(crate) const MAX_CU_DEPTH: usize = 9; /* 64x64 ~ 4x4 */
pub(crate) const NUM_CU_DEPTH: usize = (MAX_CU_DEPTH + 1);

pub(crate) const MAX_TR_LOG2: usize = 6; /* 64x64 */
pub(crate) const MIN_TR_LOG2: usize = 1; /* 2x2 */
pub(crate) const MAX_TR_SIZE: usize = (1 << MAX_TR_LOG2);
pub(crate) const MIN_TR_SIZE: usize = (1 << MIN_TR_LOG2);
pub(crate) const MAX_TR_DIM: usize = (1 << (MAX_TR_LOG2 + MAX_TR_LOG2));
pub(crate) const MIN_TR_DIM: usize = (1 << (MIN_TR_LOG2 + MIN_TR_LOG2));

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
pub(crate) const DRA_FRAME: usize = 1;
pub(crate) const MAX_PB_SIZE: usize = MAX_NUM_REF_PICS + EXTRA_FRAME + DRA_FRAME;

pub(crate) const MAX_NUM_TILES_ROW: usize = 22;
pub(crate) const MAX_NUM_TILES_COL: usize = 20;

/* Neighboring block availability flag bits */
pub(crate) const AVAIL_BIT_UP: u16 = 0;
pub(crate) const AVAIL_BIT_LE: u16 = 1;
pub(crate) const AVAIL_BIT_RI: u16 = 3;
pub(crate) const AVAIL_BIT_LO: u16 = 4;
pub(crate) const AVAIL_BIT_UP_LE: u16 = 5;
pub(crate) const AVAIL_BIT_UP_RI: u16 = 6;
pub(crate) const AVAIL_BIT_LO_LE: u16 = 7;
pub(crate) const AVAIL_BIT_LO_RI: u16 = 8;
pub(crate) const AVAIL_BIT_RI_UP: u16 = 9;
pub(crate) const AVAIL_BIT_UP_LE_LE: u16 = 10;
pub(crate) const AVAIL_BIT_UP_RI_RI: u16 = 11;

/* Neighboring block availability flags */
pub(crate) const AVAIL_UP: u16 = (1 << AVAIL_BIT_UP);
pub(crate) const AVAIL_LE: u16 = (1 << AVAIL_BIT_LE);
pub(crate) const AVAIL_RI: u16 = (1 << AVAIL_BIT_RI);
pub(crate) const AVAIL_LO: u16 = (1 << AVAIL_BIT_LO);
pub(crate) const AVAIL_UP_LE: u16 = (1 << AVAIL_BIT_UP_LE);
pub(crate) const AVAIL_UP_RI: u16 = (1 << AVAIL_BIT_UP_RI);
pub(crate) const AVAIL_LO_LE: u16 = (1 << AVAIL_BIT_LO_LE);
pub(crate) const AVAIL_LO_RI: u16 = (1 << AVAIL_BIT_LO_RI);
pub(crate) const AVAIL_RI_UP: u16 = (1 << AVAIL_BIT_RI_UP);
pub(crate) const AVAIL_UP_LE_LE: u16 = (1 << AVAIL_BIT_UP_LE_LE);
pub(crate) const AVAIL_UP_RI_RI: u16 = (1 << AVAIL_BIT_UP_RI_RI);

pub(crate) const LR_00: u16 = 0;
pub(crate) const LR_10: u16 = 1;
pub(crate) const LR_01: u16 = 2;
pub(crate) const LR_11: u16 = 3;

/* MB availability check macro */
#[inline]
pub(crate) fn IS_AVAIL(avail: u16, pos: u16) -> bool {
    (avail & pos) == pos
}
/* MB availability set macro */
#[inline]
pub(crate) fn SET_AVAIL(avail: &mut u16, pos: u16) {
    *avail |= pos;
}
/* MB availability remove macro */
#[inline]
pub(crate) fn REM_AVAIL(avail: &mut u16, pos: u16) {
    *avail &= !pos
}
/* MB availability into bit flag */
#[inline]
pub(crate) fn GET_AVAIL_FLAG(avail: u16, bit: u16) -> bool {
    (avail >> bit) & 0x1 != 0
}

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
pub(crate) enum InterPredDir {
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
    PRED_NUM = 5,
}

impl Default for InterPredDir {
    fn default() -> Self {
        InterPredDir::PRED_L0
    }
}

/*****************************************************************************
 * intra prediction direction
 *****************************************************************************/
pub(crate) const IPD_RDO_CNT: usize = 5;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum IntraPredDir {
    IPD_INVALID = -1,
    IPD_DC_B = 0,
    IPD_HOR_B = 1, /* Luma, Horizontal */
    IPD_VER_B = 2, /* Luma, Vertical */
    IPD_UL_B = 3,
    IPD_UR_B = 4,
    IPD_CNT_B = 5,
}

impl Default for IntraPredDir {
    fn default() -> Self {
        IntraPredDir::IPD_INVALID
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
            _ => IPD_INVALID,
        }
    }
}

/*****************************************************************************
* macros for CU map

- [ 0: 6] : SN: slice number (0 ~ 128)
- [ 7:14] : reserved
- [15:15] : IF: 1 -> intra CU, 0 -> inter CU
- [16:22] : QP
- [23:23] : SF: skip mode flag
- [24:24] : CBFL: luma cbf
- [25:25] : reserved
- [26:26] : reserved
- [27:30] : reserved
- [31:31] : COD: 0 -> no encoded/decoded CU, 1 -> encoded/decoded CU
*****************************************************************************/
#[derive(Default, Clone, Copy)]
pub(crate) struct MCU(u32);

impl From<u32> for MCU {
    fn from(val: u32) -> Self {
        MCU(val)
    }
}

impl MCU {
    /* set slice number to map */
    #[inline]
    pub(crate) fn SET_SN(&mut self, sn: u32) {
        self.0 = (self.0 & 0xFFFFFF80) | (sn & 0x7F);
    }
    /* get slice number from map */
    #[inline]
    pub(crate) fn GET_SN(&self) -> u32 {
        self.0 & 0x7F
    }

    /* set intra CU flag to map */
    #[inline]
    pub(crate) fn SET_IF(&mut self) {
        self.0 = self.0 | (1 << 15);
    }
    /* get intra CU flag from map */
    #[inline]
    pub(crate) fn GET_IF(&self) -> u32 {
        (self.0 >> 15) & 1
    }
    /* clear intra CU flag in map */
    #[inline]
    pub(crate) fn CLR_IF(&mut self) {
        self.0 = self.0 & 0xFFFF7FFF;
    }

    /* set QP to map */
    #[inline]
    pub(crate) fn SET_QP(&mut self, qp: u32) {
        self.0 = self.0 | ((qp & 0x7F) << 16);
    }
    /* get QP from map */
    #[inline]
    pub(crate) fn GET_QP(&self) -> u32 {
        (self.0 >> 16) & 0x7F
    }

    #[inline]
    pub(crate) fn RESET_QP(&mut self) {
        self.0 = self.0 & (!(127 << 16));
    }

    /* set skip mode flag */
    #[inline]
    pub(crate) fn SET_SF(&mut self) {
        self.0 = self.0 | (1 << 23);
    }
    /* get skip mode flag */
    #[inline]
    pub(crate) fn GET_SF(&self) -> u32 {
        (self.0 >> 23) & 1
    }
    /* clear skip mode flag */
    #[inline]
    pub(crate) fn CLR_SF(&mut self) {
        self.0 = self.0 & (!(1 << 23));
    }

    /* set luma cbf flag */
    #[inline]
    pub(crate) fn SET_CBFL(&mut self) {
        self.0 = self.0 | (1 << 24);
    }
    /* get luma cbf flag */
    #[inline]
    pub(crate) fn GET_CBFL(&self) -> u32 {
        (self.0 >> 24) & 1
    }
    /* clear luma cbf flag */
    #[inline]
    pub(crate) fn CLR_CBFL(&mut self) {
        self.0 = self.0 & (!(1 << 24))
    }

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

    /* multi bit setting: intra flag, encoded/decoded flag, slice number */
    #[inline]
    pub(crate) fn SET_IF_COD_SN_QP(&mut self, i: u32, sn: u32, qp: u8) {
        self.0 =
            (self.0 & 0xFF807F80) | ((sn) & 0x7F) | ((qp as u32) << 16) | ((i) << 15) | (1 << 31);
    }
    #[inline]
    pub(crate) fn IS_COD_NIF(&self) -> bool {
        ((self.0 >> 15) & 0x10001) == 0x10000
    }
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

impl EvcNalu {
    pub(crate) fn set_nalu(&mut self, nalu_type: NaluType, nuh_temporal_id: u8) {
        self.nal_unit_size = 0;
        self.forbidden_zero_bit = 0;
        self.nal_unit_type = nalu_type;
        self.nuh_temporal_id = nuh_temporal_id;
        self.nuh_reserved_zero_5bits = 0;
        self.nuh_extension_flag = false;
    }
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

    /*QP of previous cu in decoding order (used for dqp)*/
    pub(crate) qp_prev_eco: u8,
    pub(crate) dqp: i8,
    pub(crate) qp_prev_mode: u8,

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
    SPLIT_QUAD = 1,
}
pub(crate) const MAX_SPLIT_NUM: usize = 2;

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

pub(crate) type SBAC_CTX_MODEL = u16;

/* CABAC (START) */
pub(crate) const PROB_INIT: SBAC_CTX_MODEL = (512); /* 1/2 of initialization with mps = 0 */
/* CABAC (END) */

/* Multiple Referene (START) */
pub(crate) const MAX_NUM_ACTIVE_REF_FRAME_B: u8 = 2; /* Maximum number of active reference frames for RA condition */
pub(crate) const MAX_NUM_ACTIVE_REF_FRAME_LDB: u8 = 4; /* Maximum number of active reference frames for LDB condition */
/* Multiple Reference (END) */

pub(crate) const NUM_CTX_SKIP_FLAG: usize = 2;
pub(crate) const NUM_CTX_CBF_LUMA: usize = 1;
pub(crate) const NUM_CTX_CBF_CB: usize = 1;
pub(crate) const NUM_CTX_CBF_CR: usize = 1;
pub(crate) const NUM_CTX_CBF_ALL: usize = 1;
pub(crate) const NUM_CTX_PRED_MODE: usize = 3;
pub(crate) const NUM_CTX_INTER_PRED_IDC: usize = 2;
pub(crate) const NUM_CTX_DIRECT_MODE_FLAG: usize = 1;
pub(crate) const NUM_CTX_REF_IDX: usize = 2;
pub(crate) const NUM_CTX_MVP_IDX: usize = 3;
pub(crate) const NUM_CTX_MVD: usize = 1;
pub(crate) const NUM_CTX_INTRA_PRED_MODE: usize = 2;
pub(crate) const NUM_CTX_CC_RUN: usize = 24;
pub(crate) const NUM_CTX_CC_LAST: usize = 2;
pub(crate) const NUM_CTX_CC_LEVEL: usize = 24;
pub(crate) const NUM_CTX_SPLIT_CU_FLAG: usize = 1;
pub(crate) const NUM_CTX_DELTA_QP: usize = 1;

/* context models for arithemetic coding */
#[derive(Default, Copy, Clone)]
pub(crate) struct EvcSbacCtx {
    pub(crate) skip_flag: [SBAC_CTX_MODEL; NUM_CTX_SKIP_FLAG],
    pub(crate) cbf_luma: [SBAC_CTX_MODEL; NUM_CTX_CBF_LUMA],
    pub(crate) cbf_cb: [SBAC_CTX_MODEL; NUM_CTX_CBF_CB],
    pub(crate) cbf_cr: [SBAC_CTX_MODEL; NUM_CTX_CBF_CR],
    pub(crate) cbf_all: [SBAC_CTX_MODEL; NUM_CTX_CBF_ALL],
    pub(crate) pred_mode: [SBAC_CTX_MODEL; NUM_CTX_PRED_MODE],
    pub(crate) inter_dir: [SBAC_CTX_MODEL; NUM_CTX_INTER_PRED_IDC],
    pub(crate) direct_mode_flag: [SBAC_CTX_MODEL; NUM_CTX_DIRECT_MODE_FLAG],
    pub(crate) refi: [SBAC_CTX_MODEL; NUM_CTX_REF_IDX],
    pub(crate) mvp_idx: [SBAC_CTX_MODEL; NUM_CTX_MVP_IDX],
    pub(crate) mvd: [SBAC_CTX_MODEL; NUM_CTX_MVD],
    pub(crate) intra_dir: [SBAC_CTX_MODEL; NUM_CTX_INTRA_PRED_MODE],
    pub(crate) run: [SBAC_CTX_MODEL; NUM_CTX_CC_RUN],
    pub(crate) last: [SBAC_CTX_MODEL; NUM_CTX_CC_LAST],
    pub(crate) level: [SBAC_CTX_MODEL; NUM_CTX_CC_LEVEL],
    pub(crate) split_cu_flag: [SBAC_CTX_MODEL; NUM_CTX_SPLIT_CU_FLAG],
    pub(crate) delta_qp: [SBAC_CTX_MODEL; NUM_CTX_DELTA_QP],
}

pub(crate) const QUANT_SHIFT: usize = 14;
pub(crate) const QUANT_IQUANT_SHIFT: usize = 20;

#[derive(Clone)]
pub(crate) struct LcuSplitMode {
    pub(crate) data: Vec<Vec<Vec<SplitMode>>>,
}

impl Default for LcuSplitMode {
    fn default() -> Self {
        LcuSplitMode {
            data: vec![
                vec![
                    vec![SplitMode::NO_SPLIT; MAX_CU_CNT_IN_LCU];
                    BlockShape::NUM_BLOCK_SHAPE as usize
                ];
                NUM_CU_DEPTH
            ],
        }
    }
}

#[derive(Clone)]
pub(crate) struct CUBuffer<T: Default + Copy> {
    pub(crate) data: Vec<Vec<T>>,
}

impl<T: Default + Copy> Default for CUBuffer<T> {
    fn default() -> Self {
        CUBuffer {
            data: vec![
                vec![T::default(); MAX_CU_DIM],
                vec![T::default(); MAX_CU_DIM >> 2],
                vec![T::default(); MAX_CU_DIM >> 2],
            ],
        }
    }
}

#[derive(Clone)]
pub(crate) struct NBBuffer<T: Default + Copy> {
    pub(crate) data: Vec<Vec<T>>,
}

impl<T: Default + Copy> Default for NBBuffer<T> {
    fn default() -> Self {
        NBBuffer {
            data: vec![
                vec![T::default(); (MAX_CU_SIZE << 2) + 1], //left*2 + top_left + top*2
                vec![T::default(); (MAX_CU_SIZE << 1) + 1],
                vec![T::default(); (MAX_CU_SIZE << 1) + 1],
            ],
        }
    }
}
