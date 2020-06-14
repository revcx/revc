use super::api::*;
use super::com::*;

mod bsr;
mod eco;

use crate::dec::eco::evcd_eco_nalu;
use bsr::EvcdBsr;

/* evc decoder magic code */
pub(crate) const EVCD_MAGIC_CODE: u32 = 0x45565944; /* EVYD */

/*****************************************************************************
 * SBAC structure
 *****************************************************************************/
pub(crate) struct EvcdSbac {
    pub(crate) range: u32,
    pub(crate) value: u32,
    //    pub(crate) ctx: EvcSbacCtx,
}

/*****************************************************************************
 * CORE information used for decoding process.
 *
 * The variables in this structure are very often used in decoding process.
 *****************************************************************************/

pub(crate) struct EvcdCore {
    /*
/************** current CU **************/
/* coefficient buffer of current CU */
            coef: [[s16;MAX_CU_DIM]; N_C], //[N_C][MAX_CU_DIM]
/* pred buffer of current CU */
/* [1] is used for bi-pred. */
            pred: [[[pel;MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]
            dmvr_template:[pel; MAX_CU_DIM], //[MAX_CU_DIM]
pel            dmvr_half_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + 1) * (MAX_CU_SIZE + 1)];
pel            dmvr_ref_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT)) * (MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT))];

