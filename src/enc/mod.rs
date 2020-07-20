use super::api::frame::*;
use super::api::*;
use super::def::*;
use super::picman::*;

use std::cell::RefCell;
use std::rc::Rc;

/* EVC encoder magic code */
pub(crate) const EVCE_MAGIC_CODE: u32 = 0x45565945; /* EVYE */

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
    //EVCE_CU_DATA  cu_data_best[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_CU_DATA  cu_data_temp[MAX_CU_DEPTH][MAX_CU_DEPTH];

    //EVCE_DQP      dqp_data[MAX_CU_DEPTH][MAX_CU_DEPTH];

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
    //EVCE_DQP       dqp_curr_best[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_DQP       dqp_next_best[MAX_CU_DEPTH][MAX_CU_DEPTH];
    //EVCE_DQP       dqp_temp_best;
    //EVCE_DQP       dqp_temp_best_merge;
    //EVCE_DQP       dqp_temp_run;

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
    delta_dist: [i64; N_C], //delta distortion from filtering (negative values mean distortion reduced)
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
    rdoq_est_last: [[i32; 2]; NUM_CTX_CC_LAST],
}

/******************************************************************************
 * CONTEXT used for encoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
pub(crate) struct EvceCtx {
    /* address of current input picture, ref_picture  buffer structure */
    //EVCE_PICO            * pico_buf[EVCE_MAX_INBUF_CNT];
    /* address of current input picture buffer structure */
    //EVCE_PICO * pico;
    /* index of current input picture buffer in pico_buf[] */
    pico_idx: u8,
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
    //EVCE_PARAM             param;
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
    rpm: Option<EvcPm>,
    /* create descriptor */
    //EVCE_CDSC              cdsc;
    /* quantization value of current encoding slice */
    qp: u8,
    /* offset value of alpha and beta for deblocking filter */
    deblock_alpha_offset: u8,
    deblock_beta_offset: u8,
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
    pic_cnt: u32,
    /* current picture input count (only update when CTX0) */
    pic_icnt: u32,
    /* total input picture count (only used for bumping process) */
    pic_ticnt: u32,
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
    //EVCE_MODE              mode;
    /* intra prediction analysis */
    //EVCE_PINTRA            pintra;
    /* inter prediction analysis */
    //EVCE_PINTER            pinter;
    /* MAPS *******************************************************************/
    /* CU map (width in SCU x height in SCU) of raster scan order in a frame */
    map_scu: Vec<MCU>,
    /* cu data for current LCU */
    //EVCE_CU_DATA * map_cu_data;
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
        let mut refp = vec![];
        for j in 0..MAX_NUM_REF_PICS {
            let mut refp1d = vec![]; //[[EvcRefP::new(); REFP_NUM]; MAX_NUM_REF_PICS];
            for i in 0..REFP_NUM {
                refp1d.push(EvcRefP::new());
            }
            refp.push(refp1d);
        }

        EvceCtx {
            /* address of current input picture, ref_picture  buffer structure */
            //EVCE_PICO            * pico_buf[EVCE_MAX_INBUF_CNT];
            /* address of current input picture buffer structure */
            //EVCE_PICO * pico;
            /* index of current input picture buffer in pico_buf[] */
            pico_idx: 0,
            pico_max_cnt: 0,

            /* magic code */
            magic: EVCE_MAGIC_CODE,
            frm: None,

            /* EVCE identifier */
            //EVCE                   id;
            /* address of core structure */
            core: EvceCore::default(),
            /* current input (original) image */
            //EVC_PIC                pic_o;
            /* address indicating current encoding, list0, list1 and original pictures */
            //EVC_PIC * pic[PIC_D + 1]; /* the last one is for original */
            /* picture address for mode decision */
            //EVC_PIC * pic_m;
            /* reference picture (0: foward, 1: backward) */
            refp,
            /* encoding parameter */
            //EVCE_PARAM             param;
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
            rpm: None,
            /* create descriptor */
            //EVCE_CDSC              cdsc;
            /* quantization value of current encoding slice */
            qp: 0,
            /* offset value of alpha and beta for deblocking filter */
            deblock_alpha_offset: 0,
            deblock_beta_offset: 0,
            /* encoding picture width */
            w: 0,
            /* encoding picture height */
            h: 0,
            /* encoding picture width * height */
            f: 0,
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
            frm_rnum: 0,
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
            max_cud: 0,
            //EVCE_SBAC              sbac_enc;
            /* address of inbufs */
            //EVC_IMGB * inbuf[EVCE_MAX_INBUF_CNT];
            /* last coded intra picture's picture order count */
            last_intra_poc: 0,
            /* maximum CU width and height */
            max_cuwh: 0,
            /* log2 of maximum CU width and height */
            log2_max_cuwh: 0,
            /* minimum CU width and height */
            min_cuwh: 0,
            /* log2 of minimum CU width and height */
            log2_min_cuwh: 0,
            /* total count of remained LCU for encoding one picture. if a picture is
            encoded properly, this value should reach to zero */
            lcu_cnt: 0,
            /* picture width in LCU unit */
            w_lcu: 0,
            /* picture height in LCU unit */
            h_lcu: 0,
            /* picture size in LCU unit (= w_lcu * h_lcu) */
            f_lcu: 0,
            /* picture width in SCU unit */
            w_scu: 0,
            /* picture height in SCU unit */
            h_scu: 0,
            /* picture size in SCU unit (= w_scu * h_scu) */
            f_scu: 0,
            /* log2 of SCU count in a LCU row */
            log2_culine: 0,
            /* log2 of SCU count in a LCU (== log2_culine * 2) */
            log2_cudim: 0,
            /* mode decision structure */
            //EVCE_MODE              mode;
            /* intra prediction analysis */
            //EVCE_PINTRA            pintra;
            /* inter prediction analysis */
            //EVCE_PINTER            pinter;
            /* MAPS *******************************************************************/
            /* CU map (width in SCU x height in SCU) of raster scan order in a frame */
            map_scu: vec![],
            /* cu data for current LCU */
            //EVCE_CU_DATA * map_cu_data;
            /* map for encoded motion vectors in SCU */
            map_mv: None,
            /* map for reference indices */
            map_refi: None,
            /* map for intra pred mode */
            map_ipm: vec![],
            map_depth: vec![],
            //EVC_PIC              * pic_dbk;          //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
            map_cu_mode: vec![],
            lambda: [0.0; 3],
            sqrt_lambda: [0.0; 3],
            dist_chroma_weight: [0.0; 2],
        }
    }

    pub(crate) fn push_frm(&mut self, frm: &mut Option<Frame<pel>>) -> Result<(), EvcError> {
        self.frm = frm.take();
        Ok(())
    }

    pub(crate) fn encode_frm(&mut self) -> Result<(), EvcError> {
        Ok(())
    }

    pub(crate) fn pull_pkt(&mut self) -> Result<(), EvcError> {
        Ok(())
    }
}
