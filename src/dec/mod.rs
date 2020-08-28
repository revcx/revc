use super::api::frame::*;
use super::api::*;
use super::def::*;
use super::df::*;
use super::hawktracer::*;
use super::ipred::*;
use super::itdq::*;
use super::mc::*;
use super::picman::*;
use super::recon::*;
use super::tbl::*;
use super::tracer::*;
use super::util::*;

use std::cell::RefCell;
use std::rc::Rc;

mod bsr;
mod eco;
mod sbac;

use bsr::*;
use eco::*;
use sbac::*;

/* evc decoder magic code */
pub(crate) const EVCD_MAGIC_CODE: u32 = 0x45565944; /* EVYD */

/*****************************************************************************
 * CORE information used for decoding process.
 *
 * The variables in this structure are very often used in decoding process.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCore {
    /************** current CU **************/
    /* coefficient buffer of current CU */
    coef: CUBuffer<i16>, //[[i16; MAX_CU_DIM]; N_C], //[N_C][MAX_CU_DIM]
    /* pred buffer of current CU */
    /* [1] is used for bi-pred. */
    pred: [CUBuffer<pel>; 2], //[[[pel; MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]

    /* neighbor pixel buffer for intra prediction */
    nb: NBBuffer<pel>, // [N_C][N_REF][MAX_CU_SIZE * 3];
    /* reference index for current CU */
    refi: [i8; REFP_NUM],
    /* motion vector for current CU */
    mv: [[i16; MV_D]; REFP_NUM],

    /* CU position in current frame in SCU unit */
    scup: u32,
    /* CU position X in a frame in SCU unit */
    x_scu: u16,
    /* CU position Y in a frame in SCU unit */
    y_scu: u16,
    /* neighbor CUs availability of current CU */
    avail_cu: u16,
    /* Left, right availability of current CU */
    avail_lr: u16,
    /* intra prediction direction of current CU */
    ipm: [IntraPredDir; 2],
    /* most probable mode for intra prediction */
    mpm_b_list: &'static [u8],
    //mpm: [u8; 2],
    //mpm_ext: [u8; 8],
    //pims: [IntraPredDir; IntraPredDir::IPD_CNT_B as usize], /* probable intra mode set*/
    /* prediction mode of current CU: INTRA, INTER, ... */
    pred_mode: PredMode,
    DMVRenable: u8,
    /* log2 of cuw */
    log2_cuw: u8,
    /* log2 of cuh */
    log2_cuh: u8,
    /* is there coefficient? */
    is_coef: [bool; N_C],

    /* QP for Luma of current encoding MB */
    qp_y: u8,
    /* QP for Chroma of current encoding MB */
    qp_u: u8,
    qp_v: u8,

    qp: u8,
    cu_qp_delta_code: u8,
    cu_qp_delta_is_coded: bool,

    /************** current LCU *************/
    /* address of current LCU,  */
    lcu_num: u16,
    /* X address of current LCU */
    x_lcu: u16,
    /* Y address of current LCU */
    y_lcu: u16,
    /* left pel position of current LCU */
    x_pel: u16,
    /* top pel position of current LCU */
    y_pel: u16,
    /* split mode map for current LCU */
    split_mode: LcuSplitMode,

    mvp_idx: [u8; REFP_NUM],
    mvd: [[i16; MV_D]; REFP_NUM],
    inter_dir: InterPredDir,
    ctx_flags: [u8; CtxNevIdx::NUM_CNID as usize],
    tree_cons: TREE_CONS,

    evc_tbl_qp_chroma_dynamic_ext: Vec<Vec<i8>>, // [[i8; MAX_QP_TABLE_SIZE_EXT]; 2],
}

/******************************************************************************
 * CONTEXT used for decoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
pub(crate) struct EvcdCtx {
    /* magic code */
    magic: u32,

    /* input packet */
    pkt: Option<Packet>,

    /* EVCD identifier */
    //EVCD                    id;
    /* CORE information used for fast operation */
    core: EvcdCore,
    /* current decoding bitstream */
    bs: EvcdBsr,
    /* current nalu header */
    nalu: EvcNalu,
    /* current slice header */
    sh: EvcSh,
    /* decoded picture buffer management */
    dpm: Option<EvcPm>,
    /* create descriptor */
    //EVCD_CDSC               cdsc;
    /* sequence parameter set */
    sps: EvcSps,
    /* picture parameter set */
    pps: EvcPps,
    /* current decoded (decoding) picture buffer */
    pic: Option<Rc<RefCell<EvcPic>>>,
    /* SBAC */
    sbac_dec: EvcdSbac,
    sbac_ctx: EvcSbacCtx,
    /* decoding picture width */
    w: u16,
    /* decoding picture height */
    h: u16,
    /* decoding chroma sampling */
    cs: ChromaSampling,
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
    map_scu: Vec<MCU>,
    /* LCU split information */
    map_split: Vec<LcuSplitMode>,
    /* decoded motion vector for every blocks */
    map_mv: Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    /* reference frame indices */
    map_refi: Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    /* intra prediction modes */
    map_ipm: Vec<IntraPredDir>,
    /* new coding tool flag*/
    map_cu_mode: Vec<MCU>,
    /**************************************************************************/
    /* current slice number, which is increased whenever decoding a slice.
    when receiving a slice for new picture, this value is set to zero.
    this value can be used for distinguishing b/w slices */
    slice_num: u16,
    /* last coded intra picture's picture order count */
    last_intra_poc: i32,
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
    poc: EvcPoc,
    /* the picture order count of the previous Tid0 picture */
    prev_pic_order_cnt_val: u32,
    /* the decoding order count of the previous picture */
    prev_doc_offset: u32,
    /* the number of currently decoded pictures */
    pic_cnt: u32,
    /* flag whether current picture is refecened picture or not */
    slice_ref_flag: bool,
    /* distance between ref pics in addition to closest ref ref pic in LD*/
    ref_pic_gap_length: u32,
    /* bitstream has an error? */
    bs_err: u8,
    /* reference picture (0: foward, 1: backward) */
    refp: Vec<Vec<EvcRefP>>, //[[EvcRefP; REFP_NUM]; MAX_NUM_REF_PICS],
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

