pub(crate) mod context;
pub(crate) mod plane;
pub(crate) mod plane_region;
pub(crate) mod tbl;

use crate::api::*;

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
//pub(crate) const PEL2SCU(pel)      :usize=                 ((pel) >> MIN_CU_LOG2);

pub(crate) const PIC_PAD_SIZE_L: usize = (MAX_CU_SIZE + 16);
pub(crate) const PIC_PAD_SIZE_C: usize = (PIC_PAD_SIZE_L >> 1);

/* number of MVP candidates */
pub(crate) const MAX_NUM_MVP: usize = 4;

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

/*****************************************************************************
 * NALU header
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcNalu {
    pub(crate) nal_unit_size: u32,
    pub(crate) forbidden_zero_bit: u8,
    pub(crate) nal_unit_type_plus1: u8,
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
    cpb_cnt_minus1: u8,
    bit_rate_scale: u8,
    cpb_size_scale: u8,
    bit_rate_value_minus1: [u32; NUM_CPB],
    cpb_size_value_minus1: [u32; NUM_CPB],
    cbr_flag: [bool; NUM_CPB],
    initial_cpb_removal_delay_length_minus1: u8,
    cpb_removal_delay_length_minus1: u8,
    dpb_output_delay_length_minus1: u8,
    time_offset_length: u8,
}

/*****************************************************************************
* video usability information (VUI) part of SPS
*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcVui {
    aspect_ratio_info_present_flag: bool,
    aspect_ratio_idc: u8,
    sar_width: u16,
    sar_height: u16,
    overscan_info_present_flag: bool,
    overscan_appropriate_flag: bool,
    video_signal_type_present_flag: bool,
    video_format: u8,
    video_full_range_flag: bool,
    colour_description_present_flag: bool,
    colour_primaries: u8,
    transfer_characteristics: u8,
    matrix_coefficients: u8,
    chroma_loc_info_present_flag: bool,
    chroma_sample_loc_type_top_field: u8,
    chroma_sample_loc_type_bottom_field: u8,
    neutral_chroma_indication_flag: bool,
    field_seq_flag: bool,
    timing_info_present_flag: bool,
    num_units_in_tick: u32,
    time_scale: u32,
    fixed_pic_rate_flag: bool,
    nal_hrd_parameters_present_flag: bool,
    vcl_hrd_parameters_present_flag: bool,
    low_delay_hrd_flag: bool,
    pic_struct_present_flag: bool,
    bitstream_restriction_flag: bool,
    motion_vectors_over_pic_boundaries_flag: bool,
    max_bytes_per_pic_denom: u8,
    max_bits_per_mb_denom: u8,
    log2_max_mv_length_horizontal: u8,
    log2_max_mv_length_vertical: u8,
    num_reorder_pics: u8,
    max_dec_pic_buffering: u8,

    hrd_parameters: EvcHrd,
}

/*****************************************************************************
 * sequence parameter set
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcSps {
    sps_seq_parameter_set_id: u8,
    profile_idc: u8,
    level_idc: u8,
    toolset_idc_h: u32,
    toolset_idc_l: u32,
    chroma_format_idc: u8,
    pic_width_in_luma_samples: u16,
    pic_height_in_luma_samples: u16,
    bit_depth_luma_minus8: u8,
    bit_depth_chroma_minus8: u8,

    log2_ctu_size_minus5: u8,
    log2_min_cb_size_minus2: u8,
    log2_diff_ctu_max_14_cb_size: u8,
    log2_diff_ctu_max_tt_cb_size: u8,
    log2_diff_min_cb_min_tt_cb_size_minus2: u8,

    log2_sub_gop_length: u8,
    log2_ref_pic_gap_length: u8,

    picture_cropping_flag: bool,
    picture_crop_left_offset: u16,
    picture_crop_right_offset: u16,
    picture_crop_top_offset: u16,
    picture_crop_bottom_offset: u16,

    chroma_qp_table_struct: EvcChromaTable,

    vui_parameters_present_flag: bool,
    vui_parameters: EvcVui,
}

/*****************************************************************************
* picture parameter set
*****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPps {
    pps_pic_parameter_set_id: u8,
    pps_seq_parameter_set_id: u8,
    num_ref_idx_default_active_minus1: [u8; 2],
    additional_lt_poc_lsb_len: u8,
    rpl1_idx_present_flag: bool,
    tile_id_len_minus1: u8,
    explicit_tile_id_flag: bool,
    arbitrary_slice_present_flag: bool,
    constrained_intra_pred_flag: bool,
    cu_qp_delta_enabled_flag: bool,
    cu_qp_delta_area: u8,
}

/*****************************************************************************
 * slice header
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcSh {
    slice_pic_parameter_set_id: u8,
    slice_type: u8,
    no_output_of_prior_pics_flag: bool,

    rpl_l0: EvcRpl,
    rpl_l1: EvcRpl,

    num_ref_idx_active_override_flag: bool,
    deblocking_filter_on: bool,

    qp: u8,
    qp_u: u8,
    qp_v: u8,
    qp_u_offset: i8,
    qp_v_offset: i8,
}