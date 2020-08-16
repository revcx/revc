use super::sad::*;
use super::*;
use crate::api::*;
use crate::def::*;
use crate::mc::*;
use crate::picman::*;

use std::cell::RefCell;
use std::rc::Rc;

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

        s8   first_refi[PRED_NUM][REFP_NUM];
        u8   bi_idx[PRED_NUM];
        u8   curr_bi;
        int max_search_range;
         u8   mvp_idx_scale[REFP_NUM][MAX_NUM_ACTIVE_REF_FRAME];

        pel  p_error[MAX_CU_DIM];
        int  i_gradient[2][MAX_CU_DIM];
        s16  resi[N_C][MAX_CU_DIM];
        s16  coff_save[N_C][MAX_CU_DIM];
    */
        /* MV predictor */
    mvp: [[[i16; MV_D]; MAX_NUM_MVP]; REFP_NUM],

    mv: [[[i16; MV_D]; REFP_NUM]; InterPredDir::PRED_NUM as usize],
    mvd: [[[i16; MV_D]; REFP_NUM]; InterPredDir::PRED_NUM as usize],
    /*
      s16  org_bi[MAX_CU_DIM];
      s32  mot_bits[REFP_NUM];

       */
    /* temporary prediction buffer (only used for ME)*/
    pred: [[CUBuffer<pel>; 2]; InterPredDir::PRED_NUM as usize + 1],

    /* reconstruction buffer */
    rec: [CUBuffer<pel>; InterPredDir::PRED_NUM as usize],
    /* last one buffer used for RDO */
    coef: [CUBuffer<i16>; InterPredDir::PRED_NUM as usize + 1],

    residue: CUBuffer<i16>,

    nnz_best: [[u16; N_C]; InterPredDir::PRED_NUM as usize],

    /*
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

    */
     /* original (input) picture buffer */
    pic_o: Option<Rc<RefCell<EvcPic>>>,
    /* mode picture buffer */
    pic_m: Option<Rc<RefCell<EvcPic>>>,
    /* motion vector map */
    map_mv: Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    /* picture width in SCU unit */
    w_scu: u16,
    /* QP for luma of current encoding CU */
    pub(crate) qp_y: u8,
    /* QP for chroma of current encoding CU */
    pub(crate) qp_u: u8,
    pub(crate) qp_v: u8,
    lambda_mv: u32,
    /* reference pictures */
    //refp: Option<Rc<RefCell<Vec<Vec<EvcRefP>>>>>,
    slice_type: SliceType,
    /* search level for motion estimation */
    /*int              me_level;
    int              complexity;
    void            *pdata[4];
    int             *ndata[4];*/
    /* current picture order count */
    poc: i32,
    /* gop size */
    gop_size: usize,
}

impl EvceCtx {
    pub(crate) fn pinter_init_frame(&mut self) {
        let pi = &mut self.pinter;

        pi.slice_type = self.slice_type;
        if let Some(pic) = &self.pic[PIC_IDX_ORIG] {
            pi.pic_o = Some(Rc::clone(pic));
        }
        if let Some(pic) = &self.pic[PIC_IDX_MODE] {
            pi.pic_m = Some(Rc::clone(pic));
        }
        if let Some(mv) = &self.map_mv {
            pi.map_mv = Some(Rc::clone(mv));
        }
    }

    pub(crate) fn pinter_analyze_frame(&mut self) {}

    pub(crate) fn pinter_init_lcu(&mut self) {
        let pi = &mut self.pinter;

        pi.lambda_mv = (65536.0 * self.sqrt_lambda[0]).floor() as u32;
        pi.qp_y = self.core.qp_y;
        pi.qp_u = self.core.qp_u;
        pi.qp_v = self.core.qp_v;
        pi.poc = self.poc.poc_val;
        pi.gop_size = self.gop_size;
    }

    pub(crate) fn pinter_analyze_lcu(&mut self) {}

    pub(crate) fn pinter_analyze_cu(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
    ) -> f64 {
        let cuw = (1 << log2_cuw) as usize;
        let cuh = (1 << log2_cuh) as usize;
        let mut cost_inter = vec![MAX_COST; InterPredDir::PRED_NUM as usize];
        let mut cost_best = MAX_COST;
        let mut best_idx = InterPredDir::PRED_SKIP;

        /* skip mode */
        let mut cost = self.analyze_skip_baseline(x, y, log2_cuw, log2_cuh);
        cost_inter[InterPredDir::PRED_SKIP as usize] = cost;

        if cost < cost_best {
            self.core.cu_mode = PredMode::MODE_SKIP;
            best_idx = InterPredDir::PRED_SKIP;
            cost_inter[best_idx as usize] = cost;
            cost_best = cost;

            self.core.s_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.s_temp_best;
            self.core.c_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.c_temp_best;
            self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.dqp_temp_best;

            for v in &mut self.pinter.nnz_best[InterPredDir::PRED_SKIP as usize] {
                *v = 0;
            }
        }

        if self.pinter.slice_type == SliceType::EVC_ST_B {
            cost = self.analyze_t_direct(x, y, log2_cuw, log2_cuh);
            cost_inter[InterPredDir::PRED_DIR as usize] = cost;
            if cost < cost_best {
                self.core.cu_mode = PredMode::MODE_DIR;
                best_idx = InterPredDir::PRED_DIR;
                cost_inter[best_idx as usize] = cost;
                cost_best = cost;

                self.core.s_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.s_temp_best;
                self.core.c_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.c_temp_best;
                self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.dqp_temp_best;
            }
        }

        0.0
    }

