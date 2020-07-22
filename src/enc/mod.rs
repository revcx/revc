use super::api::frame::*;
use super::api::*;
use super::def::*;
use super::picman::*;
use super::tbl::*;

use std::cell::RefCell;
use std::rc::Rc;

/* support RDOQ */
pub(crate) const SCALE_BITS: usize = 15; /* Inherited from TMuC, pressumably for fractional bit estimates in RDOQ */
pub(crate) const ERR_SCALE_PRECISION_BITS: usize = 20;
/* EVC encoder magic code */
pub(crate) const EVCE_MAGIC_CODE: u32 = 0x45565945; /* EVYE */

/* Max. and min. Quantization parameter */
pub(crate) const MAX_QUANT: u8 = 51;
pub(crate) const MIN_QUANT: u8 = 0;

pub(crate) const GOP_P: usize = 8;

/* count of picture including encoding and reference pictures
0: encoding picture buffer
1: forward reference picture buffer
2: backward reference picture buffer, if exists
3: original (input) picture buffer
4: mode decision picture buffer, if exists
*/
pub(crate) enum PicIdx {
    /* current encoding picture buffer index */
    PIC_IDX_CURR = 0,
    /* list0 reference picture buffer index */
    PIC_IDX_FORW = 1,
    /* list1 reference picture buffer index */
    PIC_IDX_BACK = 2,
    /* original (input) picture buffer index */
    PIC_IDX_ORIG = 3,
    /* mode decision picture buffer index */
    PIC_IDX_MODE = 4,
    PIC_D = 5,
}

/* check whether bumping is progress or not */
// FORCE_OUT(ctx)          (ctx->param.force_output == 1)

/* motion vector accuracy level for inter-mode decision */
pub(crate) const ME_LEV_IPEL: usize = 1;
pub(crate) const ME_LEV_HPEL: usize = 2;
pub(crate) const ME_LEV_QPEL: usize = 3;

/* maximum inbuf count */
pub(crate) const EVCE_MAX_INBUF_CNT: usize = 34;

/* maximum cost value */
pub(crate) const MAX_COST: f64 = (1.7e+308);

/*****************************************************************************
 * mode decision structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvceMode {
    //void *pdata[4];
    //int  *ndata[4];
    //pel  *rec[N_C];
    //int   s_rec[N_C];

    /* CU count in a CU row in a LCU (== log2_max_cuwh - MIN_CU_LOG2) */
    log2_culine: u8,
    /* reference indices */
    refi: [i8; REFP_NUM],
    /* MVP indices */
    mvp_idx: [u8; REFP_NUM],
    /* MVR indices */
    //u8    mvr_idx;
    bi_idx: u8,
    /* mv difference */
    mvd: [[i16; MV_D]; REFP_NUM],

    /* mv */
    mv: [[i16; MV_D]; REFP_NUM],

    //pel  *pred_y_best;
    cu_mode: MCU,
}

/* virtual frame depth B picture */
pub(crate) const FRM_DEPTH_0: usize = 0;
pub(crate) const FRM_DEPTH_1: usize = 1;
pub(crate) const FRM_DEPTH_2: usize = 2;
pub(crate) const FRM_DEPTH_3: usize = 3;
pub(crate) const FRM_DEPTH_4: usize = 4;
pub(crate) const FRM_DEPTH_5: usize = 5;
pub(crate) const FRM_DEPTH_6: usize = 6;
pub(crate) const FRM_DEPTH_MAX: usize = 7;
/* I-slice, P-slice, B-slice + depth + 1 (max for GOP 8 size)*/
pub(crate) const LIST_NUM: usize = 1;

/*****************************************************************************
 * original picture buffer structure
 *****************************************************************************/
//#[derive(Default)]
pub(crate) struct EvcePicOrg {
    /* original picture store */
    pic: EvcPic,
    /* input picture count */
    pic_icnt: u32,
    /* be used for encoding input */
    is_used: bool,
    /* address of sub-picture */
    //EVC_PIC              * spic;
}

