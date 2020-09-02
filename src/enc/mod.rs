pub(crate) mod bsw;
pub(crate) mod eco;
pub(crate) mod me;
pub(crate) mod mode;
pub(crate) mod pinter;
pub(crate) mod pintra;
pub(crate) mod sad;
pub(crate) mod sbac;
pub(crate) mod tbl;
pub(crate) mod tq;
pub(crate) mod util;

use super::api::frame::*;
use super::api::*;
use super::def::*;
use super::df::*;
use super::ipred::*;
use super::picman::*;
use super::tbl::*;
use super::tracer::*;
use super::util::*;

use bsw::*;
use eco::*;
use mode::*;
use pinter::*;
use pintra::*;
use sbac::*;
use tbl::*;
use util::*;

use crate::tracer::{Tracer, OPEN_TRACE};
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::private::DisplayAsDisplay;

/* support RDOQ */
pub(crate) const SCALE_BITS: usize = 15; /* Inherited from TMuC, pressumably for fractional bit estimates in RDOQ */
pub(crate) const ERR_SCALE_PRECISION_BITS: usize = 20;

/* Max. and min. Quantization parameter */
pub(crate) const MAX_QUANT: u8 = 51;
pub(crate) const MIN_QUANT: u8 = 0;

pub(crate) const GOP_P: usize = 8;

pub(crate) const USE_RDOQ: bool = true; // Use RDOQ
pub(crate) const MAX_TX_DYNAMIC_RANGE: usize = 15;

pub(crate) const ENC_ECU_DEPTH_B: u16 = 8; // for early CU termination

/* count of picture including encoding and reference pictures
0: encoding picture buffer
1: forward reference picture buffer
2: backward reference picture buffer, if exists
3: original (input) picture buffer
4: mode decision picture buffer, if exists
*/
/* current encoding picture buffer index */
pub(crate) const PIC_IDX_CURR: usize = 0;
/* list0 reference picture buffer index */
pub(crate) const PIC_IDX_FORW: usize = 1;
/* list1 reference picture buffer index */
pub(crate) const PIC_IDX_BACK: usize = 2;
/* original (input) picture buffer index */
pub(crate) const PIC_IDX_ORIG: usize = 3;
/* mode decision picture buffer index */
pub(crate) const PIC_IDX_MODE: usize = 4;
pub(crate) const PIC_D: usize = 5;

/* check whether bumping is progress or not */
// FORCE_OUT(ctx)          (self.param.force_output == 1)

/* motion vector accuracy level for inter-mode decision */
pub(crate) const ME_LEV_IPEL: usize = 1;
pub(crate) const ME_LEV_HPEL: usize = 2;
pub(crate) const ME_LEV_QPEL: usize = 3;

/* maximum inbuf count */
pub(crate) const EVCE_MAX_INBUF_CNT: usize = 34;

/* maximum cost value */
pub(crate) const MAX_COST: f64 = (1.7e+308);

/* virtual frame depth B picture */
pub(crate) const FRM_DEPTH_0: u8 = 0;
pub(crate) const FRM_DEPTH_1: u8 = 1;
pub(crate) const FRM_DEPTH_2: u8 = 2;
pub(crate) const FRM_DEPTH_3: u8 = 3;
pub(crate) const FRM_DEPTH_4: u8 = 4;
pub(crate) const FRM_DEPTH_5: u8 = 5;
pub(crate) const FRM_DEPTH_6: u8 = 6;
pub(crate) const FRM_DEPTH_MAX: u8 = 7;
/* I-slice, P-slice, B-slice + depth + 1 (max for GOP 8 size)*/
pub(crate) const LIST_NUM: usize = 1;

pub(crate) const ORG_MAX_NUM_MVP: u8 = 4;

/*****************************************************************************
 * bi-prediction type
 *****************************************************************************/
pub(crate) const BI_NON: u8 = 0;
pub(crate) const BI_NORMAL: u8 = 1;
pub(crate) const BI_ITER: u8 = 4;

/*****************************************************************************
 * original picture buffer structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcePicOrg {
    /* original picture store */
    pic: Rc<RefCell<EvcPic>>,
    /* input picture count */
    pic_icnt: usize,
    /* be used for encoding input */
    is_used: bool,
}

#[derive(Default, Copy, Clone)]
pub(crate) struct EvceDQP {
    prev_QP: u8,
    curr_QP: u8,
    cu_qp_delta_is_coded: bool,
    cu_qp_delta_code: u8,
}

#[derive(Default)]
pub(crate) struct EvceRdoqEst {
    cbf_all: [i64; 2],
    cbf_luma: [i64; 2],
    cbf_cb: [i64; 2],
    cbf_cr: [i64; 2],
    run: [[i32; 2]; NUM_CTX_CC_RUN],
    level: [[i32; 2]; NUM_CTX_CC_LEVEL],
    last: [[i32; 2]; NUM_CTX_CC_LAST],
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
    nb: NBBuffer<pel>, // [N_C][MAX_CU_SIZE*4+1];
    /* current encoding LCU number */
    lcu_num: u16,
    /*QP for current encoding CU. Used to derive Luma and chroma qp*/
    qp: u8,
    cu_qp_delta_code: u8,
    cu_qp_delta_is_coded: bool,
    cu_qp_delta_code_mode: u8,
    qp_prev_eco: u8,
    dqp_curr_best: Vec<Vec<EvceDQP>>, //[[EVCE_DQP; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    dqp_next_best: Vec<Vec<EvceDQP>>, //[[EVCE_DQP; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    dqp_temp_best: EvceDQP,
    dqp_temp_best_merge: EvceDQP,
    dqp_temp_run: EvceDQP,

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
    cu_mode: PredMode,
    /* intra prediction mode */
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
    bs_temp: EvceBsw,
    s_temp_run: EvceSbac,
    c_temp_run: EvcSbacCtx,

