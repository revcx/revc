use super::sad::*;
use super::util::*;
use super::*;
use crate::api::frame::*;
use crate::api::*;
use crate::def::*;
use crate::ipred::*;
use crate::picman::*;
use crate::plane::*;
use crate::region::*;

use std::cell::RefCell;
use std::rc::Rc;

/*****************************************************************************
 * intra prediction structure
 *****************************************************************************/
//#[derive(Default)]
pub(crate) struct EvcePIntra {
    /* temporary prediction buffer */
    pred: CUBuffer<pel>, //[N_C][MAX_CU_DIM];
    pred_cache: [[pel; MAX_CU_DIM]; IntraPredDir::IPD_CNT_B as usize], // only for luma

    /* reconstruction buffer */
    pub(crate) rec: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    coef_tmp: CUBuffer<i16>,  //[N_C][MAX_CU_DIM];
    coef_best: CUBuffer<i16>, //[N_C][MAX_CU_DIM];
    nnz_best: [u16; N_C],
    rec_best: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    /* original (input) picture buffer */
    pic_o: Option<Rc<RefCell<EvcPic>>>,
    /* mode picture buffer */
    pic_m: Option<Rc<RefCell<EvcPic>>>,

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

    pub(crate) fn pintra_analyze_cu(
        &mut self,
        x: usize,
        y: usize,
        log2_cuw: usize,
        log2_cuh: usize,
        mi: &EvceMode,
        coef: &CUBuffer<i16>,
        rec: &CUBuffer<pel>,
        /*map_scu: &[MCU],
        w_scu: usize,
        h_scu: usize,
        constrained_intra_pred_flag: bool,*/
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
        if evc_check_luma(&self.core.tree_cons) {
            pred_cnt = self.make_ipred_list(log2_cuw, log2_cuh, &mut ipred_list);
            if pred_cnt == 0 {
                return MAX_COST;
            }

            for j in 0..pred_cnt {
                let mut dist_t = 0;
                let mut dist_tc = 0;

                let i = ipred_list[j];
                self.core.ipm[0] = i;
                self.core.ipm[1] = IntraPredDir::IPD_INVALID;
                cost_t =
                    self.pintra_residue_rdo(log2_cuw, log2_cuh, &coef, &mut dist_t, false, x, y);
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

            cost_t = self.pintra_residue_rdo(log2_cuw, log2_cuh, coef, &mut dist_tc, true, x, y);

            best_ipd_c = self.core.ipm[1];
            best_dist_c = dist_tc;
            for j in U_C..N_C {
                let size_tmp = (cuw * cuh) >> (if j == 0 { 0 } else { 2 });
                self.pintra.coef_best.data[j][0..size_tmp]
                    .copy_from_slice(&coef.data[j][0..size_tmp]);
                self.pintra.rec_best.data[j][0..size_tmp]
                    .copy_from_slice(&self.pintra.rec.data[j][0..size_tmp]);

                self.pintra.nnz_best[j] = self.core.nnz[j];
            }
        }
        /*
                int start_comp = evce_check_luma(ctx, core) ? Y_C : U_C;
                int end_comp = evce_check_chroma(ctx, core) ? N_C : U_C;
                for (j = start_comp; j < end_comp; j++)
                {
                    int size_tmp = (cuw * cuh) >> (j == 0 ? 0 : 2);
                    evc_mcpy(coef[j], pi->coef_best[j], size_tmp * sizeof(u16));
                    evc_mcpy(pi->rec[j], pi->rec_best[j], size_tmp * sizeof(pel));
                    core.nnz[j] = pi->nnz_best[j];
                    rec[j] = pi->rec[j];
                    s_rec[j] = cuw >> (j == 0 ? 0 : 1);
                }

                if (evce_check_luma(ctx, core))
                {
                    core.ipm[0] = best_ipd;
                }
                if (evce_check_chroma(ctx, core))
                {
                    core.ipm[1] = best_ipd_c;
                    evc_assert(best_ipd_c != IPD_INVALID);
                }

                /* cost calculation */
                SBAC_LOAD(core.s_temp_run, core.s_curr_best[log2_cuw - 2][log2_cuh - 2]);
                DQP_STORE(core.dqp_temp_run, core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2]);

                evce_sbac_bit_reset(&core.s_temp_run);
                evce_rdo_bit_cnt_cu_intra(ctx, core, ctx->sh.slice_type, core.scup, coef);

                bit_cnt = evce_get_bit_number(&core.s_temp_run);
                cost = RATE_TO_COST_LAMBDA(ctx->lambda[0], bit_cnt);

                core.dist_cu = 0;
                if (evce_check_luma(ctx, core))
                {
                    cost += best_dist_y;
                    core.dist_cu += best_dist_y;
                }
                if (evce_check_chroma(ctx, core))
                {
                    cost += best_dist_c;
                    core.dist_cu += best_dist_c;
                }

                SBAC_STORE(core.s_temp_best, core.s_temp_run);
                DQP_STORE(core.dqp_temp_best, core.dqp_temp_run);
        */
        return cost;
    }