/*****************************************************************************
 * intra prediction structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcePIntra {
    /* temporary prediction buffer */
    pred: CUBuffer<pel>, //[N_C][MAX_CU_DIM];
    //pred_cache: [[pel; MAX_CU_DIM]; IntraPredDir::IPD_CNT_B as usize], // only for luma

    /* reconstruction buffer */
    rec: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    coef_tmp: CUBuffer<i16>,  //[N_C][MAX_CU_DIM];
    coef_best: CUBuffer<i16>, //[N_C][MAX_CU_DIM];
    nnz_best: [u16; N_C],
    rec_best: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    /* original (input) picture buffer */
    //EVC_PIC          * pic_o;
    /* address of original (input) picture buffer */
    //pel               * o[N_C];
    /* stride of original (input) picture buffer */
    //int                 s_o[N_C];
    /* mode picture buffer */
    //EVC_PIC          * pic_m;
    /* address of mode picture buffer */

    //pel               * m[N_C];
    /* stride of mode picture buffer */
    //int                 s_m[N_C];

    /* QP for luma */
    qp_y: u8,
    /* QP for chroma */
    qp_u: u8,
    qp_v: u8,

    slice_type: SliceType,

    complexity: i64,
    //void              * pdata[4];
    //int               * ndata[4];
}

/*****************************************************************************
 * inter prediction structure
 *****************************************************************************/
pub(crate) const MV_RANGE_MIN: usize = 0;
pub(crate) const MV_RANGE_MAX: usize = 1;
pub(crate) const MV_RANGE_DIM: usize = 2;

#[derive(Default)]
pub(crate) struct EvcePInter {
    /* temporary prediction buffer (only used for ME)*/
    //pred_buf: [pel; MAX_CU_DIM],

    /* temporary buffer for analyze_cu */
    refi: [[i8; REFP_NUM]; InterPredDir::PRED_NUM as usize],
    /* Ref idx predictor */
    refi_pred: [[i8; MAX_NUM_MVP]; REFP_NUM],
    mvp_idx: [[u8; REFP_NUM]; InterPredDir::PRED_NUM as usize],
    /*s16  mvp_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME][MAX_NUM_MVP][MV_D];
        s16  mv_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME][MV_D];
        u8   mvp_idx_temp_for_bi[PRED_NUM][REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME];
        int  best_index[PRED_NUM][4];
        s16  mmvd_idx[PRED_NUM];
        u8   mvr_idx[PRED_NUM];
        u8   curr_mvr;
        int  max_imv[MV_D];
        s8   first_refi[PRED_NUM][REFP_NUM];
        u8   bi_idx[PRED_NUM];
        u8   curr_bi;
        int max_search_range;
        s16  affine_mvp_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME][MAX_NUM_MVP][VER_NUM][MV_D];
        s16  affine_mv_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME][VER_NUM][MV_D];
        u8   mvp_idx_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME];

        s16  affine_mvp[REFP_NUM][MAX_NUM_MVP][VER_NUM][MV_D];
        s16  affine_mv[PRED_NUM][REFP_NUM][VER_NUM][MV_D];
        s16  affine_mvd[PRED_NUM][REFP_NUM][VER_NUM][MV_D];

        pel  p_error[MAX_CU_DIM];
        int  i_gradient[2][MAX_CU_DIM];
        s16  resi[N_C][MAX_CU_DIM];
        s16  coff_save[N_C][MAX_CU_DIM];
        u8   ats_inter_info_mode[PRED_NUM];
        /* MV predictor */
        s16  mvp[REFP_NUM][MAX_NUM_MVP][MV_D];

        s16  mv[PRED_NUM][REFP_NUM][MV_D];
        s16  mvd[PRED_NUM][REFP_NUM][MV_D];

        s16  org_bi[MAX_CU_DIM];
        s32  mot_bits[REFP_NUM];
        /* temporary prediction buffer (only used for ME)*/
        pel  pred[PRED_NUM+1][2][N_C][MAX_CU_DIM];
        pel  dmvr_template[MAX_CU_DIM];
        pel dmvr_half_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + 1) * (MAX_CU_SIZE + 1)];
    #if DMVR_PADDING
        pel  dmvr_padding_buf[PRED_NUM][N_C][PAD_BUFFER_STRIDE * PAD_BUFFER_STRIDE];
    #endif
        pel  dmvr_ref_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT)) * (MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT))];

        /* reconstruction buffer */
        pel  rec[PRED_NUM][N_C][MAX_CU_DIM];
        /* last one buffer used for RDO */
        s16  coef[PRED_NUM+1][N_C][MAX_CU_DIM];

        s16  residue[N_C][MAX_CU_DIM];
        int  nnz_best[PRED_NUM][N_C];
        int  nnz_sub_best[PRED_NUM][N_C][MAX_SUB_TB_NUM];

        u8   num_refp;
        /* minimum clip value */
        s16  min_clip[MV_D];
        /* maximum clip value */
        s16  max_clip[MV_D];
        /* search range for int-pel */
        s16  search_range_ipel[MV_D];
        /* search range for sub-pel */
        s16  search_range_spel[MV_D];
        s8  (*search_pattern_hpel)[2];
        u8   search_pattern_hpel_cnt;
        s8  (*search_pattern_qpel)[2];
        u8   search_pattern_qpel_cnt;

        /* original (input) picture buffer */
        EVC_PIC        *pic_o;
        /* address of original (input) picture buffer */
        pel             *o[N_C];
        /* stride of original (input) picture buffer */
        int              s_o[N_C];
        /* mode picture buffer */
        EVC_PIC        *pic_m;
        /* address of mode picture buffer */
        pel             *m[N_C];
        /* stride of mode picture buffer */
        int              s_m[N_C];
        /* motion vector map */
        s16            (*map_mv)[REFP_NUM][MV_D];

        /* picture width in SCU unit */
        u16              w_scu;
        /* QP for luma of current encoding CU */
        u8               qp_y;
        /* QP for chroma of current encoding CU */
        u8               qp_u;
        u8               qp_v;
        u32              lambda_mv;
        /* reference pictures */
        EVC_REFP      (*refp)[REFP_NUM];
        int              slice_type;
        /* search level for motion estimation */
        int              me_level;
        int              complexity;
        void            *pdata[4];
        int             *ndata[4];
        /* current picture order count */
        int              poc;
        /* gop size */
        int              gop_size;
         */
}