#if DMVR_PADDING
pel  dmvr_padding_buf[2][N_C][PAD_BUFFER_STRIDE * PAD_BUFFER_STRIDE];
#endif
/* neighbor pixel buffer for intra prediction */
pel            nb[N_C][N_REF][MAX_CU_SIZE * 3];
/* reference index for current CU */
s8             refi[REFP_NUM];
/* motion vector for current CU */
s16            mv[REFP_NUM][MV_D];
#if DMVR_LAG
/* dmvr refined motion vector for current CU */
s16             dmvr_mv[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
#endif
/* CU position in current frame in SCU unit */
u32            scup;
/* CU position X in a frame in SCU unit */
u16            x_scu;
/* CU position Y in a frame in SCU unit */
u16            y_scu;
/* neighbor CUs availability of current CU */
u16            avail_cu;
/* Left, right availability of current CU */
u16            avail_lr;
/* intra prediction direction of current CU */
u8             ipm[2];
/* most probable mode for intra prediction */
u8             * mpm_b_list;
u8             mpm[2];
u8             mpm_ext[8];
u8             pims[IPD_CNT]; /* probable intra mode set*/
/* prediction mode of current CU: INTRA, INTER, ... */
u8             pred_mode;
u8             DMVRenable;
/* log2 of cuw */
u8             log2_cuw;
/* log2 of cuh */
u8             log2_cuh;
/* is there coefficient? */
int            is_coef[N_C];
int            is_coef_sub[N_C][MAX_SUB_TB_NUM];
/* QP for Luma of current encoding MB */
u8             qp_y;
/* QP for Chroma of current encoding MB */
u8             qp_u;
u8             qp_v;
s16            affine_mv[REFP_NUM][VER_NUM][MV_D];
u8             affine_flag;

u8             ibc_flag;
u8             ibc_skip_flag;
u8             ibc_merge_flag;

#if DQP
u8             qp;
u8             cu_qp_delta_code;
u8             cu_qp_delta_is_coded;
#endif
/************** current LCU *************/
/* address of current LCU,  */
u16            lcu_num;
/* X address of current LCU */
u16            x_lcu;
/* Y address of current LCU */
u16            y_lcu;
/* left pel position of current LCU */
u16            x_pel;
/* top pel position of current LCU */
u16            y_pel;
/* split mode map for current LCU */
s8             split_mode[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
/* SUCO flag for current LCU */
s8             suco_flag[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
/* platform specific data, if needed */
void          *pf;
s16            mmvd_idx;
u8             mmvd_flag;
/* ATS_INTRA flags */
u8             ats_intra_cu;
u8             ats_intra_mode_h;
u8             ats_intra_mode_v;

/* ATS_INTER info (index + position)*/
u8             ats_inter_info;
/* temporal pixel buffer for inter prediction */
pel            eif_tmp_buffer[ (MAX_CU_SIZE + 2) * (MAX_CU_SIZE + 2) ];
u8             mvr_idx;
#if DMVR_FLAG
u8            dmvr_flag;
#endif

/* history-based motion vector prediction candidate list */
EVC_HISTORY_BUFFER     history_buffer;
#if AFFINE_UPDATE
// spatial neighboring MV of affine block
s8             refi_sp[REFP_NUM];
s16            mv_sp[REFP_NUM][MV_D];
#endif
#if TRACE_ENC_CU_DATA
u64            trace_idx;
#endif
int            mvp_idx[REFP_NUM];
s16            mvd[REFP_NUM][MV_D];
int            inter_dir;
int            bi_idx;
int            affine_bzero[REFP_NUM];
s16            affine_mvd[REFP_NUM][3][MV_D];
int            tile_num;
u8             ctx_flags[NUM_CNID];
#if M50761_CHROMA_NOT_SPLIT
TREE_CONS      tree_cons;
#endif
*/}
/******************************************************************************
 * CONTEXT used for decoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCtx {
    /* magic code */
    pub(crate) magic: u32,

    /* EVCD identifier */
    //EVCD                    id;
    /* CORE information used for fast operation */
    // core: EvcdCore,
    /* current decoding bitstream */
    bs: EvcdBsr,
    /* current nalu header */
    nalu: EvcNalu,
    /* current slice header */
    sh: EvcSh,
    /* decoded picture buffer management */
    // EVC_PM                  dpm;
    /* create descriptor */
    //EVCD_CDSC               cdsc;
    /* sequence parameter set */
    sps: EvcSps,
    /* picture parameter set */
    pps: EvcPps,
    /* current decoded (decoding) picture buffer */
    //EVC_PIC               * pic;
    /* SBAC */
    //EVCD_SBAC               sbac_dec;
    /* decoding picture width */
    w: u16,
    /* decoding picture height */
    h: u16,
    /* maximum CU width and height */
    max_cuwh: u16,
    /* log2 of maximum CU width and height */
    log2_max_cuwh: u8,

    /* minimum CU width and height */
    min_cuwh: u16,
    /* log2 of minimum CU width and height */
    log2_min_cuwh: u8,
    /* MAPS *******************************************************************/
    /* SCU map for CU information */
    /*u32                   * map_scu;
    /* LCU split information */
    s8                   (* map_split)[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    s8                   (* map_suco)[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    /* decoded motion vector for every blocks */
    s16                  (* map_mv)[REFP_NUM][MV_D];
    /* decoded motion vector for every blocks */
    s16                  (* map_unrefined_mv)[REFP_NUM][MV_D];
    /* reference frame indices */
    s8                   (* map_refi)[REFP_NUM];
    /* intra prediction modes */
    s8                    * map_ipm;
    u32                   * map_affine;
    /* new coding tool flag*/
    u32                   * map_cu_mode;
    /* ats_inter info map */
    u8                    * map_ats_inter;*/
    /**************************************************************************/
    /* current slice number, which is increased whenever decoding a slice.
    when receiving a slice for new picture, this value is set to zero.
    this value can be used for distinguishing b/w slices */
    slice_num: u16,
    /* last coded intra picture's picture order count */
    last_intra_poc: isize,
    /* picture width in LCU unit */
    w_lcu: u16,
    /* picture height in LCU unit */
    h_lcu: u16,
    /* picture size in LCU unit (= w_lcu * h_lcu) */
    f_lcu: u32,
    /* picture width in SCU unit */
    w_scu: u16,
    /* picture height in SCU unit */
    h_scu: u16,
    /* picture size in SCU unit (= w_scu * h_scu) */
    f_scu: u32,
    /* the picture order count value */
    //EVC_POC                 poc;
    /* the picture order count of the previous Tid0 picture */
    prev_pic_order_cnt_val: u32,
    /* the decoding order count of the previous picture */
    prev_doc_offset: u32,
    /* the number of currently decoded pictures */
    pic_cnt: u32,
    /* flag whether current picture is refecened picture or not */
    slice_ref_flag: u8,
    /* distance between ref pics in addition to closest ref ref pic in LD*/
    ref_pic_gap_length: isize,
    /* bitstream has an error? */
    bs_err: u8,
    /* reference picture (0: foward, 1: backward) */
    //EVC_REFP                refp[MAX_NUM_REF_PICS][REFP_NUM];
    /* flag for picture signature enabling */
    use_pic_sign: u8,
    /* picture signature (MD5 digest 128bits) for each component */
    pic_sign: [[u8; 16]; N_C],
    /* flag to indicate picture signature existing or not */
    pic_sign_exist: u8,
    /* flag to indicate opl decoder output */
    use_opl: u8,
    num_ctb: u32,
}

const nalu_size_field_in_bytes: usize = 4;

impl EvcdCtx {
    pub(crate) fn decode_nalu(&mut self, pkt: &mut Packet) -> Result<EvcdStat, EvcError> {
        let data = pkt.data.take();
        let buf = if let Some(b) = data {
            b
        } else {
            return Err(EvcError::EVC_ERR_EMPTY_PACKET);
        };

        /* bitstream reader initialization */
        self.bs = EvcdBsr::new(buf);

        /* parse nalu header */
        let nalu = evcd_eco_nalu(&mut self.bs)?;

        Ok(EvcdStat {
            nalu_type: (nalu.nal_unit_type_plus1 - 1),
            read: nalu_size_field_in_bytes + self.bs.EVC_BSR_GET_READ_BYTE() as usize,
            ..Default::default()
        })
    }
}