    fn analyze_skip_baseline(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
    ) -> f64 {
        if self.pps.cu_qp_delta_enabled_flag {
            if self.core.cu_qp_delta_code_mode != 2 {
                self.evce_set_qp(self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].prev_QP);
            }
        }

        let cuw = (1 << log2_cuw) as usize;
        let cuh = (1 << log2_cuh) as usize;

        {
            let map_mv = self.map_mv.as_ref().unwrap().borrow();

            evc_get_motion(
                self.core.scup as usize,
                REFP_0,
                &*map_mv,
                &self.refp,
                cuw as usize,
                cuh as usize,
                self.w_scu as usize,
                self.core.avail_cu,
                &mut self.pinter.refi_pred[REFP_0],
                &mut self.pinter.mvp[REFP_0],
            );

            if self.slice_type == SliceType::EVC_ST_B {
                evc_get_motion(
                    self.core.scup as usize,
                    REFP_1,
                    &*map_mv,
                    &self.refp,
                    cuw as usize,
                    cuh as usize,
                    self.w_scu as usize,
                    self.core.avail_cu,
                    &mut self.pinter.refi_pred[REFP_1],
                    &mut self.pinter.mvp[REFP_1],
                );
            }
        }

        self.pinter.mvp_idx[InterPredDir::PRED_SKIP as usize][REFP_0] = 0;
        self.pinter.mvp_idx[InterPredDir::PRED_SKIP as usize][REFP_1] = 0;

        let mut mvp = [[0i16; MV_D]; REFP_NUM];
        let mut refi = [0i8; REFP_NUM];
        let mut cost_best = MAX_COST;
        let mut cy = 0;
        let mut cu = 0;
        let mut cv = 0;

        for idx0 in 0..4 {
            let cnt = if self.slice_type == SliceType::EVC_ST_B {
                4
            } else {
                1
            };
            for idx1 in 0..cnt {
                if idx0 != idx1 {
                    continue;
                }

                mvp[REFP_0][MV_X] = self.pinter.mvp[REFP_0][idx0][MV_X];
                mvp[REFP_0][MV_Y] = self.pinter.mvp[REFP_0][idx0][MV_Y];
                mvp[REFP_1][MV_X] = self.pinter.mvp[REFP_1][idx1][MV_X];
                mvp[REFP_1][MV_Y] = self.pinter.mvp[REFP_1][idx1][MV_Y];

                refi[REFP_0] = self.pinter.refi_pred[REFP_0][idx0];
                refi[REFP_1] = if self.sh.slice_type == SliceType::EVC_ST_B {
                    self.pinter.refi_pred[REFP_1][idx1]
                } else {
                    REFI_INVALID
                };
                if !REFI_IS_VALID(refi[REFP_0]) && !REFI_IS_VALID(refi[REFP_1]) {
                    continue;
                }

                evc_mc(
                    x as i16,
                    y as i16,
                    self.w as i16,
                    self.h as i16,
                    cuw as i16,
                    cuh as i16,
                    &refi,
                    &mvp,
                    &self.refp,
                    &mut self.pinter.pred[InterPredDir::PRED_NUM as usize],
                    self.poc.poc_val,
                );

                if let Some(pic) = &self.pintra.pic_o {
                    let frame = &pic.borrow().frame;
                    let planes = &frame.borrow().planes;
                    cy = evce_ssd_16b(
                        x,
                        y,
                        log2_cuw,
                        log2_cuh,
                        &planes[Y_C].as_region(),
                        &self.pinter.pred[InterPredDir::PRED_NUM as usize][0].data[Y_C],
                    );
                    cu = evce_ssd_16b(
                        x >> 1,
                        y >> 1,
                        log2_cuw - 1,
                        log2_cuh - 1,
                        &planes[U_C].as_region(),
                        &self.pinter.pred[InterPredDir::PRED_NUM as usize][0].data[U_C],
                    );
                    cv = evce_ssd_16b(
                        x >> 1,
                        y >> 1,
                        log2_cuw - 1,
                        log2_cuh - 1,
                        &planes[V_C].as_region(),
                        &self.pinter.pred[InterPredDir::PRED_NUM as usize][0].data[V_C],
                    );
                }

                self.calc_delta_dist_filter_boundary(); //ctx, PIC_MODE(ctx), PIC_ORIG(ctx), cuw, cuh, pi->pred[PRED_NUM][0], cuw, x, y, self.core.avail_lr, 0, 0
                                                        //, refi, mvp, 0, self.core.ats_inter_info, core);
                cy += self.core.delta_dist[Y_C];
                cu += self.core.delta_dist[U_C];
                cv += self.core.delta_dist[V_C];

                let mut cost = cy as f64
                    + (self.dist_chroma_weight[0] * cu as f64)
                    + (self.dist_chroma_weight[1] * cv as f64);

                self.core.s_temp_run = self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
                self.core.c_temp_run = self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];
                self.core.dqp_temp_run = self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2];