impl EvcdCtx {
    pub(crate) fn new(cfg: &Config) -> Self {
        let mut refp = Vec::with_capacity(MAX_NUM_REF_PICS);
        for j in 0..MAX_NUM_REF_PICS {
            let mut refp1d = Vec::with_capacity(REFP_NUM);
            for i in 0..REFP_NUM {
                refp1d.push(EvcRefP::new());
            }
            refp.push(refp1d);
        }

        EvcdCtx {
            magic: EVCD_MAGIC_CODE,
            pkt: None,

            /* EVCD identifier */
            //EVCD                    id;
            /* CORE information used for fast operation */
            core: EvcdCore::default(),
            /* current decoding bitstream */
            bs: EvcdBsr::default(),
            /* current nalu header */
            nalu: EvcNalu::default(),
            /* current slice header */
            sh: EvcSh::default(),
            /* decoded picture buffer management */
            dpm: None,
            /* create descriptor */
            //EVCD_CDSC               cdsc;
            /* sequence parameter set */
            sps: EvcSps::default(),
            /* picture parameter set */
            pps: EvcPps::default(),
            /* current decoded (decoding) picture buffer */
            pic: None,
            /* SBAC */
            sbac_dec: EvcdSbac::default(),
            sbac_ctx: EvcSbacCtx::default(),
            /* decoding picture width */
            w: 0,
            /* decoding picture height */
            h: 0,
            cs: ChromaSampling::Cs400,
            /* maximum CU width and height */
            max_cuwh: 0,
            /* log2 of maximum CU width and height */
            log2_max_cuwh: 0,

            /* minimum CU width and height */
            min_cuwh: 0,
            /* log2 of minimum CU width and height */
            log2_min_cuwh: 0,
            /* MAPS *******************************************************************/
            /* SCU map for CU information */
            map_scu: vec![],
            /* LCU split information */
            map_split: vec![],
            /* decoded motion vector for every blocks */
            map_mv: None,
            /* reference frame indices */
            map_refi: None,
            /* intra prediction modes */
            map_ipm: vec![],
            /* new coding tool flag*/
            map_cu_mode: vec![],
            /**************************************************************************/
            /* current slice number, which is increased whenever decoding a slice.
            when receiving a slice for new picture, this value is set to zero.
            this value can be used for distinguishing b/w slices */
            slice_num: 0,
            /* last coded intra picture's picture order count */
            last_intra_poc: 0,
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
            /* the picture order count value */
            poc: EvcPoc::default(),
            /* the picture order count of the previous Tid0 picture */
            prev_pic_order_cnt_val: 0,
            /* the decoding order count of the previous picture */
            prev_doc_offset: 0,
            /* the number of currently decoded pictures */
            pic_cnt: 0,
            /* flag whether current picture is refecened picture or not */
            slice_ref_flag: false,
            /* distance between ref pics in addition to closest ref ref pic in LD*/
            ref_pic_gap_length: 0,
            /* bitstream has an error? */
            bs_err: 0,
            /* reference picture (0: foward, 1: backward) */
            refp,
            /* flag for picture signature enabling */
            use_pic_sign: 0,
            /* picture signature (MD5 digest 128bits) for each component */
            pic_sign: [[0; 16]; N_C],
            /* flag to indicate picture signature existing or not */
            pic_sign_exist: 0,
            /* flag to indicate opl decoder output */
            use_opl: 0,
            num_ctb: 0,
        }
    }