#[derive(Default)]
pub(crate) struct EvceDQP {
    prev_QP: i8,
    curr_QP: i8,
    cu_qp_delta_is_coded: bool,
    cu_qp_delta_code: i8,
}

pub(crate) struct EvceCUData {
    split_mode: Vec<Vec<Vec<i8>>>,
    /*u8  *qp_y;
    u8  *qp_u;
    u8  *qp_v;
    u8  *pred_mode;
    u8  *pred_mode_chroma;
    u8  **mpm;
    u8  **mpm_ext;
    s8  **ipm;
    u8  *skip_flag;
    s8  **refi;
    u8  **mvp_idx;
    u8  *bi_idx;
    s16 bv_chroma[MAX_CU_CNT_IN_LCU][MV_D];
    s16 mv[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
    s16 mvd[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
    int *nnz[N_C];
    u32 *map_scu;
    u32 *map_cu_mode;
    s8  *depth;
    s16 *coef[N_C];
    pel *reco[N_C]; */
}

impl Default for EvceCUData {
    fn default() -> Self {
        EvceCUData {
            split_mode: vec![
                vec![vec![0; MAX_CU_CNT_IN_LCU]; BlockShape::NUM_BLOCK_SHAPE as usize];
                NUM_CU_DEPTH
            ],
        }
    }
}
impl EvceCUData {
    fn new(log2_cuw: usize, log2_cuh: usize) -> Self {
        EvceCUData::default()
    }
    fn init(&mut self, log2_cuw: usize, log2_cuh: usize) {
        /*int i, j;
            int cuw_scu, cuh_scu;
            int size_8b, size_16b, size_32b, cu_cnt, pixel_cnt;

            cuw_scu = 1 << log2_cuw;
            cuh_scu = 1 << log2_cuh;

            size_8b = cuw_scu * cuh_scu * sizeof(s8);
            size_16b = cuw_scu * cuh_scu * sizeof(s16);
            size_32b = cuw_scu * cuh_scu * sizeof(s32);
            cu_cnt = cuw_scu * cuh_scu;
            pixel_cnt = cu_cnt << 4;

            evce_malloc_1d((void**)&cu_data->qp_y, size_8b);
            evce_malloc_1d((void**)&cu_data->qp_u, size_8b);
            evce_malloc_1d((void**)&cu_data->qp_v, size_8b);
            evce_malloc_1d((void**)&cu_data->pred_mode, size_8b);
            evce_malloc_1d((void**)&cu_data->pred_mode_chroma, size_8b);
            evce_malloc_2d((s8***)&cu_data->mpm, 2, cu_cnt, sizeof(u8));
            evce_malloc_2d((s8***)&cu_data->ipm, 2, cu_cnt, sizeof(u8));
            evce_malloc_2d((s8***)&cu_data->mpm_ext, 8, cu_cnt, sizeof(u8));
            evce_malloc_1d((void**)&cu_data->skip_flag, size_8b);
            evce_malloc_1d((void**)&cu_data->ibc_flag, size_8b);
        #if DMVR_FLAG
            evce_malloc_1d((void**)&cu_data->dmvr_flag, size_8b);
        #endif
            evce_malloc_2d((s8***)&cu_data->refi, cu_cnt, REFP_NUM, sizeof(u8));
            evce_malloc_2d((s8***)&cu_data->mvp_idx, cu_cnt, REFP_NUM, sizeof(u8));
            evce_malloc_1d((void**)&cu_data->mvr_idx, size_8b);
            evce_malloc_1d((void**)&cu_data->bi_idx, size_8b);
            evce_malloc_1d((void**)&cu_data->mmvd_idx, size_16b);
            evce_malloc_1d((void**)&cu_data->mmvd_flag, size_8b);

            evce_malloc_1d((void**)& cu_data->ats_intra_cu, size_8b);
            evce_malloc_1d((void**)& cu_data->ats_mode_h, size_8b);
            evce_malloc_1d((void**)& cu_data->ats_mode_v, size_8b);

            evce_malloc_1d((void**)&cu_data->ats_inter_info, size_8b);

            for(i = 0; i < N_C; i++)
            {
                evce_malloc_1d((void**)&cu_data->nnz[i], size_32b);
            }
            for (i = 0; i < N_C; i++)
            {
                for (j = 0; j < 4; j++)
                {
                    evce_malloc_1d((void**)&cu_data->nnz_sub[i][j], size_32b);
                }
            }
            evce_malloc_1d((void**)&cu_data->map_scu, size_32b);
            evce_malloc_1d((void**)&cu_data->affine_flag, size_8b);
            evce_malloc_1d((void**)&cu_data->map_affine, size_32b);
            evce_malloc_1d((void**)&cu_data->map_cu_mode, size_32b);
            evce_malloc_1d((void**)&cu_data->depth, size_8b);

            for(i = 0; i < N_C; i++)
            {
                evce_malloc_1d((void**)&cu_data->coef[i], (pixel_cnt >> (!!(i)* 2)) * sizeof(s16));
                evce_malloc_1d((void**)&cu_data->reco[i], (pixel_cnt >> (!!(i)* 2)) * sizeof(pel));
            }
            */
    }
}

