use super::sad::*;
use super::tq::*;
use super::util::*;
use super::*;
use crate::api::frame::*;
use crate::api::*;
use crate::def::*;
use crate::hawktracer::*;
use crate::ipred::*;
use crate::itdq::*;
use crate::picman::*;
use crate::plane::*;
use crate::recon::*;
use crate::region::*;

use std::cell::RefCell;
use std::rc::Rc;

/*****************************************************************************
 * intra prediction structure
 *****************************************************************************/
//#[derive(Default)]
pub(crate) struct EvcePIntra {
    /* temporary prediction buffer */
    pub(crate) pred: CUBuffer<pel>, //[N_C][MAX_CU_DIM];
    pub(crate) pred_cache: [[pel; MAX_CU_DIM]; IntraPredDir::IPD_CNT_B as usize], // only for luma

    /* reconstruction buffer */
    pub(crate) rec: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    pub(crate) coef_tmp: CUBuffer<i16>,  //[N_C][MAX_CU_DIM];
    pub(crate) coef_best: CUBuffer<i16>, //[N_C][MAX_CU_DIM];
    pub(crate) nnz_best: [u16; N_C],
    pub(crate) rec_best: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    /* original (input) picture buffer */
    pub(crate) pic_o: Option<Rc<RefCell<EvcPic>>>,
    /* mode picture buffer */
    pub(crate) pic_m: Option<Rc<RefCell<EvcPic>>>,

    /* QP for luma */
    pub(crate) qp_y: u8,
    /* QP for chroma */
    pub(crate) qp_u: u8,
    pub(crate) qp_v: u8,

    pub(crate) slice_type: SliceType,

    pub(crate) complexity: i64,
    //void              * pdata[4];
    //int               * ndata[4];
}

impl Default for EvcePIntra {
    fn default() -> Self {
        EvcePIntra {
            /* temporary prediction buffer */
            pred: CUBuffer::default(), //[N_C][MAX_CU_DIM];
            pred_cache: [[0; MAX_CU_DIM]; IntraPredDir::IPD_CNT_B as usize], // only for luma

            /* reconstruction buffer */
            rec: CUBuffer::default(), //[N_C][MAX_CU_DIM];

            coef_tmp: CUBuffer::default(),  //[N_C][MAX_CU_DIM];
            coef_best: CUBuffer::default(), //[N_C][MAX_CU_DIM];
            nnz_best: [0; N_C],
            rec_best: CUBuffer::default(), //[N_C][MAX_CU_DIM];

            /* original (input) picture buffer */
            pic_o: None,
            /* mode picture buffer */
            pic_m: None,

            /* QP for luma */
            qp_y: 0,
            /* QP for chroma */
            qp_u: 0,
            qp_v: 0,

            slice_type: SliceType::EVC_ST_UNKNOWN,

            complexity: 0,
            //void              * pdata[4];
            //int               * ndata[4];
        }
    }
}

impl EvceCtx {
    pub(crate) fn pintra_init_frame(&mut self) {
        let pi = &mut self.pintra;

        pi.slice_type = self.slice_type;
        if let Some(pic) = &self.pic[PIC_IDX_ORIG] {
            pi.pic_o = Some(Rc::clone(pic));
        }
        if let Some(pic) = &self.pic[PIC_IDX_MODE] {
            pi.pic_m = Some(Rc::clone(pic));
        }
    }

    pub(crate) fn pintra_analyze_frame(&mut self) {}

    pub(crate) fn pintra_init_lcu(&mut self) {}

    pub(crate) fn pintra_analyze_lcu(&mut self) {}

