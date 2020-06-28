use super::api::*;
use super::com::ipred::*;
use super::com::tbl::*;
use super::com::util::*;
use super::com::*;

mod bsr;
mod eco;
mod sbac;

use bsr::*;
use eco::*;
use sbac::*;

/* evc decoder magic code */
pub(crate) const EVCD_MAGIC_CODE: u32 = 0x45565944; /* EVYD */

#[derive(Clone)]
pub(crate) struct LcuSplitMode {
    pub(crate) data:
        [[[SplitMode; MAX_CU_CNT_IN_LCU]; BlockShape::NUM_BLOCK_SHAPE as usize]; NUM_CU_DEPTH],
}

impl Default for LcuSplitMode {
    fn default() -> Self {
        LcuSplitMode {
            data: [[[SplitMode::NO_SPLIT; MAX_CU_CNT_IN_LCU]; BlockShape::NUM_BLOCK_SHAPE as usize];
                NUM_CU_DEPTH],
        }
    }
}

#[derive(Clone)]
pub(crate) struct CUBuffer<T: Default + Copy> {
    pub(crate) data: [[T; MAX_CU_DIM]; N_C],
}

impl<T: Default + Copy> Default for CUBuffer<T> {
    fn default() -> Self {
        CUBuffer {
            data: [[T::default(); MAX_CU_DIM]; N_C],
        }
    }
}

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
    pred: CUBuffer<pel>, //[[[pel; MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]

    /* neighbor pixel buffer for intra prediction */
    //nb: [[[pel; MAX_CU_SIZE * 3]; N_REF]; N_C],
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
    is_coef_sub: [[bool; MAX_SUB_TB_NUM]; N_C],

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

    /* platform specific data, if needed */
    //void          *pf;
    //s16            mmvd_idx;
    //u8             mmvd_flag;

    /* temporal pixel buffer for inter prediction */
    //pel            eif_tmp_buffer[ (MAX_CU_SIZE + 2) * (MAX_CU_SIZE + 2) ];
    mvr_idx: u8,

    mvp_idx: [u8; REFP_NUM],
    mvd: [[i16; MV_D]; REFP_NUM],
    inter_dir: PredDir,
    bi_idx: i16,
    ctx_flags: [u8; CtxNevIdx::NUM_CNID as usize],
    tree_cons: TREE_CONS,
}
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
    core: EvcdCore,
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
    sbac_dec: EvcdSbac,
    sbac_ctx: EvcSbacCtx,
    /* decoding picture width */
    pub(crate) w: u16,
    /* decoding picture height */
    pub(crate) h: u16,
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
    /*s16                  (* map_mv)[REFP_NUM][MV_D];
    /* decoded motion vector for every blocks */
    s16                  (* map_unrefined_mv)[REFP_NUM][MV_D];
    /* reference frame indices */
    s8                   (* map_refi)[REFP_NUM];*/
    /* intra prediction modes */
    map_ipm: Vec<IntraPredDir>,
    /* new coding tool flag*/
    map_cu_mode: Vec<u32>,
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
    fn sequence_init(&mut self) {
        if self.sps.pic_width_in_luma_samples != self.w
            || self.sps.pic_height_in_luma_samples != self.h
        {
            /* resolution was changed */
            self.w = self.sps.pic_width_in_luma_samples;
            self.h = self.sps.pic_height_in_luma_samples;

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
        self.map_cu_mode = vec![0; self.f_scu as usize];

        /* alloc map for CU split flag */
        self.map_split = vec![LcuSplitMode::default(); self.f_lcu as usize];
    }

    fn slice_init(&mut self) {
        self.core.lcu_num = 0;
        self.core.x_lcu = 0;
        self.core.y_lcu = 0;
        self.core.x_pel = 0;
        self.core.y_pel = 0;
        self.core.qp_y = self.sh.qp + (6 * (BIT_DEPTH - 8)) as u8;
        self.core.qp_u = (evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_u as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        self.core.qp_v = (evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_v as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;

        /* clear maps */
        /*evc_mset_x64a(self.map_scu, 0, sizeof(u32) * self.f_scu);
        evc_mset_x64a(self.map_affine, 0, sizeof(u32) * self.f_scu);
        evc_mset_x64a(self.map_ats_inter, 0, sizeof(u8) * self.f_scu);
        evc_mset_x64a(self.map_cu_mode, 0, sizeof(u32) * self.f_scu);*/

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

    fn evcd_eco_coef(&mut self) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        let mut cbf = [false; N_C];
        let mut b_no_cbf = false;

        let log2_tuw = core.log2_cuw;
        let log2_tuh = core.log2_cuh;

        let mut coef_temp_buf = [[0i16; MAX_TR_DIM]; N_C];
        let log2_w_sub = if core.log2_cuw > MAX_TR_LOG2 as u8 {
            MAX_TR_LOG2 as u8
        } else {
            core.log2_cuw
        };
        let log2_h_sub = if core.log2_cuh > MAX_TR_LOG2 as u8 {
            MAX_TR_LOG2 as u8
        } else {
            core.log2_cuh
        };
        let loop_w = if core.log2_cuw > MAX_TR_LOG2 as u8 {
            1 << (core.log2_cuw - MAX_TR_LOG2 as u8)
        } else {
            1
        };
        let loop_h = if core.log2_cuh > MAX_TR_LOG2 as u8 {
            1 << (core.log2_cuh - MAX_TR_LOG2 as u8)
        } else {
            1
        };
        let stride = (1 << core.log2_cuw);
        let sub_stride = (1 << log2_w_sub);
        let mut tmp_coef = [0; N_C];
        let is_sub = if loop_h + loop_w > 2 { true } else { false };
        let mut cbf_all = true;

        let is_intra = if core.pred_mode == PredMode::MODE_INTRA {
            true
        } else {
            false
        };

        for i in 0..N_C {
            for j in 0..MAX_SUB_TB_NUM {
                core.is_coef_sub[i][j] = false;
            }
        }

        for j in 0..loop_h {
            for i in 0..loop_w {
                if cbf_all {
                    eco_cbf(
                        bs,
                        sbac,
                        sbac_ctx,
                        core.pred_mode,
                        &mut cbf,
                        b_no_cbf,
                        is_sub,
                        j + i,
                        &mut cbf_all,
                        &core.tree_cons,
                    )?;
                } else {
                    cbf[Y_C] = false;
                    cbf[U_C] = false;
                    cbf[V_C] = false;
                }

                let mut dqp = 0;
                //int qp_i_cb, qp_i_cr;
                if self.pps.cu_qp_delta_enabled_flag
                    && (((!(self.sps.dquant_flag)
                        || (core.cu_qp_delta_code == 1 && !core.cu_qp_delta_is_coded))
                        && (cbf[Y_C] || cbf[U_C] || cbf[V_C]))
                        || (core.cu_qp_delta_code == 2 && !core.cu_qp_delta_is_coded))
                {
                    dqp = evcd_eco_dqp(bs, sbac, sbac_ctx)?;
                    core.qp = GET_QP(core.qp as i8, dqp) as u8; //GET_QP(ctx->tile[core->tile_num].qp_prev_eco, dqp);
                    core.qp_y = GET_LUMA_QP(core.qp as i8) as u8;
                    core.cu_qp_delta_is_coded = true;
                //ctx->tile[core->tile_num].qp_prev_eco = core->qp;
                } else {
                    dqp = 0;
                    //core.qp = //GET_QP(ctx->tile[core->tile_num].qp_prev_eco, dqp);
                    core.qp_y = GET_LUMA_QP(core.qp as i8) as u8;
                }

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
                core.qp_u = (evc_tbl_qp_chroma_dynamic_ext[0]
                    [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
                    + (6 * (BIT_DEPTH - 8)) as i8) as u8;
                core.qp_v = (evc_tbl_qp_chroma_dynamic_ext[1]
                    [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
                    + (6 * (BIT_DEPTH - 8)) as i8) as u8;

                for c in 0..N_C {
                    if cbf[c] {
                        let chroma = if c > 0 { 1 } else { 0 };
                        let pos_sub_x = i * (1 << (log2_w_sub - chroma));
                        let pos_sub_y = j * (1 << (log2_h_sub - chroma)) * (stride >> chroma);

                        let coef_temp = if is_sub {
                            evc_block_copy(
                                &core.coef.data[c][(pos_sub_x + pos_sub_y) as usize..],
                                (stride >> chroma) as usize,
                                &mut coef_temp_buf[c][..],
                                (sub_stride >> chroma) as usize,
                                log2_w_sub - chroma,
                                log2_h_sub - chroma,
                            );
                            &mut coef_temp_buf[c][..]
                        } else {
                            &mut core.coef.data[c][..]
                        };

                        evcd_eco_xcoef(
                            bs,
                            sbac,
                            sbac_ctx,
                            coef_temp,
                            log2_w_sub - chroma,
                            log2_h_sub - chroma,
                            c,
                        )?;

                        core.is_coef_sub[c][((j << 1) | i) as usize] = true;
                        tmp_coef[c] += 1;

                        if is_sub {
                            evc_block_copy(
                                &coef_temp_buf[c],
                                (sub_stride >> chroma) as usize,
                                &mut core.coef.data[c][(pos_sub_x + pos_sub_y) as usize..],
                                (stride >> chroma) as usize,
                                log2_w_sub - chroma,
                                log2_h_sub - chroma,
                            );
                        }
                    } else {
                        core.is_coef_sub[c][((j << 1) | i) as usize] = false;
                        tmp_coef[c] += 0;
                    }
                }
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

        core.pred_mode = PredMode::MODE_INTRA;
        core.mvp_idx[REFP_0] = 0;
        core.mvp_idx[REFP_1] = 0;
        core.inter_dir = PredDir::PRED_L0;
        core.bi_idx = 0;
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

        //evc_get_ctx_some_flags

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
            core.is_coef[V_C] = false; //TODO: Tim why we need to duplicate code here?
            for i in 0..N_C {
                for j in 0..MAX_SUB_TB_NUM {
                    core.is_coef_sub[i][j] = false;
                }
            }

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

            //TODO: fix negative array index
            core.qp_u = (evc_tbl_qp_chroma_dynamic_ext[0]
                [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
                + (6 * (BIT_DEPTH - 8)) as i8) as u8;
            core.qp_v = (evc_tbl_qp_chroma_dynamic_ext[1]
                [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
                + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        } else {
            core.pred_mode =
                evcd_eco_pred_mode(bs, sbac, sbac_ctx, &core.ctx_flags, &core.tree_cons)?;

            //TODO: bugfix? missing SLICE_TYPE==B
            if core.pred_mode == PredMode::MODE_INTER {
                core.inter_dir = evcd_eco_direct_mode_flag(bs, sbac, sbac_ctx)?;
            //TODO
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

    fn evcd_eco_unit(
        &mut self,
        x: u16,
        y: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        tree_cons: TREE_CONS_NEW,
    ) -> Result<(), EvcError> {
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

        let cuw = 1 << log2_cuw as u16;
        let cuh = 1 << log2_cuh as u16;

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

        core.avail_lr = evc_check_nev_avail(
            core.x_scu,
            core.y_scu,
            cuw,
            //cuh,
            self.w_scu,
            //self.h_scu,
            &self.map_scu,
        );

        // evc_get_ctx_some_flags

        /* parse CU info */
        self.evcd_eco_cu()?;

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
                    EVC_TRACE(&mut bs.tracer, split_mode as u8);
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
                EVC_TRACE(&mut bs.tracer, split_mode as u8);
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
            let split_struct: EvcSplitStruct = evc_split_get_part_structure(
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
        }

        Ok(())
    }

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
        evcd_eco_nalu(&mut self.bs, &mut self.nalu)?;

        let nalu_type = self.nalu.nal_unit_type;
        if nalu_type == NaluType::EVC_SPS_NUT {
            evcd_eco_sps(&mut self.bs, &mut self.sps)?;

            self.sequence_init();
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

            if !self.sps.tool_rpl {
                /* initialize reference pictures */
                //evc_picman_refp_init(&self.dpm, self.sps.max_num_ref_pics, sh->slice_type, self.poc.poc_val, self.nalu.nuh_temporal_id, self.last_intra_poc, self.refp);
            }

            if self.num_ctb == self.f_lcu {
                /* get available frame buffer for decoded image */
                //self.pic = evc_picman_get_empty_pic(&self.dpm)?;

                /* get available frame buffer for decoded image */
                //self.map_refi = self.pic->map_refi;
                //self.map_mv = self.pic->map_mv;
            }

            /* decode slice layer */
            self.decode_slice()?;
        } else if nalu_type == NaluType::EVC_SEI_NUT {
        } else {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }

        Ok(EvcdStat {
            read: nalu_size_field_in_bytes + self.bs.get_read_byte() as usize,
            nalu_type,
            stype: self.sh.slice_type,
            fnum: -1,
            ..Default::default()
        })
    }
}