/*****************************************************************************
 * CORE information used for encoding process.
 *
 * The variables in this structure are very often used in encoding process.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvceCore {
    /* coefficient buffer of current CU */
    coef: CUBuffer<i16>, //[[i16; MAX_CU_DIM]; N_C]
    /* CU data for RDO */
    cu_data_best: Vec<Vec<EvceCUData>>, //[[EvceCUData; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    cu_data_temp: Vec<Vec<EvceCUData>>, //[[EvceCUData; MAX_CU_DEPTH]; MAX_CU_DEPTH],

    dqp_data: Vec<Vec<EvceDQP>>, //[[EvceDQP; MAX_CU_DEPTH]; MAX_CU_DEPTH],

    /* temporary coefficient buffer */
    ctmp: CUBuffer<i16>, //[[i16;MAX_CU_DIM];N_C]
    /* pred buffer of current CU. [1][x][x] is used for bi-pred */
    pred: [CUBuffer<pel>; 2], //[2][N_C][MAX_CU_DIM];
    /* neighbor pixel buffer for intra prediction */
    nb: NBBuffer<pel>, //[N_C][N_REF][MAX_CU_SIZE * 3];
    /* current encoding LCU number */
    lcu_num: u16,
    /*QP for current encoding CU. Used to derive Luma and chroma qp*/
    qp: u8,
    cu_qp_delta_code: u8,
    cu_qp_delta_is_coded: u8,
    cu_qp_delta_code_mode: u8,
    dqp_curr_best: Vec<Vec<EvceCUData>>, //[[EvceCUData; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    dqp_next_best: Vec<Vec<EvceCUData>>, //[[EvceCUData; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    dqp_temp_best: EvceCUData,
    dqp_temp_best_merge: EvceCUData,
    dqp_temp_run: EvceCUData,

    /* QP for luma of current encoding CU */
    qp_y: u8,
    /* QP for chroma of current encoding CU */
    qp_u: u8,
    qp_v: u8,
    /* X address of current LCU */
    x_lcu: u16,
    /* Y address of current LCU */
    y_lcu: u16,
    /* X address of current CU in SCU unit */
    x_scu: u16,
    /* Y address of current CU in SCU unit */
    y_scu: u16,
    /* left pel position of current LCU */
    x_pel: u16,
    /* top pel position of current LCU */
    y_pel: u16,
    /* CU position in current frame in SCU unit */
    scup: u32,
    /* CU position in current LCU in SCU unit */
    cup: u32,
    /* CU depth */
    cud: u16,
    /* neighbor CUs availability of current CU */
    avail_cu: u16,
    /* Left, right availability of current CU */
    avail_lr: u16,
    bef_data_idx: u16,
    /* CU mode */
    cu_mode: MCU,
    /* intra prediction mode */
    //u8             mpm[2]; /* mpm table pointer*/
    //u8             mpm_ext[8];
    mpm_b_list: &'static [u8],
    pims: [u8; IntraPredDir::IPD_CNT_B as usize], /* probable intra mode set*/
    ipm: [IntraPredDir; 2],
    /* skip flag for MODE_INTER */
    skip_flag: bool,

    /* width of current CU */
    cuw: u16,
    /* height of current CU */
    cuh: u16,
    /* log2 of cuw */
    log2_cuw: u8,
    /* log2 of cuh */
    log2_cuh: u8,
    /* number of non-zero coefficient */
    nnz: [u16; N_C],
    /* platform specific data, if needed */
    //void          *pf;
    /* bitstream structure for RDO */
    //EVC_BSW        bs_temp;
    /* SBAC structure for full RDO */
    //EVCE_SBAC      s_curr_best[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_SBAC      s_next_best[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_SBAC      s_temp_best;
    //EVCE_SBAC      s_temp_best_merge;
    //EVCE_SBAC      s_temp_run;
    //EVCE_SBAC      s_temp_prev_comp_best;
    //EVCE_SBAC      s_temp_prev_comp_run;
    //EVCE_SBAC      s_curr_before_split[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_BEF_DATA  bef_data[MAX_CU_DEPTH][MAX_CU_DEPTH][MAX_CU_CNT_IN_LCU][MAX_BEF_DATA_NUM];
    cost_best: f64,
    inter_satd: u32,
    dist_cu: i32,
    dist_cu_best: i32, //dist of the best intra mode (note: only updated in intra coding now)
    /* temporal pixel buffer for inter prediction */
    //pel            eif_tmp_buffer[(MAX_CU_SIZE + 2) * (MAX_CU_SIZE + 2)];
    //u8             au8_eval_mvp_idx[MAX_NUM_MVP];
    tree_cons: TREE_CONS,

    ctx_flags: [u8; CtxNevIdx::NUM_CNID as usize],
    //int            split_mode_child:[4];
    //int            parent_split_allow[6];

    //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
    /*delta_dist: [i64; N_C], //delta distortion from filtering (negative values mean distortion reduced)
    dist_nofilt: [i64; N_C], //distortion of not filtered samples
    dist_filter: [i64; N_C], //distortion of filtered samples
    /* RDOQ related variables*/
    rdoq_est_cbf_all: [i64; 2],
    rdoq_est_cbf_luma: [i64; 2],
    rdoq_est_cbf_cb: [i64; 2],
    rdoq_est_cbf_cr: [i64; 2],
    //rdoq_est_sig_coeff: [[i64; 2]; NUM_CTX_SIG_COEFF_FLAG],
    rdoq_est_gtx: [[i64; 2]; NUM_CTX_GTX],
    rdoq_est_last_sig_coeff_x: [[i64; 2]; NUM_CTX_LAST_SIG_COEFF],
    rdoq_est_last_sig_coeff_y: [[i64; 2]; NUM_CTX_LAST_SIG_COEFF],
    rdoq_est_run: [[i32; 2]; NUM_CTX_CC_RUN],
    rdoq_est_level: [[i32; 2]; NUM_CTX_CC_LEVEL],
    rdoq_est_last: [[i32; 2]; NUM_CTX_CC_LAST],*/
    evc_tbl_qp_chroma_dynamic_ext: [Vec<i8>; 2], // [[i8; MAX_QP_TABLE_SIZE_EXT]; 2],
}