                self.core.s_temp_run.bit_reset();
                self.evce_rdo_bit_cnt_cu_skip(self.sh.slice_type, idx0 as u32, idx1 as u32);

                let bit_cnt = self.core.s_temp_run.get_bit_number();
                cost += (self.lambda[0] * bit_cnt as f64);

                if cost < cost_best {
                    cost_best = cost;
                    self.pinter.mvp_idx[InterPredDir::PRED_SKIP as usize][REFP_0] = idx0 as u8;
                    self.pinter.mvp_idx[InterPredDir::PRED_SKIP as usize][REFP_1] = idx1 as u8;
                    self.pinter.mv[InterPredDir::PRED_SKIP as usize][REFP_0][MV_X] =
                        mvp[REFP_0][MV_X];
                    self.pinter.mv[InterPredDir::PRED_SKIP as usize][REFP_0][MV_Y] =
                        mvp[REFP_0][MV_Y];
                    self.pinter.mv[InterPredDir::PRED_SKIP as usize][REFP_1][MV_X] =
                        mvp[REFP_1][MV_X];
                    self.pinter.mv[InterPredDir::PRED_SKIP as usize][REFP_1][MV_Y] =
                        mvp[REFP_1][MV_Y];
                    self.pinter.mvd[InterPredDir::PRED_SKIP as usize][REFP_0][MV_X] = 0;
                    self.pinter.mvd[InterPredDir::PRED_SKIP as usize][REFP_0][MV_Y] = 0;
                    self.pinter.mvd[InterPredDir::PRED_SKIP as usize][REFP_1][MV_X] = 0;
                    self.pinter.mvd[InterPredDir::PRED_SKIP as usize][REFP_1][MV_Y] = 0;
                    self.pinter.refi[InterPredDir::PRED_SKIP as usize][REFP_0] = refi[REFP_0];
                    self.pinter.refi[InterPredDir::PRED_SKIP as usize][REFP_1] = refi[REFP_1];

                    self.core.cost_best = if cost < self.core.cost_best {
                        cost
                    } else {
                        self.core.cost_best
                    };

                    let (pred_skip, pred_num) = self
                        .pinter
                        .pred
                        .split_at_mut(InterPredDir::PRED_NUM as usize);
                    for j in 0..N_C {
                        let size_tmp = (cuw * cuh) >> (if j == 0 { 0 } else { 2 });
                        pred_skip[InterPredDir::PRED_SKIP as usize][0].data[j][0..size_tmp]
                            .copy_from_slice(&pred_num[0][0].data[j][0..size_tmp]);
                    }

                    self.core.s_temp_best = self.core.s_temp_run;
                    self.core.c_temp_best = self.core.c_temp_run;
                    self.core.dqp_temp_best = self.core.dqp_temp_run;
                }
            }
        }

        cost_best
    }

    fn analyze_t_direct(&mut self, x: usize, y: usize, log2_cuw: usize, log2_cuh: usize) -> f64 {
        let refidx = 0;
        let pidx = InterPredDir::PRED_DIR as usize;
        evc_get_mv_dir(
            &self.refp[0],
            self.poc.poc_val,
            self.core.scup as usize
                + ((1 << (self.core.log2_cuw as usize - MIN_CU_LOG2)) - 1)
                + ((1 << (self.core.log2_cuh as usize - MIN_CU_LOG2)) - 1) * self.w_scu as usize,
            self.core.scup as usize,
            self.w_scu,
            self.h_scu,
            &mut self.pinter.mv[pidx],
        );

        self.pinter.mvd[pidx][REFP_0][MV_X] = 0;
        self.pinter.mvd[pidx][REFP_0][MV_Y] = 0;
        self.pinter.mvd[pidx][REFP_1][MV_X] = 0;
        self.pinter.mvd[pidx][REFP_1][MV_Y] = 0;

        self.pinter.refi[pidx][REFP_0] = 0;
        self.pinter.refi[pidx][REFP_1] = 0;

        let mvp_idx = self.pinter.mvp_idx[pidx];
        let cost = self.pinter_residue_rdo(x, y, log2_cuw, log2_cuh, pidx, &mvp_idx);

        self.pinter.nnz_best[pidx].copy_from_slice(&self.core.nnz);

        cost
    }

    fn pinter_residue_rdo(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
        pidx: usize,
        mvp_idx: &[u8],
    ) -> f64 {
        let pred = &self.pinter.pred[pidx];
        let coef = &self.pinter.coef[pidx];

        //TODO:

        0.0
    }
}