    #[hawktracer(pintra_analyze_cu)]
    pub(crate) fn pintra_analyze_cu(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
        //coef: &mut CUBuffer<i16>,
        //rec: &CUBuffer<pel>,
    ) -> f64 {
        //int i, j, s_org, s_org_c, s_mod, s_mod_c, cuw, cuh;
        let mut best_ipd = IntraPredDir::IPD_INVALID;
        let mut best_ipd_c = IntraPredDir::IPD_INVALID;
        let mut best_dist_y = 0i32;
        let mut best_dist_c = 0i32;
        let ipm_l2c = 0;
        let chk_bypass = 0;
        let bit_cnt = 0;
        let mut ipred_list = vec![IntraPredDir::IPD_INVALID; IntraPredDir::IPD_CNT_B as usize];
        let mut pred_cnt = IntraPredDir::IPD_CNT_B as usize;
        //pel* org, * mod;
        //pel* org_cb, * org_cr;
        //pel* mod_cb, * mod_cr;
        let mut cost_t = MAX_COST;
        let mut cost = MAX_COST;
        let mut sec_best_ipd = IntraPredDir::IPD_INVALID;

        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;
        let cuwxh = cuw * cuh;

        if let Some(pic) = &self.pintra.pic_m {
            let frame = &pic.borrow().frame;
            let planes = &frame.borrow().planes;

            /* Y */

            evc_get_nbr_b(
                x,
                y,
                cuw,
                cuh,
                &planes[Y_C].as_region(),
                self.core.avail_cu,
                &mut self.core.nb.data[Y_C],
                self.core.scup as usize,
                &self.map_scu,
                self.w_scu as usize,
                self.h_scu as usize,
                Y_C,
                self.pps.constrained_intra_pred_flag,
            );

            evc_get_nbr_b(
                x >> 1,
                y >> 1,
                cuw >> 1,
                cuh >> 1,
                &planes[U_C].as_region(),
                self.core.avail_cu,
                &mut self.core.nb.data[U_C],
                self.core.scup as usize,
                &self.map_scu,
                self.w_scu as usize,
                self.h_scu as usize,
                U_C,
                self.pps.constrained_intra_pred_flag,
            );

            evc_get_nbr_b(
                x >> 1,
                y >> 1,
                cuw >> 1,
                cuh >> 1,
                &planes[V_C].as_region(),
                self.core.avail_cu,
                &mut self.core.nb.data[V_C],
                self.core.scup as usize,
                &self.map_scu,
                self.w_scu as usize,
                self.h_scu as usize,
                V_C,
                self.pps.constrained_intra_pred_flag,
            );
        }

        self.core.mpm_b_list = evc_get_mpm_b(
            self.core.x_scu,
            self.core.y_scu,
            &self.map_scu,
            &self.map_ipm,
            self.core.scup,
            self.w_scu,
        );

        if evc_check_luma(&self.core.tree_cons) {
            pred_cnt = self.make_ipred_list(x, y, log2_cuw, log2_cuh, &mut ipred_list);
            if pred_cnt == 0 {
                return MAX_COST;
            }

            for j in 0..pred_cnt {
                let mut dist_t = 0;
                let mut dist_tc = 0;

                let i = ipred_list[j];
                self.core.ipm[0] = i;
                self.core.ipm[1] = IntraPredDir::IPD_INVALID;
                cost_t = self.pintra_residue_rdo(x, y, log2_cuw, log2_cuh, &mut dist_t, false);

                EVC_TRACE_COUNTER(&mut self.core.bs_temp.tracer);
                EVC_TRACE(&mut self.core.bs_temp.tracer, "Luma mode ");
                EVC_TRACE(&mut self.core.bs_temp.tracer, i as u8);
                EVC_TRACE(&mut self.core.bs_temp.tracer, "  cost is ");
                EVC_TRACE(&mut self.core.bs_temp.tracer, cost_t as i64);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " \n");

                if cost_t < cost {
                    cost = cost_t;
                    best_dist_y = dist_t;

                    if sec_best_ipd != best_ipd {
                        sec_best_ipd = best_ipd;
                    }

                    best_ipd = i;

                    self.pintra.coef_best.data[Y_C][0..cuwxh]
                        .copy_from_slice(&self.core.ctmp.data[Y_C][0..cuwxh]);
                    self.pintra.rec_best.data[Y_C][0..cuwxh]
                        .copy_from_slice(&self.pintra.rec.data[Y_C][0..cuwxh]);

                    self.pintra.nnz_best[Y_C] = self.core.nnz[Y_C];
                    self.core.s_temp_prev_comp_best = self.core.s_temp_run;
                    self.core.c_temp_prev_comp_best = self.core.c_temp_run;
                }
            }
        } else {
            let luma_cup = evc_get_luma_cup(
                0,
                0,
                PEL2SCU(cuw) as u16,
                PEL2SCU(cuh) as u16,
                PEL2SCU(cuw) as u16,
            );
            let luma_flags = self.core.cu_data_temp[log2_cuw as usize - 2][log2_cuh as usize - 2]
                .map_scu[luma_cup as usize];
            assert!(luma_flags.GET_IF() != 0);
            if luma_flags.GET_IF() != 0 {
                best_ipd = self.core.cu_data_temp[log2_cuw as usize - 2][log2_cuh as usize - 2].ipm
                    [0][luma_cup as usize];
            } else {
                best_ipd = IntraPredDir::IPD_DC_B;
            }
        }