/******************************************************************************
 * CONTEXT used for encoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
pub(crate) struct EvceCtx {
    /* address of current input picture, ref_picture  buffer structure */
    pico_buf: Vec<EvcePicOrg>,
    /* address of current input picture buffer structure */
    //pico: EvcePicOrg,
    /* index of current input picture buffer in pico_buf[] */
    pico_idx: usize,
    pico_max_cnt: usize,
    /* magic code */
    magic: u32,

    /* buffered packets */
    frm: Option<Frame<pel>>,

    /* EVCE identifier */
    //EVCE                   id;
    /* address of core structure */
    core: EvceCore,
    /* current input (original) image */
    //EVC_PIC                pic_o;
    /* address indicating current encoding, list0, list1 and original pictures */
    //EVC_PIC * pic[PIC_D + 1]; /* the last one is for original */
    /* picture address for mode decision */
    //EVC_PIC * pic_m;
    /* reference picture (0: foward, 1: backward) */
    refp: Vec<Vec<EvcRefP>>, //EVC_REFP               refp[MAX_NUM_REF_PICS][REFP_NUM];
    /* encoding parameter */
    param: EncoderConfig,
    /* bitstream structure */
    //EVC_BSW                bs;
    /* bitstream structure for RDO */
    //EVC_BSW                bs_temp;
    /* sequnce parameter set */
    sps: EvcSps,
    /* picture parameter set */
    pps: EvcPps,
    //EVC_PPS                 pps_array[64];
    /* picture order count */
    poc: EvcPoc,
    /* nal unit header */
    nalu: EvcNalu,
    /* slice header */
    sh: EvcSh,
    /* reference picture manager */
    rpm: EvcPm,
    /* create descriptor */
    //EVCE_CDSC              cdsc;
    /* quantization value of current encoding slice */
    qp: u8,
    /* encoding picture width */
    w: u16,
    /* encoding picture height */
    h: u16,
    /* encoding picture width * height */
    f: u32,
    /* the picture order count of the previous Tid0 picture */
    prev_pic_order_cnt_val: u32,
    /* the picture order count msb of the previous Tid0 picture */
    prev_pic_order_cnt_msb: u32,
    /* the picture order count lsb of the previous Tid0 picture */
    prev_pic_order_cnt_lsb: u32,
    /* the decoding order count of the previous picture */
    prev_doc_offset: u32,
    /* current encoding picture count(This is not PicNum or FrameNum.
    Just count of encoded picture correctly) */
    pic_cnt: usize,
    /* current picture input count (only update when CTX0) */
    pic_icnt: usize,
    /* total input picture count (only used for bumping process) */
    pic_ticnt: usize,
    /* remaining pictures is encoded to p or b slice (only used for bumping process) */
    force_slice: u8,
    /* ignored pictures for force slice count (unavailable pictures cnt in gop,\
    only used for bumping process) */
    force_ignored_cnt: u8,
    /* initial frame return number(delayed input count) due to B picture or Forecast */
    frm_rnum: u32,
    /* current encoding slice number in one picture */
    slice_num: i32,
    /* first mb number of current encoding slice in one picture */
    sl_first_mb: i32,
    /* current slice type */
    slice_type: SliceType,
    /* slice depth for current picture */
    slice_depth: u8,
    /* flag whether current picture is refecened picture or not */
    slice_ref_flag: bool,
    /* distance between ref pics in addition to closest ref ref pic in LD*/
    ref_pic_gap_length: u32,
    /* maximum CU depth */
    max_cud: u8,
    //EVCE_SBAC              sbac_enc;
    /* address of inbufs */
    //EVC_IMGB * inbuf[EVCE_MAX_INBUF_CNT];
    /* last coded intra picture's picture order count */
    last_intra_poc: i32,
    /* maximum CU width and height */
    max_cuwh: u16,
    /* log2 of maximum CU width and height */
    log2_max_cuwh: u8,
    /* minimum CU width and height */
    min_cuwh: u16,
    /* log2 of minimum CU width and height */
    log2_min_cuwh: u8,
    /* total count of remained LCU for encoding one picture. if a picture is
    encoded properly, this value should reach to zero */
    lcu_cnt: i32,
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
    /* log2 of SCU count in a LCU row */
    log2_culine: u8,
    /* log2 of SCU count in a LCU (== log2_culine * 2) */
    log2_cudim: u8,
    /* mode decision structure */
    mode: EvceMode,
    /* intra prediction analysis */
    pintra: EvcePIntra,
    /* inter prediction analysis */
    pinter: EvcePInter,
    /* MAPS *******************************************************************/
    /* CU map (width in SCU x height in SCU) of raster scan order in a frame */
    map_scu: Vec<MCU>,
    /* cu data for current LCU */
    map_cu_data: Vec<EvceCUData>,
    /* map for encoded motion vectors in SCU */
    map_mv: Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    /* map for reference indices */
    map_refi: Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    /* map for intra pred mode */
    map_ipm: Vec<IntraPredDir>,
    map_depth: Vec<i8>,
    //EVC_PIC              * pic_dbk;          //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
    map_cu_mode: Vec<MCU>,
    lambda: [f64; 3],
    sqrt_lambda: [f64; 3],
    dist_chroma_weight: [f64; 2],
}