    #[hawktracer(sequence_init)]
    fn sequence_init(&mut self) -> Result<(), EvcError> {
        if self.sps.pic_width_in_luma_samples != self.w
            || self.sps.pic_height_in_luma_samples != self.h
        {
            /* resolution was changed */
            self.w = self.sps.pic_width_in_luma_samples;
            self.h = self.sps.pic_height_in_luma_samples;
            self.cs = self.sps.chroma_format_idc.into();
            assert_eq!(self.sps.sps_btt_flag, false);

            self.max_cuwh = 1 << 6;
            self.min_cuwh = 1 << 2;

            self.log2_max_cuwh = CONV_LOG2(self.max_cuwh as usize);
            self.log2_min_cuwh = CONV_LOG2(self.min_cuwh as usize);
        }

        let size = self.max_cuwh;
        self.w_lcu = (self.w + (size - 1)) / size;
        self.h_lcu = (self.h + (size - 1)) / size;
        self.f_lcu = (self.w_lcu * self.h_lcu) as u32;
        self.w_scu = (self.w + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.h_scu = (self.h + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.f_scu = (self.w_scu * self.h_scu) as u32;

        /* alloc SCU map */
        self.map_scu = vec![MCU::default(); self.f_scu as usize];

        /* alloc cu mode SCU map */
        self.map_cu_mode = vec![MCU::default(); self.f_scu as usize];

        /* alloc map for CU split flag */
        self.map_split = vec![
            LcuSplitMode::default();
            self.f_lcu as usize
                * NUM_CU_DEPTH
                * BlockShape::NUM_BLOCK_SHAPE as usize
                * MAX_CU_CNT_IN_LCU
        ];

        /* alloc map for intra prediction mode */
        self.map_ipm = vec![IntraPredDir::default(); self.f_scu as usize];

        /* initialize reference picture manager */
        self.ref_pic_gap_length = (1 << self.sps.log2_ref_pic_gap_length) as u32;

        /* initialize decode picture manager */
        let mut dpm = EvcPm::new(self.w as usize, self.h as usize, self.cs);
        dpm.evc_picman_init(
            MAX_PB_SIZE as u8,
            MAX_NUM_REF_PICS as u8,
            //PICBUF_ALLOCATOR * pa
        )?;
        self.dpm = Some(dpm);

        if self.sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            self.core.evc_tbl_qp_chroma_dynamic_ext =
                evc_derived_chroma_qp_mapping_tables(&self.sps.chroma_qp_table_struct);
        } else {
            self.core.evc_tbl_qp_chroma_dynamic_ext = vec![];
            self.core
                .evc_tbl_qp_chroma_dynamic_ext
                .push(evc_tbl_qp_chroma_ajudst_base.to_vec());
            self.core
                .evc_tbl_qp_chroma_dynamic_ext
                .push(evc_tbl_qp_chroma_ajudst_base.to_vec());
        }
        Ok(())
    }

    #[hawktracer(slice_init)]
    fn slice_init(&mut self) {
        self.core.lcu_num = 0;
        self.core.x_lcu = 0;
        self.core.y_lcu = 0;
        self.core.x_pel = 0;
        self.core.y_pel = 0;
        self.core.qp = self.sh.qp;
        self.core.qp_y = self.sh.qp + (6 * (BIT_DEPTH - 8)) as u8;
        self.core.qp_u = (self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_u as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        self.core.qp_v = (self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_v as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;

        /* clear maps */
        for i in 0..self.f_scu as usize {
            self.map_scu[i] = MCU::default();
            self.map_cu_mode[i] = MCU::default();
        }

        if self.sh.slice_type == SliceType::EVC_ST_I {
            self.last_intra_poc = self.poc.poc_val;
        }
    }

    fn update_core_loc_param(&mut self) {
        self.core.x_pel = self.core.x_lcu << self.log2_max_cuwh as u16; // entry point's x location in pixel
        self.core.y_pel = self.core.y_lcu << self.log2_max_cuwh as u16; // entry point's y location in pixel
        self.core.x_scu = self.core.x_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2) as u16; // set x_scu location
        self.core.y_scu = self.core.y_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2) as u16; // set y_scu location
        self.core.lcu_num = self.core.x_lcu + self.core.y_lcu * self.w_lcu; // Init the first lcu_num in tile
    }

    fn evcd_set_dec_info(&mut self) {
        let w_scu = self.w_scu as usize;
        let scup = self.core.scup as usize;
        let w_cu = (1 << self.core.log2_cuw as usize) >> MIN_CU_LOG2;
        let h_cu = (1 << self.core.log2_cuh as usize) >> MIN_CU_LOG2;
        let flag = if self.core.pred_mode == PredMode::MODE_INTRA {
            1
        } else {
            0
        };

        if let (Some(map_refi), Some(map_mv)) = (&mut self.map_refi, &mut self.map_mv) {
            let (mut refis, mut mvs) = (map_refi.borrow_mut(), map_mv.borrow_mut());

            if evc_check_luma(&self.core.tree_cons) {
                for i in 0..h_cu {
                    let map_scu = &mut self.map_scu[scup + i * w_scu..];
                    let map_ipm = &mut self.map_ipm[scup + i * w_scu..];
                    let map_cu_mode = &mut self.map_cu_mode[scup + i * w_scu..];
                    let refi = &mut refis[scup + i * w_scu..];
                    let mv = &mut mvs[scup + i * w_scu..];

                    for j in 0..w_cu {
                        if self.core.pred_mode == PredMode::MODE_SKIP {
                            map_scu[j].SET_SF();
                        } else {
                            map_scu[j].CLR_SF();
                        }
                        if self.core.is_coef[Y_C] {
                            map_scu[j].SET_CBFL();
                        } else {
                            map_scu[j].CLR_CBFL();
                        }

                        map_cu_mode[j].SET_LOGW(self.core.log2_cuw as u32);
                        map_cu_mode[j].SET_LOGH(self.core.log2_cuh as u32);

                        if self.pps.cu_qp_delta_enabled_flag {
                            map_scu[j].RESET_QP();
                        }
                        map_scu[j].SET_IF_COD_SN_QP(flag, self.slice_num as u32, self.core.qp);

                        map_ipm[j] = self.core.ipm[0];

                        refi[j][REFP_0] = self.core.refi[REFP_0];
                        refi[j][REFP_1] = self.core.refi[REFP_1];
                        mv[j][REFP_0][MV_X] = self.core.mv[REFP_0][MV_X];
                        mv[j][REFP_0][MV_Y] = self.core.mv[REFP_0][MV_Y];
                        mv[j][REFP_1][MV_X] = self.core.mv[REFP_1][MV_X];
                        mv[j][REFP_1][MV_Y] = self.core.mv[REFP_1][MV_Y];
                    }
                }
            }
        }
    }

    fn evcd_eco_coef(&mut self) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        let mut cbf = [false; N_C];
        let mut b_no_cbf = false;

        let log2_cuw = core.log2_cuw;
        let log2_cuh = core.log2_cuh;

        let mut tmp_coef = [0; N_C];
        let is_sub = false;
        let mut cbf_all = true;

        if cbf_all {
            eco_cbf(
                bs,
                sbac,
                sbac_ctx,
                core.pred_mode,
                &mut cbf,
                b_no_cbf,
                is_sub,
                0,
                &mut cbf_all,
                &core.tree_cons,
            )?;
        } else {
            cbf[Y_C] = false;
            cbf[U_C] = false;
            cbf[V_C] = false;
        }

        let mut dqp = 0;
        if self.pps.cu_qp_delta_enabled_flag
            && (((!(self.sps.dquant_flag)
                || (core.cu_qp_delta_code == 1 && !core.cu_qp_delta_is_coded))
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
            (core.qp as i8 + self.sh.qp_u_offset) as i8,
        );
        let qp_i_cr = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            (core.qp as i8 + self.sh.qp_v_offset) as i8,
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

    fn evcd_eco_cu(&mut self) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

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

        let cuw = 1 << core.log2_cuw as u16;
        let cuh = 1 << core.log2_cuh as u16;
        core.avail_lr = evc_check_nev_avail(
            core.x_scu,
            core.y_scu,
            cuw,
            //cuh,
            self.w_scu,
            //self.h_scu,
            &self.map_scu,
        );

        if !evc_check_only_intra(&core.tree_cons) {
            /* CU skip flag */
            let cu_skip_flag = evcd_eco_cu_skip_flag(bs, sbac, sbac_ctx, &core.ctx_flags)?;
            if cu_skip_flag != 0 {
                core.pred_mode = PredMode::MODE_SKIP;
            }
        }

        /* parse prediction info */
        if core.pred_mode == PredMode::MODE_SKIP {
            core.mvp_idx[REFP_0] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
            if self.sh.slice_type == SliceType::EVC_ST_B {
                core.mvp_idx[REFP_1] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
            }

            core.is_coef[Y_C] = false;
            core.is_coef[U_C] = false;
            core.is_coef[V_C] = false;

            core.qp = self.sh.qp;
            core.qp_y = GET_LUMA_QP(core.qp as i8) as u8;
            let qp_i_cb = EVC_CLIP3(
                -6 * (BIT_DEPTH - 8) as i8,
                57,
                core.qp as i8 + self.sh.qp_u_offset,
            );
            let qp_i_cr = EVC_CLIP3(
                -6 * (BIT_DEPTH - 8) as i8,
                57,
                core.qp as i8 + self.sh.qp_v_offset,
            );

            core.qp_u = (core.evc_tbl_qp_chroma_dynamic_ext[0]
                [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
                + (6 * (BIT_DEPTH - 8)) as i8) as u8;
            core.qp_v = (core.evc_tbl_qp_chroma_dynamic_ext[1]
                [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
                + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        } else {
            core.pred_mode =
                evcd_eco_pred_mode(bs, sbac, sbac_ctx, &core.ctx_flags, &core.tree_cons)?;

            if core.pred_mode == PredMode::MODE_INTER {
                //TODO: bugfix? missing SLICE_TYPE==B for direct_mode_flag?
                core.inter_dir = evcd_eco_direct_mode_flag(bs, sbac, sbac_ctx)?;

                if core.inter_dir != InterPredDir::PRED_DIR {
                    /* inter_pred_idc */
                    core.inter_dir =
                        evcd_eco_inter_pred_idc(bs, sbac, sbac_ctx, self.sh.slice_type)?;

                    for inter_dir_idx in 0..2 {
                        /* 0: forward, 1: backward */
                        if (((core.inter_dir as usize + 1) >> inter_dir_idx) & 1) != 0 {
                            core.refi[inter_dir_idx] = evcd_eco_refi(
                                bs,
                                sbac,
                                sbac_ctx,
                                self.dpm.as_ref().unwrap().num_refp[inter_dir_idx],
                            )? as i8;
                            core.mvp_idx[inter_dir_idx] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
                            evcd_eco_get_mvd(bs, sbac, sbac_ctx, &mut core.mvd[inter_dir_idx])?;
                        }
                    }
                }
            } else if core.pred_mode == PredMode::MODE_INTRA {
                core.mpm_b_list = evc_get_mpm_b(
                    core.x_scu,
                    core.y_scu,
                    &self.map_scu,
                    &self.map_ipm,
                    core.scup,
                    self.w_scu,
                );

                let mut luma_ipm = IntraPredDir::IPD_DC_B;
                if evc_check_luma(&core.tree_cons) {
                    core.ipm[0] = evcd_eco_intra_dir_b(bs, sbac, sbac_ctx, core.mpm_b_list)?.into();
                    luma_ipm = core.ipm[0];
                } else {
                    assert!(false);
                }
                if evc_check_chroma(&core.tree_cons) {
                    core.ipm[1] = luma_ipm;
                }

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
            self.evcd_eco_coef()?;
        }

        Ok(())
    }

    fn evcd_itdq(&mut self) {
        let mut core = &mut self.core;
        evc_sub_block_itdq(
            &mut self.bs.tracer,
            &mut core.coef.data,
            core.log2_cuw,
            core.log2_cuh,
            core.qp_y,
            core.qp_u,
            core.qp_v,
            &core.is_coef,
        );
    }

    fn get_nbr_yuv(&mut self, mut x: u16, mut y: u16, mut cuw: u8, mut cuh: u8) {
        let constrained_intra_flag =
            self.core.pred_mode == PredMode::MODE_INTRA && self.pps.constrained_intra_pred_flag;

        if let Some(pic) = &self.pic {
            let frame = &pic.borrow().frame;
            let planes = &frame.borrow().planes;
            if evc_check_luma(&self.core.tree_cons) {
                /* Y */
                evc_get_nbr_b(
                    x as usize,
                    y as usize,
                    cuw as usize,
                    cuh as usize,
                    &planes[Y_C].as_region(),
                    self.core.avail_cu,
                    &mut self.core.nb.data[Y_C],
                    self.core.scup as usize,
                    &self.map_scu,
                    self.w_scu as usize,
                    self.h_scu as usize,
                    Y_C,
                    constrained_intra_flag,
                );
            }

            if evc_check_chroma(&self.core.tree_cons) {
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
                    self.core.avail_cu,
                    &mut self.core.nb.data[U_C],
                    self.core.scup as usize,
                    &self.map_scu,
                    self.w_scu as usize,
                    self.h_scu as usize,
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
                    self.core.avail_cu,
                    &mut self.core.nb.data[V_C],
                    self.core.scup as usize,
                    &self.map_scu,
                    self.w_scu as usize,
                    self.h_scu as usize,
                    V_C,
                    constrained_intra_flag,
                );
            }
        }
    }

    fn evcd_get_skip_motion(&mut self, cuw: u8, cuh: u8) {
        let mut srefi = [[0i8; MAX_NUM_MVP]; REFP_NUM];
        let mut smvp = [[[0i16; MV_D]; MAX_NUM_MVP]; REFP_NUM];

        let core = &mut self.core;
        let map_mv = self.map_mv.as_ref().unwrap().borrow();

        evc_get_motion(
            core.scup as usize,
            REFP_0,
            &*map_mv,
            &self.refp,
            cuw as usize,
            cuh as usize,
            self.w_scu as usize,
            core.avail_cu,
            &mut srefi[REFP_0],
            &mut smvp[REFP_0],
        );

        core.refi[REFP_0] = srefi[REFP_0][core.mvp_idx[REFP_0] as usize];

        core.mv[REFP_0][MV_X] = smvp[REFP_0][core.mvp_idx[REFP_0] as usize][MV_X];
        core.mv[REFP_0][MV_Y] = smvp[REFP_0][core.mvp_idx[REFP_0] as usize][MV_Y];

        if self.sh.slice_type == SliceType::EVC_ST_P {
            core.refi[REFP_1] = REFI_INVALID;
            core.mv[REFP_1][MV_X] = 0;
            core.mv[REFP_1][MV_Y] = 0;
        } else {
            evc_get_motion(
                core.scup as usize,
                REFP_1,
                &*map_mv,
                &self.refp,
                cuw as usize,
                cuh as usize,
                self.w_scu as usize,
                core.avail_cu,
                &mut srefi[REFP_1],
                &mut smvp[REFP_1],
            );

            core.refi[REFP_1] = srefi[REFP_1][core.mvp_idx[REFP_1] as usize];
            core.mv[REFP_1][MV_X] = smvp[REFP_1][core.mvp_idx[REFP_1] as usize][MV_X];
            core.mv[REFP_1][MV_Y] = smvp[REFP_1][core.mvp_idx[REFP_1] as usize][MV_Y];
        }
    }

    fn evcd_get_inter_motion(&mut self, cuw: u8, cuh: u8) {
        let mut mvp = [[0i16; MV_D]; MAX_NUM_MVP];
        let mut refi = [0i8; MAX_NUM_MVP];

        let core = &mut self.core;
        let map_mv = self.map_mv.as_ref().unwrap().borrow();

        for inter_dir_idx in 0..2 {
            /* 0: forward, 1: backward */
            if (((core.inter_dir as usize + 1) >> inter_dir_idx) & 1) != 0 {
                evc_get_motion(
                    core.scup as usize,
                    inter_dir_idx,
                    &*map_mv,
                    &self.refp,
                    cuw as usize,
                    cuh as usize,
                    self.w_scu as usize,
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

    fn evcd_eco_unit(
        &mut self,
        x: u16,
        y: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        tree_cons: TREE_CONS_NEW,
    ) -> Result<(), EvcError> {
        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;

        //entropy decoding
        {
            let core = &mut self.core;
            let bs = &mut self.bs;
            let sbac = &mut self.sbac_dec;

            core.tree_cons = TREE_CONS {
                changed: false,
                tree_type: tree_cons.tree_type,
                mode_cons: tree_cons.mode_cons,
            };

            core.log2_cuw = log2_cuw;
            core.log2_cuh = log2_cuh;
            core.x_scu = PEL2SCU(x as usize) as u16;
            core.y_scu = PEL2SCU(y as usize) as u16;
            core.scup = core.x_scu as u32 + core.y_scu as u32 * self.w_scu as u32;

            EVC_TRACE_COUNTER(&mut bs.tracer);
            EVC_TRACE(&mut bs.tracer, "poc: ");
            EVC_TRACE(&mut bs.tracer, self.poc.poc_val);
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
            self.evcd_eco_cu()?;
        }

        /* inverse transform and dequantization */
        if self.core.pred_mode != PredMode::MODE_SKIP {
            self.evcd_itdq();
        }

        /* prediction */
        if self.core.pred_mode != PredMode::MODE_INTRA {
            self.core.avail_cu = evc_get_avail_inter(
                self.core.x_scu as usize,
                self.core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                self.core.scup as usize,
                cuw as usize,
                cuh as usize,
                &self.map_scu,
            );
            if self.core.pred_mode == PredMode::MODE_SKIP {
                self.evcd_get_skip_motion(cuw, cuh);
            } else {
                if self.core.inter_dir == InterPredDir::PRED_DIR {
                    evc_get_mv_dir(
                        &self.refp[0],
                        self.poc.poc_val,
                        self.core.scup as usize
                            + ((1 << (self.core.log2_cuw as usize - MIN_CU_LOG2)) - 1)
                            + ((1 << (self.core.log2_cuh as usize - MIN_CU_LOG2)) - 1)
                                * self.w_scu as usize,
                        self.core.scup as usize,
                        self.w_scu,
                        self.h_scu,
                        &mut self.core.mv,
                    );
                    self.core.refi[REFP_0] = 0;
                    self.core.refi[REFP_1] = 0;
                } else {
                    self.evcd_get_inter_motion(cuw, cuh);
                }
            }

            EVC_TRACE_COUNTER(&mut self.bs.tracer);
            EVC_TRACE(&mut self.bs.tracer, "Inter: ");
            EVC_TRACE(&mut self.bs.tracer, self.core.inter_dir as isize);
            EVC_TRACE(&mut self.bs.tracer, " , mv[REFP_0]:( ");
            EVC_TRACE(&mut self.bs.tracer, self.core.mv[REFP_0][MV_X]);
            EVC_TRACE(&mut self.bs.tracer, " , ");
            EVC_TRACE(&mut self.bs.tracer, self.core.mv[REFP_0][MV_Y]);
            EVC_TRACE(&mut self.bs.tracer, " ), mv[REFP_1]:( ");
            EVC_TRACE(&mut self.bs.tracer, self.core.mv[REFP_1][MV_X]);
            EVC_TRACE(&mut self.bs.tracer, " , ");
            EVC_TRACE(&mut self.bs.tracer, self.core.mv[REFP_1][MV_Y]);
            EVC_TRACE(&mut self.bs.tracer, " )\n");

            evc_mc(
                x as i16,
                y as i16,
                self.w as i16,
                self.h as i16,
                cuw as i16,
                cuh as i16,
                &self.core.refi,
                &self.core.mv,
                &self.refp,
                &mut self.core.pred,
                self.poc.poc_val,
            );
        } else {
            self.core.avail_cu = evc_get_avail_intra(
                self.core.x_scu as usize,
                self.core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                self.core.scup as usize,
                self.core.log2_cuw,
                self.core.log2_cuh,
                &self.map_scu,
            );
            self.get_nbr_yuv(x, y, cuw, cuh);

            EVC_TRACE_COUNTER(&mut self.bs.tracer);
            EVC_TRACE(&mut self.bs.tracer, "Intra: ");
            EVC_TRACE(&mut self.bs.tracer, self.core.ipm[0] as isize);
            EVC_TRACE(&mut self.bs.tracer, " , ");
            EVC_TRACE(&mut self.bs.tracer, self.core.ipm[1] as isize);
            EVC_TRACE(&mut self.bs.tracer, " \n");

            if evc_check_luma(&self.core.tree_cons) {
                evc_ipred_b(
                    &self.core.nb.data[Y_C][0][2..],
                    &self.core.nb.data[Y_C][1][cuh as usize..],
                    self.core.nb.data[Y_C][1][cuh as usize - 1],
                    &mut self.core.pred[0].data[Y_C],
                    self.core.ipm[0],
                    cuw as usize,
                    cuh as usize,
                );
            }
            if evc_check_chroma(&self.core.tree_cons) {
                evc_ipred_b(
                    &self.core.nb.data[U_C][0][2..],
                    &self.core.nb.data[U_C][1][(cuh >> 1) as usize..],
                    self.core.nb.data[U_C][1][(cuh >> 1) as usize - 1],
                    &mut self.core.pred[0].data[U_C],
                    self.core.ipm[1],
                    cuw as usize >> 1,
                    cuh as usize >> 1,
                );
                evc_ipred_b(
                    &self.core.nb.data[V_C][0][2..],
                    &self.core.nb.data[V_C][1][(cuh >> 1) as usize..],
                    self.core.nb.data[V_C][1][(cuh >> 1) as usize - 1],
                    &mut self.core.pred[0].data[V_C],
                    self.core.ipm[1],
                    cuw as usize >> 1,
                    cuh as usize >> 1,
                );
            }
        }
        self.evcd_set_dec_info();

        TRACE_PRED(
            &mut self.bs.tracer,
            Y_C,
            cuw as usize,
            cuh as usize,
            &self.core.pred[0].data[Y_C],
        );
        TRACE_PRED(
            &mut self.bs.tracer,
            U_C,
            cuw as usize >> 1,
            cuh as usize >> 1,
            &self.core.pred[0].data[U_C],
        );
        TRACE_PRED(
            &mut self.bs.tracer,
            V_C,
            cuw as usize >> 1,
            cuh as usize >> 1,
            &self.core.pred[0].data[V_C],
        );

        /* reconstruction */
        if let Some(pic) = &self.pic {
            evc_recon_yuv(
                &mut self.bs.tracer,
                x as usize,
                y as usize,
                cuw as usize,
                cuh as usize,
                &self.core.coef.data,
                &self.core.pred[0].data,
                &self.core.is_coef,
                &mut pic.borrow().frame.borrow_mut().planes,
                &self.core.tree_cons,
            );
        }

        Ok(())
    }

    fn evcd_eco_tree(
        &mut self,
        x0: u16,
        y0: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        cup: u16,
        cud: u16,
        next_split: bool,
        qt_depth: u8,
        mut cu_qp_delta_code: u8,
        mut mode_cons: MODE_CONS,
    ) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        let cuw = 1 << log2_cuw as u16;
        let cuh = 1 << log2_cuh as u16;
        let mut split_mode = SplitMode::NO_SPLIT;
        if cuw > self.min_cuwh || cuh > self.min_cuwh {
            if x0 + cuw <= self.w && y0 + cuh <= self.h {
                if next_split {
                    split_mode = evcd_eco_split_mode(bs, sbac, sbac_ctx, cuw, cuh)?;
                    EVC_TRACE_COUNTER(&mut bs.tracer);
                    EVC_TRACE(&mut bs.tracer, "x pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.x_pel
                            + (cup % (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " y pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.y_pel
                            + (cup / (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " width ");
                    EVC_TRACE(&mut bs.tracer, cuw);
                    EVC_TRACE(&mut bs.tracer, " height ");
                    EVC_TRACE(&mut bs.tracer, cuh);
                    EVC_TRACE(&mut bs.tracer, " depth ");
                    EVC_TRACE(&mut bs.tracer, cud);
                    EVC_TRACE(&mut bs.tracer, " split mode ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        if split_mode == SplitMode::NO_SPLIT {
                            0
                        } else {
                            5
                        },
                    );
                    EVC_TRACE(&mut bs.tracer, " \n");
                } else {
                    split_mode = SplitMode::NO_SPLIT;
                }
            } else {
                split_mode = evcd_eco_split_mode(bs, sbac, sbac_ctx, cuw, cuh)?;
                EVC_TRACE_COUNTER(&mut bs.tracer);
                EVC_TRACE(&mut bs.tracer, "x pos ");
                EVC_TRACE(
                    &mut bs.tracer,
                    core.x_pel
                        + (cup % (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " y pos ");
                EVC_TRACE(
                    &mut bs.tracer,
                    core.y_pel
                        + (cup / (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " width ");
                EVC_TRACE(&mut bs.tracer, cuw);
                EVC_TRACE(&mut bs.tracer, " height ");
                EVC_TRACE(&mut bs.tracer, cuh);
                EVC_TRACE(&mut bs.tracer, " depth ");
                EVC_TRACE(&mut bs.tracer, cud);
                EVC_TRACE(&mut bs.tracer, " split mode ");
                EVC_TRACE(
                    &mut bs.tracer,
                    if split_mode == SplitMode::NO_SPLIT {
                        0
                    } else {
                        5
                    },
                );
                EVC_TRACE(&mut bs.tracer, " \n");
            }
        } else {
            split_mode = SplitMode::NO_SPLIT;
        }

        if self.pps.cu_qp_delta_enabled_flag && self.sps.dquant_flag {
            if split_mode == SplitMode::NO_SPLIT
                && (log2_cuh + log2_cuw >= self.pps.cu_qp_delta_area)
                && cu_qp_delta_code != 2
            {
                if log2_cuh == 7 || log2_cuw == 7 {
                    cu_qp_delta_code = 2;
                } else {
                    cu_qp_delta_code = 1;
                }
                core.cu_qp_delta_is_coded = false;
            }
        }

        evc_set_split_mode(
            &mut core.split_mode,
            split_mode,
            cud,
            cup,
            cuw,
            cuh,
            self.max_cuwh,
        );

        if split_mode != SplitMode::NO_SPLIT {
            let split_struct = evc_split_get_part_structure(
                split_mode,
                x0,
                y0,
                cuw,
                cuh,
                cup,
                cud,
                self.log2_max_cuwh - MIN_CU_LOG2 as u8,
            );

            for cur_part_num in 0..split_struct.part_count {
                let log2_sub_cuw = split_struct.log_cuw[cur_part_num];
                let log2_sub_cuh = split_struct.log_cuh[cur_part_num];
                let x_pos = split_struct.x_pos[cur_part_num];
                let y_pos = split_struct.y_pos[cur_part_num];

                if x_pos < self.w && y_pos < self.h {
                    self.evcd_eco_tree(
                        x_pos,
                        y_pos,
                        log2_sub_cuw,
                        log2_sub_cuh,
                        split_struct.cup[cur_part_num],
                        split_struct.cud[cur_part_num],
                        true,
                        split_mode.inc_qt_depth(qt_depth),
                        cu_qp_delta_code,
                        mode_cons,
                    )?;
                }
            }
        } else {
            core.cu_qp_delta_code = cu_qp_delta_code;

            let tree_type = if mode_cons == MODE_CONS::eOnlyIntra {
                TREE_TYPE::TREE_L
            } else {
                TREE_TYPE::TREE_LC
            };

            if self.sh.slice_type == SliceType::EVC_ST_I {
                mode_cons = MODE_CONS::eOnlyIntra;
            }

            self.evcd_eco_unit(
                x0,
                y0,
                log2_cuw,
                log2_cuh,
                TREE_CONS_NEW {
                    tree_type,
                    mode_cons,
                },
            )?;
        }

        Ok(())
    }

    #[hawktracer(decode_slice)]
    fn decode_slice(&mut self) -> Result<(), EvcError> {
        // Initialize CABAC at each tile
        self.sbac_dec.reset(
            &mut self.bs,
            &mut self.sbac_ctx,
            self.sh.slice_type,
            self.sh.qp,
        );

        //TODO: move x_lcu/y_lcu=0 to pic init
        self.core.x_lcu = 0; //entry point lcu's x location
        self.core.y_lcu = 0; // entry point lcu's y location
        while self.num_ctb > 0 {
            self.update_core_loc_param();

            //LCU decoding with in a tile
            evc_assert_rv(
                (self.core.lcu_num as u32) < self.f_lcu,
                EvcError::EVC_ERR_UNEXPECTED,
            )?;

            // invoke coding_tree() recursion
            for i in 0..NUM_CU_DEPTH {
                for j in 0..BlockShape::NUM_BLOCK_SHAPE as usize {
                    for k in 0..MAX_CU_CNT_IN_LCU {
                        self.core.split_mode.data[i][j][k] = SplitMode::NO_SPLIT;
                    }
                }
            }

            self.evcd_eco_tree(
                self.core.x_pel,
                self.core.y_pel,
                self.log2_max_cuwh,
                self.log2_max_cuwh,
                0,
                0,
                true,
                0,
                0,
                MODE_CONS::eAll,
            )?;
            // set split flags to map
            self.map_split[self.core.lcu_num as usize].clone_from(&self.core.split_mode);

            self.num_ctb -= 1;
            // read end_of_picture_flag
            if (self.num_ctb == 0) {
                evcd_eco_tile_end_flag(&mut self.bs, &mut self.sbac_dec)?;
            } else {
                self.core.x_lcu += 1;
                if self.core.x_lcu >= self.w_lcu {
                    self.core.x_lcu = 0;
                    self.core.y_lcu += 1;
                }
            }
            //eprint!("{} ", self.num_ctb);
        }

        Ok(())
    }

    fn make_stat(&mut self, btype: NaluType) -> EvcStat {
        let mut stat = EvcStat {
            nalu_type: btype,
            stype: SliceType::EVC_ST_I,
            fnum: -1,
            bytes: NALU_SIZE_FIELD_IN_BYTES + self.bs.get_read_byte() as usize,
            ..Default::default()
        };

        if btype < NaluType::EVC_SPS_NUT {
            stat.fnum = self.pic_cnt as isize;
            stat.stype = self.sh.slice_type;

            /* increase decoded picture count */
            self.pic_cnt += 1;
            stat.poc = self.poc.poc_val as isize;
            stat.tid = self.nalu.nuh_temporal_id as isize;

            for i in 0..2 {
                stat.refpic_num[i] = self.dpm.as_ref().unwrap().num_refp[i];
                for j in 0..stat.refpic_num[i] as usize {
                    stat.refpic[i][j] = self.refp[j][i].poc as isize;
                }
            }
        }

        stat
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
        tree_cons: &TREE_CONS_NEW,
    ) {
        self.core.tree_cons.changed = false;
        self.core.tree_cons.tree_type = tree_cons.tree_type;
        self.core.tree_cons.mode_cons = tree_cons.mode_cons;
        let lcu_num = (x >> self.log2_max_cuwh) + (y >> self.log2_max_cuwh) * self.w_lcu;
        let split_mode = evc_get_split_mode(
            cud,
            cup,
            cuw,
            cuh,
            self.max_cuwh,
            &self.map_split[lcu_num as usize],
        );

        EVC_TRACE_COUNTER(&mut self.bs.tracer);
        EVC_TRACE(&mut self.bs.tracer, "split_mod ");
        EVC_TRACE(&mut self.bs.tracer, split_mode as u8);
        EVC_TRACE(&mut self.bs.tracer, " \n");

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
            let tree_constrain_for_child = TREE_CONS_NEW {
                tree_type: TREE_TYPE::TREE_LC,
                mode_cons: MODE_CONS::eAll,
            };

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
                        &tree_constrain_for_child,
                    );
                }
            }

            self.core.tree_cons.changed = false;
            self.core.tree_cons.tree_type = tree_cons.tree_type;
            self.core.tree_cons.mode_cons = tree_cons.mode_cons;
        } else if let (Some(pic), Some(map_refi), Some(map_mv)) =
            (&self.pic, &self.map_refi, &self.map_mv)
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
                        &self.core.tree_cons,
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
                        &self.core.tree_cons,
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
                        &self.core.tree_cons,
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
                        &self.core.tree_cons,
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
                        &self.core.tree_cons,
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
                        &self.core.tree_cons,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                }
            }
        }

        self.core.tree_cons.changed = false;
        self.core.tree_cons.tree_type = tree_cons.tree_type;
        self.core.tree_cons.mode_cons = tree_cons.mode_cons;
    }

    #[hawktracer(evcd_deblock)]
    fn evcd_deblock(&mut self) -> Result<(), EvcError> {
        if let Some(pic) = &self.pic {
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
                    &TREE_CONS_NEW {
                        tree_type: TREE_TYPE::TREE_LC,
                        mode_cons: MODE_CONS::eAll,
                    },
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
                    &TREE_CONS_NEW {
                        tree_type: TREE_TYPE::TREE_LC,
                        mode_cons: MODE_CONS::eAll,
                    },
                );
            }
        }

        Ok(())
    }

    pub(crate) fn push_pkt(&mut self, pkt: &mut Option<Packet>) -> Result<(), EvcError> {
        self.pkt = pkt.take();
        Ok(())
    }

    pub(crate) fn decode_nalu(&mut self) -> Result<EvcStat, EvcError> {
        if self.pkt.is_none() {
            return Err(EvcError::EVC_OK_FLUSH);
        }

        let pkt = self.pkt.take().ok_or(EvcError::EVC_ERR_EMPTY_PACKET)?;

        /* bitstream reader initialization */
        self.bs = EvcdBsr::new(pkt);

        /* parse nalu header */
        evcd_eco_nalu(&mut self.bs, &mut self.nalu)?;

        let nalu_type = self.nalu.nal_unit_type;
        if nalu_type == NaluType::EVC_SPS_NUT {
            evcd_eco_sps(&mut self.bs, &mut self.sps)?;

            self.sequence_init()?;
        } else if nalu_type == NaluType::EVC_PPS_NUT {
            evcd_eco_pps(&mut self.bs, &self.sps, &mut self.pps)?;
        } else if nalu_type < NaluType::EVC_SPS_NUT {
            /* decode slice header */
            self.sh.num_ctb = self.f_lcu as u16;

            evcd_eco_sh(&mut self.bs, &self.sps, &self.pps, &mut self.sh, nalu_type)?;

            if self.num_ctb == 0 {
                self.num_ctb = self.f_lcu;
            }

            /* POC derivation process */
            assert_eq!(self.sps.tool_pocs, false);
            if !self.sps.tool_pocs {
                //sps_pocs_flag == 0
                if nalu_type == NaluType::EVC_IDR_NUT {
                    self.sh.poc_lsb = 0;
                    self.poc.prev_doc_offset = -1;
                    self.poc.prev_poc_val = 0;
                    self.slice_ref_flag = (self.nalu.nuh_temporal_id == 0
                        || self.nalu.nuh_temporal_id < self.sps.log2_sub_gop_length);
                    self.poc.poc_val = 0;
                } else {
                    self.slice_ref_flag = (self.nalu.nuh_temporal_id == 0
                        || self.nalu.nuh_temporal_id < self.sps.log2_sub_gop_length);
                    evc_poc_derivation(&self.sps, self.nalu.nuh_temporal_id, &mut self.poc);
                    self.sh.poc_lsb = self.poc.poc_val;
                }
            }

            self.slice_init();

            if self.num_ctb == 0 {
                self.num_ctb = self.f_lcu;
                self.slice_num = 0;
            } else {
                self.slice_num += 1;
            }

            /* initialize reference pictures */
            self.dpm.as_mut().unwrap().evc_picman_refp_init(
                self.sps.max_num_ref_pics,
                self.sh.slice_type,
                self.poc.poc_val as u32,
                self.nalu.nuh_temporal_id,
                self.last_intra_poc,
                &mut self.refp,
            );

            if self.num_ctb == self.f_lcu {
                /* get available frame buffer for decoded image */
                self.pic = self.dpm.as_mut().unwrap().evc_picman_get_empty_pic()?;

                /* get available frame buffer for decoded image */
                if let Some(pic) = &self.pic {
                    let p = pic.borrow();
                    self.map_refi = Some(Rc::clone(&p.map_refi));
                    self.map_mv = Some(Rc::clone(&p.map_mv));
                }
            }

            /* decode slice layer */
            self.decode_slice()?;

            /* deblocking filter */
            if self.sh.deblocking_filter_on {
                self.evcd_deblock()?;
            }

            if self.num_ctb == 0 {
                /* expand pixels to padding area */
                if let Some(pic) = &self.pic {
                    let frame = &pic.borrow().frame;
                    frame.borrow_mut().pad();
                }

                /* put decoded picture to DPB */
                self.dpm.as_mut().unwrap().evc_picman_put_pic(
                    &self.pic,
                    self.nalu.nal_unit_type == NaluType::EVC_IDR_NUT,
                    self.poc.poc_val as u32,
                    self.nalu.nuh_temporal_id,
                    true,
                    &mut self.refp,
                    self.slice_ref_flag,
                    self.ref_pic_gap_length,
                );
            }
        } else if nalu_type == NaluType::EVC_SEI_NUT {
        } else {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }

        let mut stat = self.make_stat(nalu_type);
        if self.num_ctb > 0 {
            stat.fnum = -1;
        }

        Ok(stat)
    }

    pub(crate) fn pull_frm(&mut self) -> Result<Rc<RefCell<Frame<pel>>>, EvcError> {
        let pic = self.dpm.as_mut().unwrap().evc_picman_out_pic()?;
        if let Some(p) = &pic {
            Ok(Rc::clone(&p.borrow().frame))
        } else {
            Err(EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE)
        }
    }
}