        if evc_check_chroma(&self.core.tree_cons) {
            let mut dist_tc = 0i32;
            self.core.ipm[0] = best_ipd;
            self.core.ipm[1] = best_ipd;

            cost_t = self.pintra_residue_rdo(x, y, log2_cuw, log2_cuh, &mut dist_tc, true);

            best_ipd_c = self.core.ipm[1];
            best_dist_c = dist_tc;
            for j in U_C..N_C {
                let size_tmp = (cuw * cuh) >> (if j == 0 { 0 } else { 2 });
                self.pintra.coef_best.data[j][0..size_tmp]
                    .copy_from_slice(&self.core.ctmp.data[j][0..size_tmp]);
                self.pintra.rec_best.data[j][0..size_tmp]
                    .copy_from_slice(&self.pintra.rec.data[j][0..size_tmp]);

                self.pintra.nnz_best[j] = self.core.nnz[j];
            }
        }

        let start_comp = if evc_check_luma(&self.core.tree_cons) {
            Y_C
        } else {
            U_C
        };
        let end_comp = if evc_check_chroma(&self.core.tree_cons) {
            N_C
        } else {
            U_C
        };
        for j in start_comp..end_comp {
            let size_tmp = (cuw * cuh) >> (if j == 0 { 0 } else { 2 });
            self.core.ctmp.data[j][0..size_tmp]
                .copy_from_slice(&self.pintra.coef_best.data[j][0..size_tmp]);
            self.pintra.rec.data[j][0..size_tmp]
                .copy_from_slice(&self.pintra.rec_best.data[j][0..size_tmp]);
            self.core.nnz[j] = self.pintra.nnz_best[j];
            //rec[j] = pi->rec[j];
            //s_rec[j] = cuw >> (j == 0 ? 0 : 1);
        }

        if evc_check_luma(&self.core.tree_cons) {
            self.core.ipm[0] = best_ipd;
        }
        if evc_check_chroma(&self.core.tree_cons) {
            self.core.ipm[1] = best_ipd_c;
            assert!(best_ipd_c != IntraPredDir::IPD_INVALID);
        }

        /* cost calculation */
        self.core.s_temp_run = self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
        self.core.c_temp_run = self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