impl EvceCtx {
    pub(crate) fn new(cfg: &Config) -> Self {
        let mut refp = Vec::with_capacity(MAX_NUM_REF_PICS);
        for j in 0..MAX_NUM_REF_PICS {
            let mut refp1d = Vec::with_capacity(REFP_NUM);
            for i in 0..REFP_NUM {
                refp1d.push(EvcRefP::new());
            }
            refp.push(refp1d);
        }

        let mut core = EvceCore::default();

        core.evc_tbl_qp_chroma_dynamic_ext[0] = vec![0; MAX_QP_TABLE_SIZE_EXT];
        core.evc_tbl_qp_chroma_dynamic_ext[1] = vec![0; MAX_QP_TABLE_SIZE_EXT];
        /*if sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            evc_derived_chroma_qp_mapping_tables(
                &sps.chroma_qp_table_struct,
                &mut core.evc_tbl_qp_chroma_dynamic_ext,
            );
        } else*/
        {
            core.evc_tbl_qp_chroma_dynamic_ext[0].copy_from_slice(&evc_tbl_qp_chroma_ajudst_base);
            core.evc_tbl_qp_chroma_dynamic_ext[1].copy_from_slice(&evc_tbl_qp_chroma_ajudst_base);
        }

        for i in 0..MAX_CU_DEPTH {
            for j in 0..MAX_CU_DEPTH {
                core.cu_data_best[i][j].init(i, j);
                core.cu_data_temp[i][j].init(i, j);
            }
        }

        let param = cfg.enc.unwrap();

        let w = param.width as u16;
        let h = param.height as u16;
        let f = w as u32 * h as u32;
        let max_cuwh = 64;
        let min_cuwh = 1 << 2;
        let log2_min_cuwh = 2;
        let log2_max_cuwh = 6;
        let max_cud = log2_max_cuwh - MIN_CU_LOG2 as u8;
        let w_lcu = (w + max_cuwh - 1) >> 6;
        let h_lcu = (h + max_cuwh - 1) >> 6;
        let f_lcu = w_lcu as u32 * h_lcu as u32;
        let w_scu = (w + ((1 << MIN_CU_LOG2) - 1)) >> MIN_CU_LOG2;
        let h_scu = (h + ((1 << MIN_CU_LOG2) - 1)) >> MIN_CU_LOG2;
        let f_scu = w_scu as u32 * h_scu as u32;
        let log2_culine = log2_max_cuwh - MIN_CU_LOG2 as u8;
        let log2_cudim = log2_culine << 1;

        /*  allocate CU data map*/
        let mut map_cu_data = Vec::with_capacity(f_lcu as usize);
        for i in 0..f_lcu as usize {
            let mut cu_data = EvceCUData::new(
                log2_max_cuwh as usize - MIN_CU_LOG2,
                log2_max_cuwh as usize - MIN_CU_LOG2,
            );
            map_cu_data.push(cu_data);
        }

        /* allocate maps */
        let map_scu = vec![MCU::default(); f_scu as usize];

        let map_ipm = vec![IntraPredDir::default(); f_scu as usize];
        let map_depth = vec![-1; f_scu as usize];
        let map_cu_mode = vec![MCU::default(); f_scu as usize];

        let pico_max_cnt = 1 + ((param.max_b_frames as usize) << 1);
        /* initialize decode picture manager */
        let mut rpm = EvcPm::new(w as usize, h as usize, param.chroma_sampling);
        rpm.evc_picman_init(
            MAX_PB_SIZE as u8,
            MAX_NUM_REF_PICS as u8,
            //PICBUF_ALLOCATOR * pa
        );

        let mut pico_buf = vec![];
        for i in 0..pico_max_cnt {
            //pico_buf.push(EvcePicOrg::default());
        }

        EvceCtx {
            /* address of current input picture, ref_picture  buffer structure */
            pico_buf,
            /* address of current input picture buffer structure */
            //pico://EVCE_PICO *
            /* index of current input picture buffer in pico_buf[] */
            pico_idx: 0,
            pico_max_cnt,

            /* magic code */
            magic: EVCE_MAGIC_CODE,
            frm: None,

            /* EVCE identifier */
            //EVCE                   id;
            /* address of core structure */
            core,
            /* current input (original) image */
            //EVC_PIC                pic_o;
            /* address indicating current encoding, list0, list1 and original pictures */
            //EVC_PIC * pic[PIC_D + 1]; /* the last one is for original */
            /* picture address for mode decision */
            //EVC_PIC * pic_m;
            /* reference picture (0: foward, 1: backward) */
            refp,
            /* encoding parameter */
            param,
            /* bitstream structure */
            //EVC_BSW                bs;
            /* bitstream structure for RDO */
            //EVC_BSW                bs_temp;
            /* sequnce parameter set */
            sps: EvcSps::default(),
            /* picture parameter set */
            pps: EvcPps::default(),
            //EVC_PPS                 pps_array[64];
            /* picture order count */
            poc: EvcPoc::default(),
            /* nal unit header */
            nalu: EvcNalu::default(),
            /* slice header */
            sh: EvcSh::default(),
            /* reference picture manager */
            rpm,
            /* create descriptor */
            //EVCE_CDSC              cdsc;
            /* quantization value of current encoding slice */
            qp: param.qp,
            /* encoding picture width */
            w,
            /* encoding picture height */
            h,
            /* encoding picture width * height */
            f,
            /* the picture order count of the previous Tid0 picture */
            prev_pic_order_cnt_val: 0,
            /* the picture order count msb of the previous Tid0 picture */
            prev_pic_order_cnt_msb: 0,
            /* the picture order count lsb of the previous Tid0 picture */
            prev_pic_order_cnt_lsb: 0,
            /* the decoding order count of the previous picture */
            prev_doc_offset: 0,
            /* current encoding picture count(This is not PicNum or FrameNum.
            Just count of encoded picture correctly) */
            pic_cnt: 0,
            /* current picture input count (only update when CTX0) */
            pic_icnt: 0,
            /* total input picture count (only used for bumping process) */
            pic_ticnt: 0,
            /* remaining pictures is encoded to p or b slice (only used for bumping process) */
            force_slice: 0,
            /* ignored pictures for force slice count (unavailable pictures cnt in gop,\
            only used for bumping process) */
            force_ignored_cnt: 0,
            /* initial frame return number(delayed input count) due to B picture or Forecast */
            frm_rnum: param.max_b_frames as u32,
            /* current encoding slice number in one picture */
            slice_num: 0,
            /* first mb number of current encoding slice in one picture */
            sl_first_mb: 0,
            /* current slice type */
            slice_type: SliceType::default(),
            /* slice depth for current picture */
            slice_depth: 0,
            /* flag whether current picture is refecened picture or not */
            slice_ref_flag: false,
            /* distance between ref pics in addition to closest ref ref pic in LD*/
            ref_pic_gap_length: 0,
            /* maximum CU depth */
            max_cud,
            //EVCE_SBAC              sbac_enc;
            /* address of inbufs */
            //EVC_IMGB * inbuf[EVCE_MAX_INBUF_CNT];
            /* last coded intra picture's picture order count */
            last_intra_poc: 0,
            /* maximum CU width and height */
            max_cuwh,
            /* log2 of maximum CU width and height */
            log2_max_cuwh,
            /* minimum CU width and height */
            min_cuwh,
            /* log2 of minimum CU width and height */
            log2_min_cuwh,
            /* total count of remained LCU for encoding one picture. if a picture is
            encoded properly, this value should reach to zero */
            lcu_cnt: 0,
            /* picture width in LCU unit */
            w_lcu,
            /* picture height in LCU unit */
            h_lcu,
            /* picture size in LCU unit (= w_lcu * h_lcu) */
            f_lcu,
            /* picture width in SCU unit */
            w_scu,
            /* picture height in SCU unit */
            h_scu,
            /* picture size in SCU unit (= w_scu * h_scu) */
            f_scu,
            /* log2 of SCU count in a LCU row */
            log2_culine,
            /* log2 of SCU count in a LCU (== log2_culine * 2) */
            log2_cudim,
            /* mode decision structure */
            mode: EvceMode::default(),
            /* intra prediction analysis */
            pintra: EvcePIntra::default(),
            /* inter prediction analysis */
            pinter: EvcePInter::default(),
            /* MAPS *******************************************************************/
            /* CU map (width in SCU x height in SCU) of raster scan order in a frame */
            map_scu,
            /* cu data for current LCU */
            map_cu_data,
            /* map for encoded motion vectors in SCU */
            map_mv: None,
            /* map for reference indices */
            map_refi: None,
            /* map for intra pred mode */
            map_ipm,
            map_depth,
            //EVC_PIC              * pic_dbk;          //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
            map_cu_mode,
            lambda: [0.0; 3],
            sqrt_lambda: [0.0; 3],
            dist_chroma_weight: [0.0; 2],
        }
    }

    pub(crate) fn push_frm(&mut self, frm: &mut Option<Frame<pel>>) -> Result<(), EvcError> {
        self.pico_idx = self.pic_icnt % self.pico_max_cnt;
        //self.pico = self.pico_buf[self.pico_idx];
        //self.pico->pic_icnt = ctx->pic_icnt;
        //self.pico->is_used = 1;
        self.pic_icnt += 1;
        self.frm = frm.take();
        Ok(())
    }

    pub(crate) fn encode_frm(&mut self) -> Result<EvcStat, EvcError> {
        if self.frm.is_none() {
            return Err(EvcError::EVC_OK_FLUSH);
        }

        Err(EvcError::EVC_ERR_EMPTY_FRAME)
    }

    pub(crate) fn pull_pkt(&mut self) -> Result<Rc<RefCell<Packet>>, EvcError> {
        Err(EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE)
    }
}