    /* SBAC structure for full RDO */
    s_curr_best: [[EvceSbac; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    s_next_best: [[EvceSbac; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    s_temp_best: EvceSbac,
    s_temp_best_merge: EvceSbac,
    s_temp_prev_comp_best: EvceSbac,
    s_temp_prev_comp_run: EvceSbac,
    s_curr_before_split: [[EvceSbac; MAX_CU_DEPTH]; MAX_CU_DEPTH],

    /* SBAC_CTX structures for full RDO */
    c_curr_best: [[EvcSbacCtx; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    c_next_best: [[EvcSbacCtx; MAX_CU_DEPTH]; MAX_CU_DEPTH],
    c_temp_best: EvcSbacCtx,
    c_temp_best_merge: EvcSbacCtx,
    c_temp_prev_comp_best: EvcSbacCtx,
    c_temp_prev_comp_run: EvcSbacCtx,
    c_curr_before_split: [[EvcSbacCtx; MAX_CU_DEPTH]; MAX_CU_DEPTH],

    //EVCE_BEF_DATA  bef_data[MAX_CU_DEPTH][MAX_CU_DEPTH][MAX_CU_CNT_IN_LCU][MAX_BEF_DATA_NUM];
    cost_best: f64,
    inter_satd: u32,
    dist_cu: i32,
    dist_cu_best: i32, //dist of the best intra mode (note: only updated in intra coding now)

    split_mode_child: [bool; 4],
    parent_split_allow: [bool; 6],

    //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
    delta_dist: [i64; N_C], //delta distortion from filtering (negative values mean distortion reduced)
    dist_nofilt: [i64; N_C], //distortion of not filtered samples
    dist_filter: [i64; N_C], //distortion of filtered samples

    /* RDOQ related variables*/
    rdoq_est: EvceRdoqEst,

    evc_tbl_qp_chroma_dynamic_ext: Vec<Vec<i8>>, // [[i8; MAX_QP_TABLE_SIZE_EXT]; 2],
}
impl EvceCore {
    pub(crate) fn new() -> Self {
        let mut evc_tbl_qp_chroma_dynamic_ext = vec![];
        /*if sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            evc_derived_chroma_qp_mapping_tables(
                &sps.chroma_qp_table_struct,
                &mut core.evc_tbl_qp_chroma_dynamic_ext,
            );
        } else*/
        {
            evc_tbl_qp_chroma_dynamic_ext.push(evc_tbl_qp_chroma_ajudst_base.to_vec());
            evc_tbl_qp_chroma_dynamic_ext.push(evc_tbl_qp_chroma_ajudst_base.to_vec());
        }

        let mut cu_data_best = Vec::with_capacity(MAX_CU_DEPTH);
        let mut cu_data_temp = Vec::with_capacity(MAX_CU_DEPTH);
        let mut dqp_data = Vec::with_capacity(MAX_CU_DEPTH);
        let mut dqp_curr_best = Vec::with_capacity(MAX_CU_DEPTH);
        let mut dqp_next_best = Vec::with_capacity(MAX_CU_DEPTH);
        for i in 0..MAX_CU_DEPTH {
            let mut best = Vec::with_capacity(MAX_CU_DEPTH);
            let mut temp = Vec::with_capacity(MAX_CU_DEPTH);
            let mut data = Vec::with_capacity(MAX_CU_DEPTH);
            let mut curr = Vec::with_capacity(MAX_CU_DEPTH);
            let mut next = Vec::with_capacity(MAX_CU_DEPTH);
            for j in 0..MAX_CU_DEPTH {
                best.push(EvceCUData::new(i as u8, j as u8));
                temp.push(EvceCUData::new(i as u8, j as u8));
                data.push(EvceDQP::default());
                curr.push(EvceDQP::default());
                next.push(EvceDQP::default());
            }
            cu_data_best.push(best);
            cu_data_temp.push(temp);
            dqp_data.push(data);
            dqp_curr_best.push(curr);
            dqp_next_best.push(next);
        }

        EvceCore {
            cu_data_best,
            cu_data_temp,
            dqp_data,
            dqp_curr_best,
            dqp_next_best,

            delta_dist: [0; N_C],
            dist_nofilt: [0; N_C],
            dist_filter: [0; N_C],

            evc_tbl_qp_chroma_dynamic_ext,
            ..Default::default()
        }
    }

    fn update_core_loc_param(&mut self, log2_max_cuwh: u8, w_lcu: u16) {
        self.x_pel = self.x_lcu << log2_max_cuwh; // entry point's x location in pixel
        self.y_pel = self.y_lcu << log2_max_cuwh; // entry point's y location in pixel
        self.x_scu = self.x_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2); // set x_scu location
        self.y_scu = self.y_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2); // set y_scu location
        self.lcu_num = self.x_lcu + self.y_lcu * w_lcu; // Init the first lcu_num in tile
    }
}

/******************************************************************************
 * CONTEXT used for encoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
pub(crate) struct EvceCtx {
    /* input frame */
    frm: Option<Frame<pel>>,
    /* output packet */
    pkt: Option<Packet>,

    flush: bool,
    /* address of current input picture, ref_picture  buffer structure */
    pico_buf: Vec<EvcePicOrg>,
    /* index of current input picture buffer in pico_buf[] */
    pico_idx: usize,
    pico_max_cnt: usize,
    gop_size: usize,

    sps_pps_once: bool,

    /* address of core structure */
    core: EvceCore,

    /* address indicating current encoding, list0, list1 and original pictures */
    pic: Vec<Option<Rc<RefCell<EvcPic>>>>, /* the last one is for original */
    /* reference picture (0: foward, 1: backward) */
    refp: Vec<Vec<EvcRefP>>, // Rc<RefCell<Vec<Vec<EvcRefP>>>>  refp[MAX_NUM_REF_PICS][REFP_NUM];
    /* encoding parameter */
    param: EncoderConfig,
    /* SBAC */
    sbac_enc: EvceSbac,
    sbac_ctx: EvcSbacCtx,
    /* debug tracer */
    tracer: Option<Tracer>,
    /* bitstream structure */
    bs: EvceBsw,
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
    pic_icnt: isize,
    /* total input picture count (only used for bumping process) */
    pic_ticnt: usize,
    /* remaining pictures is encoded to p or b slice (only used for bumping process) */
    force_slice: bool,
    /* ignored pictures for force slice count (unavailable pictures cnt in gop,\
    only used for bumping process) */
    force_ignored_cnt: usize,
    /* initial frame return number(delayed input count) due to B picture or Forecast */
    frm_rnum: isize,
    /* current encoding slice number in one picture */
    slice_num: usize,
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
    lcu_cnt: u32,
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
    pic_dbk: Option<Rc<RefCell<EvcPic>>>, //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
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

        let core = EvceCore::new();

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
                log2_max_cuwh - MIN_CU_LOG2 as u8,
                log2_max_cuwh - MIN_CU_LOG2 as u8,
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

        let mut pico_buf = Vec::with_capacity(pico_max_cnt);
        for i in 0..pico_max_cnt {
            pico_buf.push(EvcePicOrg::default());
        }

        EvceCtx {
            frm: None,
            pkt: Some(Packet::default()),

            flush: false,
            pico_buf,
            pico_idx: 0,
            pico_max_cnt,
            gop_size: param.max_b_frames as usize + 1,

            sps_pps_once: false,
            core,
            pic: vec![None; PIC_D + 1],

            refp,
            param,
            /* SBAC */
            sbac_enc: EvceSbac::default(),
            sbac_ctx: EvcSbacCtx::default(),
            tracer: OPEN_TRACE(true),
            /* bitstream structure */
            bs: EvceBsw::default(),
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
            pic_icnt: -1,
            /* total input picture count (only used for bumping process) */
            pic_ticnt: 0,
            /* remaining pictures is encoded to p or b slice (only used for bumping process) */
            force_slice: false,
            /* ignored pictures for force slice count (unavailable pictures cnt in gop,\
            only used for bumping process) */
            force_ignored_cnt: 0,
            /* initial frame return number(delayed input count) due to B picture or Forecast */
            frm_rnum: param.max_b_frames as isize,
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
            pinter: EvcePInter::new(w, h, param.max_b_frames),
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
            pic_dbk: None, //one picture that arranges cu pixels and neighboring pixels for deblocking (just to match the interface of deblocking functions)
            lambda: [0.0; 3],
            sqrt_lambda: [0.0; 3],
            dist_chroma_weight: [0.0; 2],
        }
    }

    pub(crate) fn push_frm(&mut self, frm: &mut Option<Frame<pel>>) -> Result<(), EvcError> {
        self.frm = frm.take();
        Ok(())
    }

    pub(crate) fn encode_frm(&mut self) -> Result<EvcStat, EvcError> {
        if self.frm.is_none() && !self.flush {
            self.flush = true;
            self.pic_ticnt = self.pic_icnt as usize;
        } else {
            if let Some(f) = self.frm.take() {
                self.pic_icnt += 1;
                self.pico_idx = (self.pic_icnt as usize) % self.pico_max_cnt;
                let pico = &mut self.pico_buf[self.pico_idx];
                pico.pic_icnt = self.pic_icnt as usize;
                pico.is_used = true;

                pico.pic.borrow_mut().frame = Rc::new(RefCell::new(f));

                self.pic[PIC_IDX_ORIG] = Some(Rc::clone(&pico.pic));
            }
        }

        /* bumping - check whether input pictures are remaining or not in pico_buf[] */
        self.check_more_frames()?;
        /* store input picture and return if needed */
        self.check_frame_delay()?;

        let pic_cnt = self.pic_icnt - self.frm_rnum;
        self.force_slice = if (self.pic_ticnt % self.gop_size) as isize
            >= (self.pic_ticnt as isize - pic_cnt + 1)
            && self.flush
        {
            true
        } else {
            false
        };

        /* initialize variables for a picture encoding */
        self.evce_enc_pic_prepare()?;

        /* encode one picture */
        self.evce_enc_pic()?;

        /* finishing of encoding a picture */
        self.evce_enc_pic_finish()
    }

    pub(crate) fn pull_pkt(&mut self) -> Result<Rc<RefCell<Packet>>, EvcError> {
        let pkt = self.pkt.take();
        self.pkt = Some(Packet::default());
        if let Some(data) = pkt {
            Ok(Rc::new(RefCell::new(data)))
        } else {
            Err(EvcError::EVC_ERR_EMPTY_PACKET)
        }
    }

    fn check_frame_delay(&self) -> Result<(), EvcError> {
        if self.pic_icnt < self.frm_rnum {
            Err(EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE)
        } else {
            Ok(())
        }
    }

    fn check_more_frames(&mut self) -> Result<(), EvcError> {
        if self.flush {
            /* pseudo evce_push() for bumping process ****************/
            self.pic_icnt += 1;
            /**********************************************************/

            for pico in &self.pico_buf {
                if pico.is_used {
                    return Ok(());
                }
            }

            return Err(EvcError::EVC_OK_NO_MORE_OUTPUT);
        }

        Ok(())
    }

    fn evce_enc_pic_prepare(&mut self) -> Result<(), EvcError> {
        //evc_assert_rv(PIC_ORIG(ctx) != NULL, EVC_ERR_UNEXPECTED);

        self.qp = self.param.qp;

        self.pic[PIC_IDX_CURR] = self.rpm.evc_picman_get_empty_pic()?;
        if let Some(pic) = &self.pic[PIC_IDX_CURR] {
            {
                let p = pic.borrow();
                self.map_refi = Some(Rc::clone(&p.map_refi));
                self.map_mv = Some(Rc::clone(&p.map_mv));
            }

            /*if self.sps.picture_cropping_flag {
                PIC_CURR(ctx)->imgb->crop_idx = 1;
                PIC_CURR(ctx)->imgb->crop_l = self.sps.picture_crop_left_offset;
                PIC_CURR(ctx)->imgb->crop_r = self.sps.picture_crop_right_offset;
                PIC_CURR(ctx)->imgb->crop_t = self.sps.picture_crop_top_offset;
                PIC_CURR(ctx)->imgb->crop_b = self.sps.picture_crop_bottom_offset;
            }*/

            self.pic[PIC_IDX_MODE] = Some(Rc::clone(pic));
        }

        if self.pic_dbk.is_none() {
            self.pic_dbk = Some(Rc::new(RefCell::new(EvcPic::new(
                self.w as usize,
                self.h as usize,
                self.param.chroma_sampling,
            ))));
        }

        self.decide_slice_type();

        if self.slice_type == SliceType::EVC_ST_I {
            if !self.sps_pps_once {
                self.evce_encode_sps();
                self.evce_encode_pps();

                //TODO:
                self.sps_pps_once = true;
            }
        }

        self.lcu_cnt = self.f_lcu;
        self.slice_num = 0;

        if self.slice_type == SliceType::EVC_ST_I {
            self.last_intra_poc = self.poc.poc_val;
        }

        if let (Some(map_refi), Some(map_mv)) = (&mut self.map_refi, &mut self.map_mv) {
            let (mut map_refi, mut map_mv) = (map_refi.borrow_mut(), map_mv.borrow_mut());
            for v in &mut *map_refi {
                v[REFP_0] = -1;
                v[REFP_1] = -1;
            }
            for v in &mut *map_mv {
                v[REFP_0][MV_X] = 0;
                v[REFP_0][MV_Y] = 0;
                v[REFP_1][MV_X] = 0;
                v[REFP_1][MV_Y] = 0;
            }
        }

        /* clear map */
        for v in &mut self.map_scu {
            *v = MCU::default();
        }

        //TODO: support MULTIPLE_NAL?

        Ok(())
    }

    fn evce_enc_pic(&mut self) -> Result<(), EvcError> {
        let split_allow: [bool; 6] = [false, false, false, false, false, true];
        let num_slice_in_pic = self.param.num_slices_in_pic;

        /* initialize bitstream container */
        self.bs.init();
        self.bs.tracer = self.tracer.take();

        for slice_num in 0..num_slice_in_pic {
            self.slice_num = slice_num;

            if self.poc.poc_val > self.last_intra_poc {
                self.last_intra_poc = i32::MAX;
            }
            if self.slice_type == SliceType::EVC_ST_I {
                self.last_intra_poc = self.poc.poc_val;
            }

            /* initialize reference pictures */
            self.rpm.evc_picman_refp_init(
                self.sps.max_num_ref_pics,
                self.slice_type,
                self.poc.poc_val as u32,
                self.nalu.nuh_temporal_id,
                self.last_intra_poc,
                &mut self.refp,
            );

            /* initialize mode decision for frame encoding */
            self.mode_init_frame();

            /* mode analyze frame */
            self.mode_analyze_frame();

            /* slice layer encoding loop */
            {
                let core = &mut self.core;
                core.x_lcu = 0;
                core.y_lcu = 0;
                core.x_pel = 0;
                core.y_pel = 0;
                core.lcu_num = 0;
            }
            self.lcu_cnt = self.f_lcu;

            /* Set nalu header */
            self.nalu.set_nalu(
                if self.pic_cnt == 0
                    || (self.slice_type == SliceType::EVC_ST_I && self.param.closed_gop)
                {
                    NaluType::EVC_IDR_NUT
                } else {
                    NaluType::EVC_NONIDR_NUT
                },
                self.nalu.nuh_temporal_id,
            );

            /* Encode nalu header */
            evce_eco_nalu(&mut self.bs, &self.nalu);

            self.set_sh();
            /* Encode slice header */
            evce_eco_sh(
                &mut self.bs,
                &self.sps,
                &self.pps,
                &self.sh,
                self.nalu.nal_unit_type,
            );

            {
                let core = &mut self.core;
                let sh = &mut self.sh;

                core.qp_y = sh.qp + 6 * (BIT_DEPTH as u8 - 8);
                core.qp_u = (core.evc_tbl_qp_chroma_dynamic_ext[0]
                    [EVC_TBL_CHROMA_QP_OFFSET as usize + sh.qp_u as usize]
                    + 6 * (BIT_DEPTH as i8 - 8)) as u8;
                core.qp_v = (core.evc_tbl_qp_chroma_dynamic_ext[1]
                    [EVC_TBL_CHROMA_QP_OFFSET as usize + sh.qp_v as usize]
                    + 6 * (BIT_DEPTH as i8 - 8)) as u8;

                sh.qp_prev_eco = sh.qp;
                sh.qp_prev_mode = sh.qp;
                core.dqp_data[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
                    .prev_QP = sh.qp_prev_mode;
                core.dqp_curr_best[self.log2_max_cuwh as usize - 2]
                    [self.log2_max_cuwh as usize - 2]
                    .curr_QP = sh.qp;
                core.dqp_curr_best[self.log2_max_cuwh as usize - 2]
                    [self.log2_max_cuwh as usize - 2]
                    .prev_QP = sh.qp;
            }

            self.sbac_enc
                .reset(&mut self.sbac_ctx, self.sh.slice_type, self.sh.qp);
            self.core.s_curr_best[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
                .reset(
                    &mut self.core.c_curr_best[self.log2_max_cuwh as usize - 2]
                        [self.log2_max_cuwh as usize - 2],
                    self.sh.slice_type,
                    self.sh.qp,
                );

            /*Set entry point for each Tile in the tile Slice*/
            //TODO: fix slice-based x/y-LCU
            self.core.x_lcu = 0; //entry point lcu's x location
            self.core.y_lcu = 0; // entry point lcu's y location
            let mut lcu_cnt = self.f_lcu;
            self.core
                .update_core_loc_param(self.log2_max_cuwh, self.w_lcu);

            /* LCU decoding loop */
            loop {
                /* initialize structures *****************************************/
                self.mode_init_lcu();

                /* mode decision *************************************************/
                self.core.s_curr_best[self.log2_max_cuwh as usize - 2]
                    [self.log2_max_cuwh as usize - 2] = self.sbac_enc;
                self.core.c_curr_best[self.log2_max_cuwh as usize - 2]
                    [self.log2_max_cuwh as usize - 2] = self.sbac_ctx;

                self.core.s_curr_best[self.log2_max_cuwh as usize - 2]
                    [self.log2_max_cuwh as usize - 2]
                    .is_bitcount = true;

                /* analyzer lcu */

                // TRACE_RDO = 0: comment this line, otherwise, 2: uncomment it
                self.core.bs_temp.tracer = self.bs.tracer.take();
                self.mode_analyze_lcu();
                self.bs.tracer = self.core.bs_temp.tracer.take();

                /* entropy coding ************************************************/
                self.evce_eco_tree(
                    self.core.x_pel,
                    self.core.y_pel,
                    self.max_cuwh,
                    self.max_cuwh,
                    0,
                    0,
                    true,
                    0,
                    0,
                );

                /* prepare next step *********************************************/
                self.core.x_lcu += 1;
                if self.core.x_lcu >= self.w_lcu {
                    self.core.x_lcu = 0;
                    self.core.y_lcu += 1;
                }

                self.core
                    .update_core_loc_param(self.log2_max_cuwh, self.w_lcu);
                lcu_cnt -= 1;
                self.lcu_cnt -= 1; //To be updated properly in case of multicore

                if lcu_cnt == 0 {
                    evce_eco_tile_end_flag(&mut self.bs, &mut self.sbac_enc, 1);
                    self.sbac_enc.finish(&mut self.bs);
                    break;
                }
            } //End of Slice encoding LCU loop

            /* deblocking filter */
            if self.sh.deblocking_filter_on {
                self.evce_deblock();
            }
        }

        /* de-init BSW */
        self.bs.deinit();
        self.tracer = self.bs.tracer.take();

        /* write the bitstream size */
        self.bs.write_nalu_size();

        /* append bs.pkt to ctx.pkt */
        if let Some(bs_pkt) = self.bs.pkt.take() {
            if let Some(pkt) = &mut self.pkt {
                pkt.data.extend_from_slice(&bs_pkt.data);
            }
        }

        Ok(())
    }

    fn evce_enc_pic_finish(&mut self) -> Result<EvcStat, EvcError> {
        let mut stat = EvcStat::default();

        //TODO: adding picture signature

        /* expand current encoding picture, if needs */
        //self.fn_picbuf_expand(ctx, PIC_CURR(ctx));
        let pic_curr = &self.pic[PIC_IDX_CURR];
        if let Some(pic) = &pic_curr {
            let frame = &pic.borrow().frame;
            frame.borrow_mut().pad();
            stat.rec = Some(Rc::clone(frame));
        }

        /* picture buffer management */
        self.rpm.evc_picman_put_pic(
            pic_curr,
            self.nalu.nal_unit_type == NaluType::EVC_IDR_NUT,
            self.poc.poc_val as u32,
            self.nalu.nuh_temporal_id,
            false,
            &mut self.refp,
            self.slice_ref_flag,
            self.ref_pic_gap_length,
        );

        /* set stat */
        stat.bytes = if let Some(pkt) = &self.pkt {
            pkt.data.len()
        } else {
            0
        };
        stat.nalu_type = if self.slice_type == SliceType::EVC_ST_I {
            NaluType::EVC_IDR_NUT
        } else {
            NaluType::EVC_NONIDR_NUT
        };
        stat.stype = self.slice_type;
        stat.fnum = self.pic_cnt as isize;
        stat.qp = self.sh.qp;
        stat.poc = self.poc.poc_val as isize;
        stat.tid = self.nalu.nuh_temporal_id as isize;

        for i in 0..2 {
            stat.refpic_num[i] = self.rpm.num_refp[i];
            for j in 0..stat.refpic_num[i] as usize {
                stat.refpic[i][j] = self.refp[j][i].poc as isize;
            }
        }

        self.pic_cnt += 1; /* increase picture count */
        //self.param.f_ifrm = 0; /* clear force-IDR flag */ //TODO
        let pico = &mut self.pico_buf[self.pico_idx];
        pico.is_used = false;

        if let (Some(pkt), Some(org), Some(cur)) = (
            &mut self.pkt,
            &self.pic[PIC_IDX_ORIG],
            &self.pic[PIC_IDX_CURR],
        ) {
            let (frame_org, frame_cur) = (&org.borrow().frame, &cur.borrow().frame);
            {
                pkt.ts = frame_org.borrow().ts;
                frame_cur.borrow_mut().ts = pkt.ts;
            }

            {
                let (planes_org, planes_cur) =
                    (&frame_org.borrow().planes, &frame_cur.borrow().planes);
                stat.psnr = Some([
                    calc_psnr(
                        self.w,
                        self.h,
                        self.param.bit_depth,
                        &planes_org[Y_C].as_region(),
                        &planes_cur[Y_C].as_region(),
                    ),
                    calc_psnr(
                        self.w >> 1,
                        self.h >> 1,
                        self.param.bit_depth,
                        &planes_org[U_C].as_region(),
                        &planes_cur[U_C].as_region(),
                    ),
                    calc_psnr(
                        self.w >> 1,
                        self.h >> 1,
                        self.param.bit_depth,
                        &planes_org[V_C].as_region(),
                        &planes_cur[V_C].as_region(),
                    ),
                ]);
            }
        }

        Ok(stat)
    }

    fn decide_slice_type(&mut self) {
        let mut force_cnt = 0;
        let i_period = self.param.max_key_frame_interval as usize;
        let gop_size = self.gop_size;
        let mut pic_icnt = self.pic_cnt + self.param.max_b_frames as usize;
        let mut pic_imcnt = pic_icnt;
        self.pico_idx = pic_icnt % self.pico_max_cnt;
        let pico = &self.pico_buf[self.pico_idx];
        self.pic[PIC_IDX_ORIG] = Some(Rc::clone(&pico.pic));

        if gop_size == 1 {
            if i_period == 1 {
                /* IIII... */
                self.slice_type = SliceType::EVC_ST_I;
                self.slice_depth = FRM_DEPTH_0;
                self.poc.poc_val = pic_icnt as i32;
                self.slice_ref_flag = false;
            } else {
                /* IPPP... */
                pic_imcnt = if i_period > 0 {
                    pic_icnt % i_period
                } else {
                    pic_icnt
                };
                if pic_imcnt == 0 {
                    self.slice_type = SliceType::EVC_ST_I;
                    self.slice_depth = FRM_DEPTH_0;
                    self.poc.poc_val = 0;
                    self.slice_ref_flag = true;
                } else {
                    self.slice_type = self.param.inter_slice_type;

                    if !self.param.disable_hgop {
                        self.slice_depth = tbl_slice_depth_P
                            [self.param.ref_pic_gap_length as usize >> 2]
                            [(pic_imcnt - 1) % self.param.ref_pic_gap_length as usize];
                    } else {
                        self.slice_depth = FRM_DEPTH_1;
                    }
                    self.poc.poc_val = if i_period > 0 {
                        self.pic_cnt % i_period
                    } else {
                        self.pic_cnt
                    } as i32;
                    self.slice_ref_flag = true;
                }
            }
        } else {
            /* include B Picture (gop_size = 2 or 4 or 8 or 16) */
            if pic_icnt == gop_size - 1 {
                /* special case when sequence start */

                self.slice_type = SliceType::EVC_ST_I;
                self.slice_depth = FRM_DEPTH_0;
                self.poc.poc_val = 0;
                self.poc.prev_doc_offset = 0;
                self.poc.prev_poc_val = self.poc.poc_val as u32;
                self.slice_ref_flag = true;

                /* flush the first IDR picture */
                self.pico_idx = 0;
                self.pic[PIC_IDX_ORIG] = Some(Rc::clone(&self.pico_buf[self.pico_idx].pic));
            } else if self.force_slice {
                force_cnt = self.force_ignored_cnt as usize;
                while force_cnt < gop_size {
                    pic_icnt = self.pic_cnt + self.param.max_b_frames as usize + force_cnt;
                    pic_imcnt = pic_icnt;

                    self.decide_normal_gop(pic_imcnt);

                    if self.poc.poc_val <= self.pic_ticnt as i32 {
                        break;
                    }
                    force_cnt += 1;
                }
                self.force_ignored_cnt = force_cnt;
            } else {
                /* normal GOP case */
                self.decide_normal_gop(pic_imcnt);
            }
        }
        if !self.param.disable_hgop && gop_size > 1 {
            self.nalu.nuh_temporal_id = self.slice_depth - if self.slice_depth > 0 { 1 } else { 0 };
        } else {
            self.nalu.nuh_temporal_id = 0;
        }
    }

    fn decide_normal_gop(&mut self, pic_imcnt: usize) {
        let i_period = self.param.max_key_frame_interval;
        let gop_size = self.gop_size;

        if i_period == 0 && pic_imcnt == 0 {
            self.slice_type = SliceType::EVC_ST_I;
            self.slice_depth = FRM_DEPTH_0;
            self.poc.poc_val = pic_imcnt as i32;
            self.poc.prev_doc_offset = 0;
            self.poc.prev_poc_val = self.poc.poc_val as u32;
            self.slice_ref_flag = true;
        } else if (i_period != 0) && pic_imcnt % i_period == 0 {
            self.slice_type = SliceType::EVC_ST_I;
            self.slice_depth = FRM_DEPTH_0;
            self.poc.poc_val = pic_imcnt as i32;
            self.poc.prev_doc_offset = 0;
            self.poc.prev_poc_val = self.poc.poc_val as u32;
            self.slice_ref_flag = true;
        } else if pic_imcnt % gop_size == 0 {
            self.slice_type = self.param.inter_slice_type;
            self.slice_depth = FRM_DEPTH_1;
            self.poc.poc_val = pic_imcnt as i32;
            self.poc.prev_doc_offset = 0;
            self.poc.prev_poc_val = self.poc.poc_val as u32;
            self.slice_ref_flag = true;
        } else {
            self.slice_type = self.param.inter_slice_type;
            if !self.param.disable_hgop {
                let pos = (pic_imcnt % gop_size) - 1;

                self.slice_depth = tbl_slice_depth[gop_size >> 2][pos];
                let tid = self.slice_depth - if self.slice_depth > 0 { 1 } else { 0 };
                evc_poc_derivation(&self.sps, tid, &mut self.poc);

                if gop_size >= 2 {
                    self.slice_ref_flag =
                        if self.slice_depth == tbl_slice_depth[gop_size >> 2][gop_size - 2] {
                            false
                        } else {
                            true
                        };
                } else {
                    self.slice_ref_flag = true;
                }
            } else {
                let pos = (pic_imcnt % gop_size) - 1;
                self.slice_depth = FRM_DEPTH_2;
                self.poc.poc_val =
                    (((pic_imcnt / gop_size) * gop_size) - gop_size + pos + 1) as i32;
                self.slice_ref_flag = false;
            }
            /* find current encoding picture's(B picture) pic_icnt */
            let pic_icnt_b = self.poc.poc_val;

            /* find pico again here */
            self.pico_idx = pic_icnt_b as usize % self.pico_max_cnt;
            let pico = &self.pico_buf[self.pico_idx];

            self.pic[PIC_IDX_ORIG] = Some(Rc::clone(&pico.pic));
        }
    }

    fn set_sps(&mut self) {
        let sps = &mut self.sps;
        sps.profile_idc = 0; // baseline profile only
        sps.level_idc = self.param.level * 3;
        sps.pic_width_in_luma_samples = self.param.width as u16;
        sps.pic_height_in_luma_samples = self.param.height as u16;
        sps.toolset_idc_h = 0;
        sps.toolset_idc_l = 0;
        sps.bit_depth_luma_minus8 = 0; //TODO: self.param.out_bit_depth - 8;
        sps.bit_depth_chroma_minus8 = 0; //TODO: self.cdsc.out_bit_depth - 8;
        sps.chroma_format_idc = 1; // YCbCr 4:2:0
        if self.param.max_b_frames > 0 {
            sps.max_num_ref_pics = MAX_NUM_ACTIVE_REF_FRAME_B;
        } else {
            sps.max_num_ref_pics = MAX_NUM_ACTIVE_REF_FRAME_LDB;
        }
        sps.sps_btt_flag = false;
        sps.sps_suco_flag = false;
        sps.tool_amvr = false;
        sps.tool_mmvd = false;
        sps.tool_affine = false;
        sps.tool_dmvr = false;
        sps.tool_addb = false;
        sps.tool_dra = false;
        sps.tool_alf = false;
        sps.tool_htdf = false;
        sps.tool_admvp = false;
        sps.tool_hmvp = false;
        sps.tool_eipd = false;
        sps.tool_iqt = false;
        sps.tool_adcc = false;
        sps.tool_cm_init = false;
        sps.tool_ats = false;
        sps.tool_rpl = false;
        sps.tool_pocs = false;

        sps.log2_sub_gop_length = ((self.gop_size as f32).log2() + 0.5f32) as u8;
        self.ref_pic_gap_length = self.param.ref_pic_gap_length as u32;
        sps.log2_ref_pic_gap_length =
            ((self.param.ref_pic_gap_length as f32).log2() + 0.5f32) as u8;

        sps.vui_parameters_present_flag = false;
        sps.dquant_flag = false; /*Baseline : Active SPSs shall have sps_dquant_flag equal to 0 only*/

        //if (self.cdsc.chroma_qp_table_struct.chroma_qp_table_present_flag)
        //{
        //    evce_copy_chroma_qp_mapping_params(&(sps.chroma_qp_table_struct), &(self.cdsc.chroma_qp_table_struct));
        //}

        sps.picture_cropping_flag = false; //self.cdsc.picture_cropping_flag;
                                           /*if (sps.picture_cropping_flag)
                                           {
                                               sps.picture_crop_left_offset = self.cdsc.picture_crop_left_offset;
                                               sps.picture_crop_right_offset = self.cdsc.picture_crop_right_offset;
                                               sps.picture_crop_top_offset = self.cdsc.picture_crop_top_offset;
                                               sps.picture_crop_bottom_offset = self.cdsc.picture_crop_bottom_offset;
                                           }*/
    }

    fn set_pps(&mut self) {
        let pps = &mut self.pps;

        pps.single_tile_in_pic_flag = true;
        pps.constrained_intra_pred_flag = self.param.enable_cip;
        pps.cu_qp_delta_enabled_flag = false; //self.param.use_dqp;
        pps.cu_qp_delta_area = self.param.cu_qp_delta_area;
        pps.single_tile_in_pic_flag = true;
        pps.arbitrary_slice_present_flag = false;
        pps.tile_id_len_minus1 = 0;
        pps.num_ref_idx_default_active_minus1[REFP_0] = 0; /* To be checked */
        pps.num_ref_idx_default_active_minus1[REFP_1] = 0; /* To be checked */
    }

    fn set_sh(&mut self) {
        let sh = &mut self.sh;

        let qp_adapt_param = if self.param.max_b_frames == 0 {
            if self.param.max_key_frame_interval == 1 {
                &qp_adapt_param_ai
            } else {
                &qp_adapt_param_ld
            }
        } else {
            &qp_adapt_param_ra
        };

        sh.slice_type = self.slice_type;
        sh.no_output_of_prior_pics_flag = false;
        sh.deblocking_filter_on = if self.param.disable_dbf { false } else { true };

        /* set lambda */
        let mut qp = self.qp as i8; //EVC_CLIP3(0, MAX_QUANT, (self.param.qp_incread_frame != 0 && (int)(self.poc.poc_val) >= self.param.qp_incread_frame) ? self.qp + 1.0 : self.qp);

        if !self.param.disable_hgop {
            qp += qp_adapt_param[self.slice_depth as usize].qp_offset_layer;
            let dqp_offset = qp as f64
                * qp_adapt_param[self.slice_depth as usize].qp_offset_model_scale
                + qp_adapt_param[self.slice_depth as usize].qp_offset_model_offset
                + 0.5;

            let qp_offset = EVC_CLIP3(0.0, 3.0, dqp_offset).floor() as i8;
            qp += qp_offset;
        }

        sh.qp = EVC_CLIP3(0, MAX_QUANT as i8, qp) as u8;
        sh.qp_u_offset = self.param.cb_qp_offset;
        sh.qp_v_offset = self.param.cr_qp_offset;
        sh.qp_u = EVC_CLIP3(-6 * (BIT_DEPTH as i8 - 8), 57, sh.qp as i8 + sh.qp_u_offset) as u8;
        sh.qp_v = EVC_CLIP3(-6 * (BIT_DEPTH as i8 - 8), 57, sh.qp as i8 + sh.qp_v_offset) as u8;

        let qp_l_i = sh.qp as i8;
        self.lambda[0] = 0.57 * (2.0f64).powf((qp_l_i - 12) as f64 / 3.0);
        let qp_c_i = self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [EVC_TBL_CHROMA_QP_OFFSET as usize + sh.qp_u as usize];
        self.dist_chroma_weight[0] = (2.0f64).powf((qp_l_i - qp_c_i) as f64 / 3.0);
        let qp_c_i = self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [EVC_TBL_CHROMA_QP_OFFSET as usize + sh.qp_v as usize];
        self.dist_chroma_weight[1] = (2.0f64).powf((qp_l_i - qp_c_i) as f64 / 3.0);
        self.lambda[1] = self.lambda[0] / self.dist_chroma_weight[0];
        self.lambda[2] = self.lambda[0] / self.dist_chroma_weight[1];
        self.sqrt_lambda[0] = self.lambda[0].sqrt();
        self.sqrt_lambda[1] = self.lambda[1].sqrt();
        self.sqrt_lambda[2] = self.lambda[2].sqrt();
    }

    fn evce_eco_tree(
        &mut self,
        x0: u16,
        y0: u16,
        cuw: u16,
        cuh: u16,
        cup: u16,
        cud: u16,
        next_split: bool,
        qt_depth: u8,
        mut cu_qp_delta_code: u8,
    ) {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_enc;
        let sbac_ctx = &mut self.sbac_ctx;

        let split_mode = evc_get_split_mode(
            cud,
            cup,
            cuw,
            cuh,
            self.max_cuwh,
            &self.map_cu_data[core.lcu_num as usize].split_mode,
        );

        //same_layer_split[node_idx] = split_mode;

        if self.pps.cu_qp_delta_enabled_flag && self.sps.dquant_flag {
            if split_mode == SplitMode::NO_SPLIT
                && (CONV_LOG2(cuw as usize) + CONV_LOG2(cuh as usize) >= self.pps.cu_qp_delta_area)
                && cu_qp_delta_code != 2
            {
                if CONV_LOG2(cuw as usize) == 7 || CONV_LOG2(cuh as usize) == 7 {
                    cu_qp_delta_code = 2;
                } else {
                    cu_qp_delta_code = 1;
                }
                core.cu_qp_delta_is_coded = false;
            } else if CONV_LOG2(cuh as usize) + CONV_LOG2(cuw as usize) == self.pps.cu_qp_delta_area
                && cu_qp_delta_code != 2
            {
                cu_qp_delta_code = 2;
                core.cu_qp_delta_is_coded = false;
            }
        }

        if split_mode != SplitMode::NO_SPLIT {
            evce_eco_split_mode(
                bs,
                sbac,
                sbac_ctx,
                x0,
                y0,
                cud,
                cup,
                cuw,
                cuh,
                self.max_cuwh,
                &self.map_cu_data[core.lcu_num as usize].split_mode,
            );

            let split_struct = evc_split_get_part_structure(
                split_mode,
                x0,
                y0,
                cuw,
                cuh,
                cup,
                cud,
                self.log2_culine,
            );

            for cur_part_num in 0..split_struct.part_count {
                let sub_cuw = split_struct.width[cur_part_num];
                let sub_cuh = split_struct.height[cur_part_num];
                let x_pos = split_struct.x_pos[cur_part_num];
                let y_pos = split_struct.y_pos[cur_part_num];

                if x_pos < self.w && y_pos < self.h {
                    self.evce_eco_tree(
                        x_pos,
                        y_pos,
                        sub_cuw,
                        sub_cuh,
                        split_struct.cup[cur_part_num],
                        split_struct.cud[cur_part_num],
                        true,
                        split_mode.inc_qt_depth(qt_depth),
                        cu_qp_delta_code,
                    );
                }
            }
        } else {
            assert!(x0 + cuw <= self.w && y0 + cuh <= self.h);

            if (cuw > MIN_CU_SIZE as u16 || cuh > MIN_CU_SIZE as u16) && next_split {
                evce_eco_split_mode(
                    bs,
                    sbac,
                    sbac_ctx,
                    x0,
                    y0,
                    cud,
                    cup,
                    cuw,
                    cuh,
                    self.max_cuwh,
                    &self.map_cu_data[core.lcu_num as usize].split_mode,
                );
            }

            core.cu_qp_delta_code = cu_qp_delta_code;
            self.evce_eco_unit(x0, y0, cup as usize, cuw, cuh);
        }
    }

    fn evce_eco_unit(&mut self, x: u16, y: u16, cup: usize, cuw: u16, cuh: u16) {
        let enc_dqp = 0;
        let slice_type = self.slice_type;

        self.cu_init(x, y, cup, cuw, cuh);

        EVC_TRACE_COUNTER(&mut self.bs.tracer);
        EVC_TRACE(&mut self.bs.tracer, "poc: ");
        EVC_TRACE(&mut self.bs.tracer, self.poc.poc_val);
        EVC_TRACE(&mut self.bs.tracer, " x pos ");
        EVC_TRACE(
            &mut self.bs.tracer,
            self.core.x_pel + ((cup as u16 % (self.max_cuwh >> MIN_CU_LOG2)) << MIN_CU_LOG2),
        );
        EVC_TRACE(&mut self.bs.tracer, " y pos ");
        EVC_TRACE(
            &mut self.bs.tracer,
            self.core.y_pel + ((cup as u16 / (self.max_cuwh as u16 >> MIN_CU_LOG2)) << MIN_CU_LOG2),
        );
        EVC_TRACE(&mut self.bs.tracer, " width ");
        EVC_TRACE(&mut self.bs.tracer, cuw);
        EVC_TRACE(&mut self.bs.tracer, " height ");
        EVC_TRACE(&mut self.bs.tracer, cuh);
        EVC_TRACE(&mut self.bs.tracer, " \n");

        {
            let core = &mut self.core;
            let cu_data = &mut self.map_cu_data[core.lcu_num as usize];
            let bs = &mut self.bs;
            let sbac = &mut self.sbac_enc;
            let sbac_ctx = &mut self.sbac_ctx;

            if !core.skip_flag {
                /* get coefficients and tq */
                coef_rect_to_series(
                    &mut core.ctmp,
                    &cu_data.coef,
                    self.log2_max_cuwh,
                    x,
                    y,
                    cuw,
                    cuh,
                );

                for i in 0..N_C {
                    core.nnz[i] = cu_data.nnz[i][cup as usize];
                }
            } else {
                for i in 0..N_C {
                    core.nnz[i] = 0;
                }
            }

            /* entropy coding a CU */
            if slice_type != SliceType::EVC_ST_I {
                evce_eco_skip_flag(bs, sbac, sbac_ctx, core.skip_flag as u32);

                if core.skip_flag {
                    evce_eco_mvp_idx(
                        bs,
                        sbac,
                        sbac_ctx,
                        cu_data.mvp_idx[cup as usize][REFP_0] as u32,
                    );

                    if slice_type == SliceType::EVC_ST_B {
                        evce_eco_mvp_idx(
                            bs,
                            sbac,
                            sbac_ctx,
                            cu_data.mvp_idx[cup as usize][REFP_1] as u32,
                        );
                    }
                } else {
                    evce_eco_pred_mode(bs, sbac, sbac_ctx, core.cu_mode);

                    if core.cu_mode != PredMode::MODE_INTRA {
                        evce_eco_direct_mode_flag(
                            bs,
                            sbac,
                            sbac_ctx,
                            if cu_data.pred_mode[cup as usize] == PredMode::MODE_DIR {
                                1
                            } else {
                                0
                            },
                        );

                        if cu_data.pred_mode[cup as usize] != PredMode::MODE_DIR {
                            evce_eco_inter_pred_idc(
                                bs,
                                sbac,
                                sbac_ctx,
                                &cu_data.refi[cup as usize],
                                slice_type,
                            );

                            let refi0 = cu_data.refi[cup as usize][REFP_0];
                            let refi1 = cu_data.refi[cup as usize][REFP_1];
                            if slice_type.IS_INTER_SLICE() && REFI_IS_VALID(refi0) {
                                evce_eco_refi(bs, sbac, sbac_ctx, self.rpm.num_refp[REFP_0], refi0);
                                evce_eco_mvp_idx(
                                    bs,
                                    sbac,
                                    sbac_ctx,
                                    cu_data.mvp_idx[cup as usize][REFP_0] as u32,
                                );
                                evce_eco_mvd(
                                    bs,
                                    sbac,
                                    sbac_ctx,
                                    &cu_data.mvd[cup as usize][REFP_0],
                                );
                            }

                            if slice_type == SliceType::EVC_ST_B && REFI_IS_VALID(refi1) {
                                evce_eco_refi(bs, sbac, sbac_ctx, self.rpm.num_refp[REFP_1], refi1);
                                evce_eco_mvp_idx(
                                    bs,
                                    sbac,
                                    sbac_ctx,
                                    cu_data.mvp_idx[cup as usize][REFP_1] as u32,
                                );
                                evce_eco_mvd(
                                    bs,
                                    sbac,
                                    sbac_ctx,
                                    &cu_data.mvd[cup as usize][REFP_1],
                                );
                            }
                        }
                    }
                }
            }

            if core.cu_mode == PredMode::MODE_INTRA {
                assert_ne!(
                    cu_data.ipm[0][cup as usize] as i8,
                    IntraPredDir::IPD_INVALID as i8
                );
                assert_ne!(
                    cu_data.ipm[1][cup as usize] as i8,
                    IntraPredDir::IPD_INVALID as i8
                );

                core.mpm_b_list = evc_get_mpm_b(
                    core.x_scu,
                    core.y_scu,
                    &self.map_scu,
                    &self.map_ipm,
                    core.scup,
                    self.w_scu,
                );

                evce_eco_intra_dir_b(
                    bs,
                    sbac,
                    sbac_ctx,
                    cu_data.ipm[0][cup] as u8,
                    core.mpm_b_list,
                );
            }
        }

        if !self.core.skip_flag {
            evce_eco_coef(
                &mut self.bs,
                &mut self.sbac_enc,
                &mut self.sbac_ctx,
                &self.core.ctmp,
                self.core.log2_cuw,
                self.core.log2_cuh,
                self.core.cu_mode,
                &self.core.nnz,
                false,
                TQC_RUN::RUN_L as u8 | TQC_RUN::RUN_CB as u8 | TQC_RUN::RUN_CR as u8,
                true,
                self.map_cu_data[self.core.lcu_num as usize].qp_y[cup] - 6 * (BIT_DEPTH as u8 - 8),
                self.sps.dquant_flag,
                self.pps.cu_qp_delta_enabled_flag,
                self.core.cu_qp_delta_code,
                &mut self.core.cu_qp_delta_is_coded,
                &mut self.core.qp_prev_eco,
            );
        }

        self.evce_set_enc_info();
    }

    fn cu_init(&mut self, x: u16, y: u16, cup: usize, cuw: u16, cuh: u16) {
        let core = &mut self.core;
        let cu_data = &mut self.map_cu_data[core.lcu_num as usize];

        core.cuw = cuw;
        core.cuh = cuh;
        core.log2_cuw = CONV_LOG2(cuw as usize);
        core.log2_cuh = CONV_LOG2(cuh as usize);
        core.x_scu = PEL2SCU(x as usize) as u16;
        core.y_scu = PEL2SCU(y as usize) as u16;
        core.scup = (core.y_scu as u32 * self.w_scu as u32) + core.x_scu as u32;
        core.avail_cu = 0;
        core.skip_flag = false;
        core.nnz[Y_C] = 0;
        core.nnz[U_C] = 0;
        core.nnz[V_C] = 0;
        core.cu_mode = cu_data.pred_mode[cup as usize];

        if core.cu_mode == PredMode::MODE_INTRA {
            core.avail_cu = evc_get_avail_intra(
                core.x_scu as usize,
                core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                core.scup as usize,
                core.log2_cuw,
                core.log2_cuh,
                &self.map_scu,
            );
        } else {
            if cu_data.pred_mode[cup as usize] == PredMode::MODE_SKIP {
                core.skip_flag = true;
            }

            core.avail_cu = evc_get_avail_inter(
                core.x_scu as usize,
                core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                core.scup as usize,
                core.cuw as usize,
                core.cuh as usize,
                &self.map_scu,
            );
        }

        core.avail_lr = evc_check_nev_avail(core.x_scu, core.y_scu, cuw, self.w_scu, &self.map_scu);
    }

    fn evce_set_enc_info(&mut self) {
        let w_scu = self.w_scu as usize;
        let scup = self.core.scup as usize;
        let w_cu = (1 << self.core.log2_cuw as usize) >> MIN_CU_LOG2;
        let h_cu = (1 << self.core.log2_cuh as usize) >> MIN_CU_LOG2;

        for i in 0..h_cu {
            let map_scu = &mut self.map_scu[scup + i * w_scu..];

            for j in 0..w_cu {
                if self.core.cu_mode == PredMode::MODE_SKIP {
                    map_scu[j].SET_SF();
                } else {
                    map_scu[j].CLR_SF();
                }
                if self.core.nnz[Y_C] > 0 {
                    map_scu[j].SET_CBFL();
                } else {
                    map_scu[j].CLR_CBFL();
                }

                map_scu[j].SET_COD();

                if self.pps.cu_qp_delta_enabled_flag {
                    map_scu[j].RESET_QP();
                    map_scu[j].SET_QP(self.core.qp_prev_eco as u32);
                }
            }
        }
    }

    fn evce_encode_sps(&mut self) {
        /* bitsteam initialize for sequence */
        self.bs.init();
        self.bs.tracer = self.tracer.take();

        /* nalu header */
        self.nalu.set_nalu(NaluType::EVC_SPS_NUT, 0);

        evce_eco_nalu(&mut self.bs, &self.nalu);

        /* sequence parameter set*/
        self.set_sps();
        evce_eco_sps(&mut self.bs, &self.sps);

        /* de-init BSW */
        self.bs.deinit();
        self.tracer = self.bs.tracer.take();

        /* write the bitstream size */
        self.bs.write_nalu_size();

        /* append bs.pkt to ctx.pkt */
        if let Some(bs_pkt) = self.bs.pkt.take() {
            if let Some(pkt) = &mut self.pkt {
                pkt.data.extend_from_slice(&bs_pkt.data);
            }
        }
    }

    fn evce_encode_pps(&mut self) {
        /* bitsteam initialize for sequence */
        self.bs.init();
        self.bs.tracer = self.tracer.take();

        /* nalu header */
        self.nalu
            .set_nalu(NaluType::EVC_PPS_NUT, self.nalu.nuh_temporal_id);

        evce_eco_nalu(&mut self.bs, &self.nalu);

        /* sequence parameter set*/
        self.set_pps();
        evce_eco_pps(&mut self.bs, &self.sps, &self.pps);

        /* de-init BSW */
        self.bs.deinit();
        self.tracer = self.bs.tracer.take();

        /* write the bitstream size */
        self.bs.write_nalu_size();

        /* append bs.pkt to ctx.pkt */
        if let Some(bs_pkt) = self.bs.pkt.take() {
            if let Some(pkt) = &mut self.pkt {
                pkt.data.extend_from_slice(&bs_pkt.data);
            }
        }
    }

    fn deblock_tree(
        &mut self,
        x: u16,
        y: u16,
        cuw: u16,
        cuh: u16,
        cud: u16,
        cup: u16,
        is_hor_edge: bool,
    ) {
        let lcu_num = (x >> self.log2_max_cuwh) + (y >> self.log2_max_cuwh) * self.w_lcu;
        let split_mode = evc_get_split_mode(
            cud,
            cup,
            cuw,
            cuh,
            self.max_cuwh,
            &self.map_cu_data[lcu_num as usize].split_mode,
        );

        /*EVC_TRACE_COUNTER(&mut self.bs.tracer);
        EVC_TRACE(&mut self.bs.tracer, "split_mod ");
        EVC_TRACE(&mut self.bs.tracer, split_mode as u8);
        EVC_TRACE(&mut self.bs.tracer, " \n");*/

        if split_mode != SplitMode::NO_SPLIT {
            let split_struct = evc_split_get_part_structure(
                split_mode,
                x,
                y,
                cuw,
                cuh,
                cup,
                cud,
                self.log2_max_cuwh - MIN_CU_LOG2 as u8,
            );

            // In base profile we have small chroma blocks
            for part_num in 0..split_struct.part_count {
                let cur_part_num = part_num;
                let sub_cuw = split_struct.width[cur_part_num];
                let sub_cuh = split_struct.height[cur_part_num];
                let x_pos = split_struct.x_pos[cur_part_num];
                let y_pos = split_struct.y_pos[cur_part_num];

                if x_pos < self.w && y_pos < self.h {
                    self.deblock_tree(
                        x_pos,
                        y_pos,
                        sub_cuw,
                        sub_cuh,
                        split_struct.cud[cur_part_num],
                        split_struct.cup[cur_part_num],
                        is_hor_edge,
                    );
                }
            }
        } else if let (Some(pic), Some(map_refi), Some(map_mv)) =
            (&self.pic[PIC_IDX_MODE], &self.map_refi, &self.map_mv)
        {
            // deblock
            if is_hor_edge {
                if cuh > MAX_TR_SIZE as u16 {
                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize >> 1,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );

                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize + MAX_TR_SIZE,
                        cuw as usize,
                        cuh as usize >> 1,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );
                } else {
                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );
                }
            } else {
                if cuw > MAX_TR_SIZE as u16 {
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize >> 1,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize + MAX_TR_SIZE,
                        y as usize,
                        cuw as usize >> 1,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                } else {
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                }
            }
        }
    }

    fn evce_deblock(&mut self) {
        if let Some(pic) = &self.pic[PIC_IDX_MODE] {
            let mut p = pic.borrow_mut();
            p.pic_qp_u_offset = self.sh.qp_u_offset;
            p.pic_qp_v_offset = self.sh.qp_v_offset;
        }

        let scu_in_lcu_wh = 1 << (self.log2_max_cuwh - MIN_CU_LOG2 as u8);

        let x_l = 0; //entry point lcu's x location
        let y_l = 0; // entry point lcu's y location
        let x_r = x_l + self.w_lcu;
        let y_r = y_l + self.h_lcu;
        let l_scu = x_l * scu_in_lcu_wh;
        let r_scu = EVC_CLIP3(0, self.w_scu, x_r * scu_in_lcu_wh);
        let t_scu = y_l * scu_in_lcu_wh;
        let b_scu = EVC_CLIP3(0, self.h_scu, y_r * scu_in_lcu_wh);

        for j in t_scu..b_scu {
            for i in l_scu..r_scu {
                self.map_scu[(i + j * self.w_scu) as usize].CLR_COD();
            }
        }

        /* horizontal filtering */
        for j in y_l..y_r {
            for i in x_l..x_r {
                self.deblock_tree(
                    (i << self.log2_max_cuwh),
                    (j << self.log2_max_cuwh),
                    self.max_cuwh,
                    self.max_cuwh,
                    0,
                    0,
                    false, /*horizontal filtering of vertical edge*/
                );
            }
        }

        for j in t_scu..b_scu {
            for i in l_scu..r_scu {
                self.map_scu[(i + j * self.w_scu) as usize].CLR_COD();
            }
        }

        /* vertical filtering */
        for j in y_l..y_r {
            for i in x_l..x_r {
                self.deblock_tree(
                    (i << self.log2_max_cuwh),
                    (j << self.log2_max_cuwh),
                    self.max_cuwh,
                    self.max_cuwh,
                    0,
                    0,
                    true, /*vertical filtering of horizontal edge*/
                );
            }
        }
    }
}