    fn make_ipred_list(
        &mut self,
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
                core.nb.data[Y_C][2][cuh - 1],
                pred_buf,
                i.into(),
                cuw,
                cuh,
            );

            if let Some(pic) = &pi.pic_o {
                let frame = &pic.borrow().frame;
                let org = &frame.borrow().planes[Y_C];
                cost_satd = evce_satd_16b(cuw, cuh, &org.as_region(), pred_buf);
                cost = cost_satd as f64;
            }
            core.s_temp_run = core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
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
        log2_cuw: usize,
        log2_cuh: usize,
        coef: &CUBuffer<i16>,
        dist: &mut i32,
        chroma: bool,
        x: usize,
        y: usize,
    ) -> f64 {
        let mut cost = 0f64;
        let mut tmp_cbf_l = 0;

        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;

        if let Some(pic) = &self.pintra.pic_m {
            let frame = &pic.borrow().frame;
            let planes = &frame.borrow().planes;

            if !chroma {
                assert!(evc_check_luma(&self.core.tree_cons));
                let pred = &self.pintra.pred_cache[self.core.ipm[0] as usize];
                evce_diff_16b(
                    cuw,
                    cuh,
                    &planes[Y_C].as_region(),
                    pred,
                    &mut self.pintra.coef_tmp.data[Y_C],
                );

            /*
            evce_sub_block_tq(pi->coef_tmp, log2_cuw, log2_cuh, core->qp_y, core->qp_u, core->qp_v, pi->slice_type, core->nnz
                              , core->nnz_sub, 1, ctx->lambda[0], ctx->lambda[1], ctx->lambda[2], RUN_L, ctx->sps.tool_cm_init, ctx->sps.tool_iqt, core->ats_intra_cu, core->ats_mode, 0, ctx->sps.tool_adcc
                              , core->tree_cons, core);

            if (core->ats_intra_cu != 0 && core->nnz[Y_C] == 0)
            {
                return MAX_COST;
            }
            evc_mcpy(coef[Y_C], pi->coef_tmp[Y_C], sizeof(u16) * (cuw * cuh));

            SBAC_LOAD(core->s_temp_run, core->s_curr_best[log2_cuw - 2][log2_cuh - 2]);
            DQP_LOAD(core->dqp_temp_run, core->dqp_curr_best[log2_cuw - 2][log2_cuh - 2]);
            evce_sbac_bit_reset(&core->s_temp_run);
            evce_rdo_bit_cnt_cu_intra_luma(ctx, core, ctx->sh.slice_type, core->scup, pi->coef_tmp);
            bit_cnt = evce_get_bit_number(&core->s_temp_run);

            evc_sub_block_itdq(pi->coef_tmp, log2_cuw, log2_cuh, core->qp_y, core->qp_u, core->qp_v, core->nnz, core->nnz_sub, ctx->sps.tool_iqt, core->ats_intra_cu, core->ats_mode, core->ats_inter_info);
            evc_recon(pi->coef_tmp[Y_C], pred, core->nnz[Y_C], cuw, cuh, cuw, pi->rec[Y_C], core->ats_inter_info);

            cost += evce_ssd_16b(log2_cuw, log2_cuh, pi->rec[Y_C], org_luma, cuw, s_org);

            calc_delta_dist_filter_boundary(ctx, PIC_MODE(ctx), PIC_ORIG(ctx), cuw, cuh, pi->rec, cuw, x, y, core->avail_lr, 1, core->nnz[Y_C] != 0, NULL, NULL, 0, core->ats_inter_info, core);
            cost += core->delta_dist[Y_C];
            *dist = (s32)cost;
            cost += RATE_TO_COST_LAMBDA(ctx->lambda[0], bit_cnt);*/
            } else {
                assert!(evc_check_chroma(&self.core.tree_cons));

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
                /*
                evce_diff_16b(log2_cuw - 1, log2_cuh - 1, org_cb, pi -> pred[U_C], s_org_c, cuw >> 1, cuw >> 1, pi -> coef_tmp[U_C]);
                evce_diff_16b(log2_cuw - 1, log2_cuh - 1, org_cr, pi -> pred[V_C], s_org_c, cuw >> 1, cuw >> 1, pi -> coef_tmp[V_C]);

                evce_sub_block_tq(pi -> coef_tmp, log2_cuw, log2_cuh, core -> qp_y, core -> qp_u, core -> qp_v, pi -> slice_type, core -> nnz, core -> nnz_sub
                , 1, ctx -> lambda[0], ctx -> lambda[1], ctx -> lambda[2], RUN_CB | RUN_CR, ctx -> sps.tool_cm_init, ctx -> sps.tool_iqt
                , core -> ats_intra_cu, core -> ats_mode, 0, ctx -> sps.tool_adcc, core -> tree_cons, core);

                evc_mcpy(coef[U_C], pi -> coef_tmp[U_C], sizeof(u16) * (cuw * cuh) >> 2);
                evc_mcpy(coef[V_C], pi -> coef_tmp[V_C], sizeof(u16) * (cuw * cuh) >> 2);

                evc_sub_block_itdq(pi -> coef_tmp, log2_cuw, log2_cuh, core -> qp_y, core -> qp_u, core -> qp_v, core -> nnz, core -> nnz_sub, ctx -> sps.tool_iqt, core -> ats_intra_cu, core -> ats_mode, 0);

                evc_recon(pi -> coef_tmp[U_C], pi -> pred[U_C], core -> nnz[U_C], cuw >> 1, cuh >> 1, cuw >> 1, pi -> rec[U_C], 0);
                evc_recon(pi -> coef_tmp[V_C], pi -> pred[V_C], core -> nnz[V_C], cuw >> 1, cuh >> 1, cuw >> 1, pi -> rec[V_C], 0);

                evce_sbac_bit_reset(&core -> s_temp_run);

                evce_rdo_bit_cnt_cu_intra_chroma(ctx, core, ctx -> sh.slice_type, core -> scup, coef);

                bit_cnt = evce_get_bit_number(&core -> s_temp_run);

                cost += ctx -> dist_chroma_weight[0] * evce_ssd_16b(log2_cuw - 1, log2_cuh - 1, pi -> rec[U_C], org_cb, cuw >> 1, s_org_c);
                cost += ctx -> dist_chroma_weight[1] * evce_ssd_16b(log2_cuw - 1, log2_cuh - 1, pi -> rec[V_C], org_cr, cuw >> 1, s_org_c);
                {
                    calc_delta_dist_filter_boundary(ctx, PIC_MODE(ctx), PIC_ORIG(ctx), cuw, cuh, pi -> rec, cuw, x, y, core -> avail_lr, 1,
                    !evce_check_luma(ctx, core)?
                    core -> cu_data_temp[log2_cuw - 2][log2_cuh - 2].nnz[Y_C] != 0:
                    core -> nnz[Y_C] != 0, NULL, NULL, 0, core -> ats_inter_info, core);
                    cost += (core -> delta_dist[U_C] * ctx -> dist_chroma_weight[0]) + (core -> delta_dist[V_C] * ctx -> dist_chroma_weight[1]);
                }
                *dist = (s32)
                cost;

                cost += evce_ssd_16b(log2_cuw, log2_cuh, pi -> rec[Y_C], org_luma, cuw, s_org);
                cost += RATE_TO_COST_LAMBDA(ctx -> lambda[0], bit_cnt);*/
            }
        }

        return cost;
    }
}