        self.core.dqp_temp_run = self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2];

        self.core.s_temp_run.bit_reset();
        self.evce_rdo_bit_cnt_cu_intra(self.sh.slice_type);

        let bit_cnt = self.core.s_temp_run.get_bit_number();
        cost = (self.lambda[0] * bit_cnt as f64);

        self.core.dist_cu = 0;
        if evc_check_luma(&self.core.tree_cons) {
            cost += best_dist_y as f64;
            self.core.dist_cu += best_dist_y;
        }
        if evc_check_chroma(&self.core.tree_cons) {
            cost += best_dist_c as f64;
            self.core.dist_cu += best_dist_c;
        }

        self.core.s_temp_best = self.core.s_temp_run;
        self.core.c_temp_best = self.core.c_temp_run;

        self.core.dqp_temp_best = self.core.dqp_temp_run;

        cost
    }

    fn make_ipred_list(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
        ipred_list: &mut [IntraPredDir],
    ) -> usize {
        let pi = &mut self.pintra;
        let core = &mut self.core;

        //int cuw, cuh, pred_cnt, i, j;
        let mut cost = 0.0f64;
        let mut cost_satd = 0u32;
        let ipd_rdo_cnt = if (log2_cuw as i8 - log2_cuh as i8).abs() >= 2 {
            IPD_RDO_CNT - 1
        } else {
            IPD_RDO_CNT
        };

        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;

        let mut cand_cost = vec![MAX_COST; IPD_RDO_CNT];
        let mut cand_satd_cost = vec![u32::MAX; IPD_RDO_CNT];
        for i in 0..ipd_rdo_cnt {
            ipred_list[i] = IntraPredDir::IPD_DC_B;
        }

        for i in 0..IntraPredDir::IPD_CNT_B as u8 {
            let mut shift = 0;
            let pred_buf = &mut pi.pred_cache[i as usize];

            evc_ipred_b(
                &core.nb.data[Y_C][0][2..],
                &core.nb.data[Y_C][1][cuh..],
                core.nb.data[Y_C][1][cuh - 1],
                pred_buf,
                i.into(),
                cuw,
                cuh,
            );

            if let Some(pic) = &pi.pic_o {
                let frame = &pic.borrow().frame;
                let org = &frame.borrow().planes[Y_C];
                cost_satd = evce_satd_16b(x, y, cuw, cuh, &org.as_region(), pred_buf);
                cost = cost_satd as f64;
            }
            core.s_temp_run = core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
            core.c_temp_run = core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

            core.s_temp_run.bit_reset();

            evce_eco_intra_dir_b(
                &mut core.bs_temp,
                &mut core.s_temp_run,
                &mut core.c_temp_run,
                i,
                core.mpm_b_list,
            );

            let bit_cnt = core.s_temp_run.get_bit_number();
            cost += self.sqrt_lambda[0] * bit_cnt as f64;

            while shift < ipd_rdo_cnt && cost < cand_cost[ipd_rdo_cnt - 1 - shift] {
                shift += 1;
            }

            if shift != 0 {
                for j in 1..shift {
                    ipred_list.swap(ipd_rdo_cnt - j, ipd_rdo_cnt - 1 - j);
                    cand_cost.swap(ipd_rdo_cnt - j, ipd_rdo_cnt - 1 - j);
                    cand_satd_cost.swap(ipd_rdo_cnt - j, ipd_rdo_cnt - 1 - j);
                }
                ipred_list[ipd_rdo_cnt - shift] = i.into();
                cand_cost[ipd_rdo_cnt - shift] = cost;
                cand_satd_cost[ipd_rdo_cnt - shift] = cost_satd;
            }
        }

        let mut pred_cnt = ipd_rdo_cnt as i8;
        let mut i = ipd_rdo_cnt as i8 - 1;
        while i >= 0 {
            if cand_satd_cost[i as usize] as f32 > core.inter_satd as f32 * 1.1 {
                pred_cnt -= 1;
            } else {
                break;
            }
            i -= 1;
        }

        return std::cmp::min(pred_cnt as usize, ipd_rdo_cnt);
    }

    fn pintra_residue_rdo(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
        //coef: &mut CUBuffer<i16>,
        dist: &mut i32,
        chroma: bool,
    ) -> f64 {
        let mut cost = 0f64;
        let mut tmp_cbf_l = 0;

        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;

        if !chroma {
            assert!(evc_check_luma(&self.core.tree_cons));

            if let Some(pic) = &self.pintra.pic_o {
                let frame = &pic.borrow().frame;
                let planes = &frame.borrow().planes;
                evce_diff_16b(
                    x,
                    y,
                    log2_cuw,
                    log2_cuh,
                    &planes[Y_C].as_region(),
                    &self.pintra.pred_cache[self.core.ipm[0] as usize],
                    &mut self.pintra.coef_tmp.data[Y_C],
                );
            }

            evce_sub_block_tq(
                &mut self.pintra.coef_tmp,
                log2_cuw,
                log2_cuh,
                self.core.qp_y,
                self.core.qp_u,
                self.core.qp_v,
                self.pintra.slice_type,
                &mut self.core.nnz,
                true,
                self.lambda[0],
                self.lambda[1],
                self.lambda[2],
                TQC_RUN::RUN_L as u8,
                &self.core.tree_cons,
                &self.core.rdoq_est,
            );

            //if core->ats_intra_cu != 0 &&self.core.nnz[Y_C] == 0 {
            //   return MAX_COST;
            //}

            self.core.ctmp.data[Y_C][0..cuw * cuh]
                .copy_from_slice(&self.pintra.coef_tmp.data[Y_C][0..cuw * cuh]);

            self.core.s_temp_run = self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
            self.core.c_temp_run = self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

            self.core.dqp_temp_run = self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2];
            self.core.s_temp_run.bit_reset();
            self.evce_rdo_bit_cnt_cu_intra_luma(self.sh.slice_type);
            let bit_cnt = self.core.s_temp_run.get_bit_number();

            let is_coef = [
                self.core.nnz[Y_C] != 0,
                self.core.nnz[U_C] != 0,
                self.core.nnz[V_C] != 0,
            ];
            evc_sub_block_itdq(
                &mut self.core.bs_temp.tracer,
                &mut self.pintra.coef_tmp.data,
                log2_cuw as u8,
                log2_cuh as u8,
                self.core.qp_y,
                self.core.qp_u,
                self.core.qp_v,
                &is_coef,
            );

            evc_recon(
                &mut self.core.bs_temp.tracer,
                &self.pintra.coef_tmp.data[Y_C],
                &self.pintra.pred_cache[self.core.ipm[0] as usize],
                self.core.nnz[Y_C] != 0,
                cuw,
                cuh,
                &mut self.pintra.rec.data[Y_C],
                Y_C,
            );

            if let Some(pic) = &self.pintra.pic_o {
                let frame = &pic.borrow().frame;
                let planes = &frame.borrow().planes;
                cost += evce_ssd_16b(
                    x,
                    y,
                    log2_cuw,
                    log2_cuh,
                    &planes[Y_C].as_region(),
                    &self.pintra.rec.data[Y_C],
                ) as f64;
            }

            self.calc_delta_dist_filter_boundary(
                x as i16,
                y as i16,
                log2_cuw,
                log2_cuh,
                self.core.avail_lr,
                true,
                true,
                0,
                self.core.nnz[Y_C] != 0,
                &[],
                &[],
            );

            cost += self.core.delta_dist[Y_C] as f64;
            *dist = cost as i32;
            cost += (self.lambda[0] * bit_cnt as f64);
        } else {
            assert!(evc_check_chroma(&self.core.tree_cons));

            evc_ipred_b(
                &self.core.nb.data[U_C][0][2..],
                &self.core.nb.data[U_C][1][(cuh >> 1) as usize..],
                self.core.nb.data[U_C][1][(cuh >> 1) as usize - 1],
                &mut self.pintra.pred.data[U_C],
                self.core.ipm[1],
                cuw as usize >> 1,
                cuh as usize >> 1,
            );

            evc_ipred_b(
                &self.core.nb.data[V_C][0][2..],
                &self.core.nb.data[V_C][1][(cuh >> 1) as usize..],
                self.core.nb.data[V_C][1][(cuh >> 1) as usize - 1],
                &mut self.pintra.pred.data[V_C],
                self.core.ipm[1],
                cuw as usize >> 1,
                cuh as usize >> 1,
            );

            if let Some(pic) = &self.pintra.pic_o {
                let frame = &pic.borrow().frame;
                let planes = &frame.borrow().planes;
                evce_diff_16b(
                    x >> 1,
                    y >> 1,
                    log2_cuw - 1,
                    log2_cuh - 1,
                    &planes[U_C].as_region(),
                    &self.pintra.pred.data[U_C],
                    &mut self.pintra.coef_tmp.data[U_C],
                );
                evce_diff_16b(
                    x >> 1,
                    y >> 1,
                    log2_cuw - 1,
                    log2_cuh - 1,
                    &planes[V_C].as_region(),
                    &self.pintra.pred.data[V_C],
                    &mut self.pintra.coef_tmp.data[V_C],
                );
            }

            evce_sub_block_tq(
                &mut self.pintra.coef_tmp,
                log2_cuw,
                log2_cuh,
                self.core.qp_y,
                self.core.qp_u,
                self.core.qp_v,
                self.pintra.slice_type,
                &mut self.core.nnz,
                true,
                self.lambda[0],
                self.lambda[1],
                self.lambda[2],
                TQC_RUN::RUN_CB as u8 | TQC_RUN::RUN_CR as u8,
                &self.core.tree_cons,
                &self.core.rdoq_est,
            );

            self.core.ctmp.data[U_C][0..(cuw * cuh) >> 2]
                .copy_from_slice(&self.pintra.coef_tmp.data[U_C][0..(cuw * cuh) >> 2]);
            self.core.ctmp.data[V_C][0..(cuw * cuh) >> 2]
                .copy_from_slice(&self.pintra.coef_tmp.data[V_C][0..(cuw * cuh) >> 2]);

            let is_coef = [
                self.core.nnz[Y_C] != 0,
                self.core.nnz[U_C] != 0,
                self.core.nnz[V_C] != 0,
            ];
            evc_sub_block_itdq(
                &mut self.core.bs_temp.tracer,
                &mut self.pintra.coef_tmp.data,
                log2_cuw as u8,
                log2_cuh as u8,
                self.core.qp_y,
                self.core.qp_u,
                self.core.qp_v,
                &is_coef,
            );

            evc_recon(
                &mut self.core.bs_temp.tracer,
                &self.pintra.coef_tmp.data[U_C],
                &self.pintra.pred.data[U_C],
                self.core.nnz[U_C] != 0,
                cuw >> 1,
                cuh >> 1,
                &mut self.pintra.rec.data[U_C],
                U_C,
            );
            evc_recon(
                &mut self.core.bs_temp.tracer,
                &self.pintra.coef_tmp.data[V_C],
                &self.pintra.pred.data[V_C],
                self.core.nnz[V_C] != 0,
                cuw >> 1,
                cuh >> 1,
                &mut self.pintra.rec.data[V_C],
                V_C,
            );

            self.core.s_temp_run.bit_reset();

            self.evce_rdo_bit_cnt_cu_intra_chroma(self.sh.slice_type);

            let bit_cnt = self.core.s_temp_run.get_bit_number();

            if let Some(pic) = &self.pintra.pic_o {
                let frame = &pic.borrow().frame;
                let planes = &frame.borrow().planes;
                cost += self.dist_chroma_weight[0]
                    * evce_ssd_16b(
                        x >> 1,
                        y >> 1,
                        log2_cuw - 1,
                        log2_cuh - 1,
                        &planes[U_C].as_region(),
                        &self.pintra.rec.data[U_C],
                    ) as f64;
                cost += self.dist_chroma_weight[1]
                    * evce_ssd_16b(
                        x >> 1,
                        y >> 1,
                        log2_cuw - 1,
                        log2_cuh - 1,
                        &planes[V_C].as_region(),
                        &self.pintra.rec.data[V_C],
                    ) as f64;
            }

            self.calc_delta_dist_filter_boundary(
                x as i16,
                y as i16,
                log2_cuw,
                log2_cuh,
                self.core.avail_lr,
                true,
                true,
                0,
                //if !evc_check_luma(&self.core.tree_cons) {
                //self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].nnz[Y_C] != 0
                //} else {
                self.core.nnz[Y_C] != 0,
                //},
                &[],
                &[],
            );

            cost += (self.core.delta_dist[U_C] as f64 * self.dist_chroma_weight[0])
                + (self.core.delta_dist[V_C] as f64 * self.dist_chroma_weight[1]);

            *dist = cost as i32;

            if let Some(pic) = &self.pintra.pic_o {
                let frame = &pic.borrow().frame;
                let planes = &frame.borrow().planes;
                cost += evce_ssd_16b(
                    x,
                    y,
                    log2_cuw,
                    log2_cuh,
                    &planes[Y_C].as_region(),
                    &self.pintra.rec.data[Y_C],
                ) as f64;
            }

            cost += (self.lambda[0] * bit_cnt as f64);
        }

        return cost;
    }
}
