use super::sad::*;
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
        let pi = &mut self.pintra;
        let core = &mut self.core;

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

        if let Some(pic) = &pi.pic_m {
            let frame = &pic.borrow().frame;
            let planes = &frame.borrow().planes;
            /* Y */

            evc_get_nbr_b(
                x,
                y,
                cuw,
                cuh,
                &planes[Y_C].as_region(),
                core.avail_cu,
                &mut core.nb.data[Y_C],
                core.scup as usize,
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
                core.avail_cu,
                &mut core.nb.data[U_C],
                core.scup as usize,
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
                core.avail_cu,
                &mut core.nb.data[V_C],
                core.scup as usize,
                &self.map_scu,
                self.w_scu as usize,
                self.h_scu as usize,
                V_C,
                self.pps.constrained_intra_pred_flag,
            );
        }

        if evc_check_luma(&core.tree_cons) {
            pred_cnt = self.make_ipred_list(log2_cuw, log2_cuh, &mut ipred_list);
            if pred_cnt == 0 {
                return MAX_COST;
            }
        } else {
            /*int luma_cup = evc_get_luma_cup(0, 0, PEL2SCU(cuw), PEL2SCU(cuh), PEL2SCU(cuw));
            u32 luma_flags = core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].map_scu[luma_cup];
            evc_assert(MCU_GET_IF(luma_flags) || MCU_GET_IBC(luma_flags));
            if (MCU_GET_IF(luma_flags))
            {
                best_ipd = core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].ipm[0][luma_cup];
            }
            else
            {
                best_ipd = IPD_DC;
            }*/
        }
        /*
                if evc_check_chroma(&core.tree_cons) {
                    s32 dist_tc = 0;
                    core.ipm[0] = best_ipd;
                    core.ipm[1] = best_ipd;
                    cost_t = pintra_residue_rdo(ctx, core, org, org_cb, org_cr, s_org, s_org_c, log2_cuw, log2_cuh, coef, &dist_tc, 1, x, y);

                    best_ipd_c = core.ipm[1];
                    best_dist_c = dist_tc;
                    for(j = U_C; j < N_C; j++)
                    {
                        int size_tmp = (cuw * cuh) >> (j == 0 ? 0 : 2);
                        evc_mcpy(pi->coef_best[j], coef[j], size_tmp * sizeof(s16));
                        evc_mcpy(pi->rec_best[j], pi->rec[j], size_tmp * sizeof(pel));

                        pi->nnz_best[j] = core.nnz[j];
                        evc_mcpy(pi->nnz_sub_best[j], core.nnz_sub[j], sizeof(int) * MAX_SUB_TB_NUM);
                    }
                }

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
}
