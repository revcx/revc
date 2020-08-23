use super::eco::*;
use super::sad::*;
use super::util::*;
use super::*;
use crate::api::*;
use crate::def::*;
use crate::plane::*;
use crate::tracer::*;

#[derive(Default)]
pub(crate) struct EvceCUData {
    pub(crate) split_mode: LcuSplitMode,
    pub(crate) qp_y: Vec<u8>,
    pub(crate) qp_u: Vec<u8>,
    pub(crate) qp_v: Vec<u8>,
    pub(crate) pred_mode: Vec<PredMode>,
    pub(crate) pred_mode_chroma: Vec<PredMode>,
    pub(crate) ipm: Vec<Vec<IntraPredDir>>,
    pub(crate) skip_flag: Vec<bool>,
    pub(crate) refi: Vec<[i8; REFP_NUM]>,
    pub(crate) mvp_idx: Vec<Vec<u8>>,
    pub(crate) mv: Vec<[[i16; MV_D]; REFP_NUM]>, //[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
    pub(crate) mvd: Vec<Vec<Vec<i16>>>,          //[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
    pub(crate) nnz: Vec<Vec<u16>>,               //[N_C];
    pub(crate) map_scu: Vec<MCU>,
    pub(crate) map_cu_mode: Vec<MCU>,
    pub(crate) depth: Vec<i8>,
    pub(crate) coef: Vec<Vec<i16>>, //[N_C];
    pub(crate) reco: Vec<Vec<pel>>, //[N_C];

    #[cfg(feature = "trace_cudata")]
    pub(crate) trace_idx: Vec<u64>, // MAX_CU_CNT_IN_LCU],
}

impl EvceCUData {
    pub(crate) fn new(log2_cuw: u8, log2_cuh: u8) -> Self {
        let cuw_scu = 1 << log2_cuw;
        let cuh_scu = 1 << log2_cuh;

        let cu_cnt = cuw_scu * cuh_scu;
        let pixel_cnt = cu_cnt << 4;

        let mut coef = Vec::with_capacity(N_C);
        let mut reco = Vec::with_capacity(N_C);
        for i in 0..N_C {
            let chroma = if i > 0 { 1 } else { 0 };
            coef.push(vec![0; pixel_cnt >> (chroma * 2)]);
            reco.push(vec![0; pixel_cnt >> (chroma * 2)]);
        }

        EvceCUData {
            split_mode: LcuSplitMode::default(),
            qp_y: vec![0; cu_cnt],
            qp_u: vec![0; cu_cnt],
            qp_v: vec![0; cu_cnt],
            pred_mode: vec![PredMode::MODE_INTRA; cu_cnt],
            pred_mode_chroma: vec![PredMode::MODE_INTRA; cu_cnt],
            ipm: vec![vec![IntraPredDir::IPD_DC_B; cu_cnt]; 2],
            skip_flag: vec![false; cu_cnt],
            refi: vec![[0; REFP_NUM]; cu_cnt],
            mvp_idx: vec![vec![0; REFP_NUM]; cu_cnt],
            mv: vec![[[0; MV_D]; REFP_NUM]; cu_cnt],
            mvd: vec![vec![vec![0; MV_D]; REFP_NUM]; cu_cnt],
            nnz: vec![vec![0; cu_cnt]; N_C],
            map_scu: vec![MCU::default(); cu_cnt],
            map_cu_mode: vec![MCU::default(); cu_cnt],
            depth: vec![0; cu_cnt],
            coef,
            reco,

            #[cfg(feature = "trace_cudata")]
            trace_idx: vec![0; cu_cnt],
        }
    }
    pub(crate) fn init(&mut self, log2_cuw: u8, log2_cuh: u8, qp_y: u8, qp_u: u8, qp_v: u8) {
        let cuw_scu = 1 << (log2_cuw - MIN_CU_LOG2 as u8);
        let cuh_scu = 1 << (log2_cuh - MIN_CU_LOG2 as u8);
        let cu_cnt = cuw_scu * cuh_scu;

        for i in 0..NUM_CU_DEPTH {
            for j in 0..BlockShape::NUM_BLOCK_SHAPE as usize {
                for v in &mut self.split_mode.data[i][j] {
                    *v = SplitMode::NO_SPLIT;
                }
            }
        }

        for i in 0..cu_cnt {
            self.qp_y[i] = 0;
            self.qp_u[i] = 0;
            self.qp_v[i] = 0;
            self.ipm[0][i] = IntraPredDir::IPD_DC_B;
            self.ipm[1][i] = IntraPredDir::IPD_DC_B;
        }

        #[cfg(feature = "trace_cudata")]
        {
            for v in &mut self.trace_idx[0..cu_cnt] {
                *v = 0;
            }
        }
    }

    pub(crate) fn copy(
        &mut self,
        src: &EvceCUData,
        x: u16,
        y: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        log2_cus: u8,
        cud: u16,
        tree_cons: &TREE_CONS,
    ) {
        let cx = x as usize >> MIN_CU_LOG2; //x = position in LCU, cx = 4x4 CU horizontal index
        let cy = y as usize >> MIN_CU_LOG2; //y = position in LCU, cy = 4x4 CU vertical index

        let cuw = (1 << log2_cuw) as usize; //current CU width
        let cuh = (1 << log2_cuh) as usize; //current CU height
        let cus = (1 << log2_cus) as usize; //current CU buffer stride (= current CU width)
        let cuw_scu = 1 << (log2_cuw as usize - MIN_CU_LOG2); //4x4 CU number in width
        let cuh_scu = 1 << (log2_cuh as usize - MIN_CU_LOG2); //4x4 CU number in height
        let cus_scu = 1 << (log2_cus as usize - MIN_CU_LOG2); //4x4 CU number in stride

        // only copy src's first row of 4x4 CUs to dis's all 4x4 CUs
        if evc_check_luma(tree_cons) {
            let size = cuw_scu;
            for j in 0..cuh_scu {
                let idx_dst = (cy + j) * cus_scu + cx;
                let idx_src = j * cuw_scu;

                for k in cud as usize..NUM_CU_DEPTH {
                    for i in 0..BlockShape::NUM_BLOCK_SHAPE as usize {
                        self.split_mode.data[k][i][idx_dst..idx_dst + size]
                            .copy_from_slice(&src.split_mode.data[k][i][idx_src..idx_src + size]);
                    }
                }

                self.qp_y[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.qp_y[idx_src..idx_src + size]);
                self.pred_mode[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.pred_mode[idx_src..idx_src + size]);
                self.ipm[0][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.ipm[0][idx_src..idx_src + size]);
                self.skip_flag[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.skip_flag[idx_src..idx_src + size]);
                self.depth[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.depth[idx_src..idx_src + size]);
                self.map_scu[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.map_scu[idx_src..idx_src + size]);
                self.map_cu_mode[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.map_cu_mode[idx_src..idx_src + size]);
                self.refi[idx_dst..idx_dst + size]
                    .clone_from_slice(&src.refi[idx_src..idx_src + size]);
                self.mvp_idx[idx_dst..idx_dst + size]
                    .clone_from_slice(&src.mvp_idx[idx_src..idx_src + size]);
                self.mv[idx_dst..idx_dst + size].clone_from_slice(&src.mv[idx_src..idx_src + size]);
                self.mvd[idx_dst..idx_dst + size]
                    .clone_from_slice(&src.mvd[idx_src..idx_src + size]);
                self.nnz[Y_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.nnz[Y_C][idx_src..idx_src + size]);

                #[cfg(feature = "trace_cudata")]
                {
                    self.trace_idx[idx_dst..idx_dst + size]
                        .copy_from_slice(&src.trace_idx[idx_src..idx_src + size]);
                }
            }

            let size = cuw;
            for j in 0..cuh {
                let idx_dst = (y as usize + j) * cus + x as usize;
                let idx_src = j * cuw;

                self.coef[Y_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.coef[Y_C][idx_src..idx_src + size]);
                self.reco[Y_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.reco[Y_C][idx_src..idx_src + size]);
            }
        }

        if evc_check_chroma(tree_cons) {
            let size = cuw >> 1;
            for j in 0..cuh >> 1 {
                let idx_dst = ((y >> 1) as usize + j) * (cus >> 1) + (x >> 1) as usize;
                let idx_src = j * (cuw >> 1);

                self.coef[U_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.coef[U_C][idx_src..idx_src + size]);
                self.reco[U_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.reco[U_C][idx_src..idx_src + size]);

                self.coef[V_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.coef[V_C][idx_src..idx_src + size]);
                self.reco[V_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.reco[V_C][idx_src..idx_src + size]);
            }

            let size = cuw_scu;
            for j in 0..cuh_scu {
                let idx_dst = (cy + j) * cus_scu + cx;
                let idx_src = j * cuw_scu;

                self.qp_u[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.qp_u[idx_src..idx_src + size]);
                self.qp_v[idx_dst..idx_dst + size]
                    .copy_from_slice(&src.qp_v[idx_src..idx_src + size]);

                self.ipm[1][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.ipm[1][idx_src..idx_src + size]);

                self.nnz[U_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.nnz[U_C][idx_src..idx_src + size]);
                self.nnz[V_C][idx_dst..idx_dst + size]
                    .copy_from_slice(&src.nnz[V_C][idx_src..idx_src + size]);
            }
        }
    }

    fn mode_cpy_rec_to_ref(
        &mut self,
        tracer: &mut Option<Tracer>,
        mut x: usize,
        mut y: usize,
        mut w: usize,
        mut h: usize,
        planes: &mut [Plane<pel>; N_C],
        tree_cons: &TREE_CONS,
    ) {
        let mut stride = w;
        if x + w > planes[Y_C].cfg.width {
            w = planes[Y_C].cfg.width - x;
        }

        if y + h > planes[Y_C].cfg.height {
            h = planes[Y_C].cfg.height - y;
        }

        if evc_check_luma(tree_cons) {
            /* luma */
            let dst = &mut planes[Y_C].as_region_mut();
            let src = &self.reco[Y_C];

            for j in 0..h {
                for i in 0..w {
                    dst[y + j][x + i] = src[j * stride + i];
                }
            }

            TRACE_CUDATA(tracer, Y_C, w, h, stride, src)
        }

        if evc_check_chroma(tree_cons) {
            /* chroma */
            x >>= 1;
            y >>= 1;
            w >>= 1;
            h >>= 1;
            stride >>= 1;

            {
                let dst = &mut planes[U_C].as_region_mut();
                let src = &self.reco[U_C];

                for j in 0..h {
                    for i in 0..w {
                        dst[y + j][x + i] = src[j * stride + i];
                    }
                }

                TRACE_CUDATA(tracer, U_C, w, h, stride, src)
            }

            {
                let dst = &mut planes[V_C].as_region_mut();
                let src = &self.reco[V_C];

                for j in 0..h {
                    for i in 0..w {
                        dst[y + j][x + i] = src[j * stride + i];
                    }
                }

                TRACE_CUDATA(tracer, V_C, w, h, stride, src)
            }
        }
    }

    pub(crate) fn copy_to_cu_data(
        &mut self,
        cu_mode: PredMode,
        cuw: u16,
        cuh: u16,
        cud: u16,
        coef_src: &CUBuffer<i16>,
        rec_src: &CUBuffer<pel>,
        tree_cons: &TREE_CONS,
        slice_num: usize,
        ipm: &[IntraPredDir],
        mi: &EvceMode,
        qp: u8,
        qp_y: u8,
        qp_u: u8,
        qp_v: u8,
        nnz: &[u16],

        #[cfg(feature = "trace_cudata")] core_trace_idx: u64,
    ) {
        let log2_cuw = CONV_LOG2(cuw as usize);
        let log2_cuh = CONV_LOG2(cuh as usize);

        if evc_check_luma(tree_cons) {
            let size = cuw as usize * cuh as usize;

            /* copy coef */
            self.coef[Y_C][0..size].copy_from_slice(&coef_src.data[Y_C][0..size]);

            /* copy reco */
            self.reco[Y_C][0..size].copy_from_slice(&rec_src.data[Y_C][0..size]);

            #[cfg(feature = "trace_cudata")]
            {
                assert_eq!(core_trace_idx, mi.trace_cu_idx);
                assert_ne!(core_trace_idx, 0);
            }

            /* copy mode info */

            let mut idx = 0;
            for j in 0..(cuh as usize) >> MIN_CU_LOG2 {
                for i in 0..(cuw as usize) >> MIN_CU_LOG2 {
                    self.pred_mode[idx + i] = cu_mode;
                    self.skip_flag[idx + i] = cu_mode == PredMode::MODE_SKIP;
                    self.nnz[Y_C][idx + i] = nnz[Y_C];

                    self.qp_y[idx + i] = qp_y;
                    self.map_scu[idx + i].RESET_QP();
                    self.map_scu[idx + i].SET_IF_COD_SN_QP(
                        if cu_mode == PredMode::MODE_INTRA {
                            1
                        } else {
                            0
                        },
                        slice_num as u32,
                        qp,
                    );

                    if self.skip_flag[idx + i] {
                        self.map_scu[idx + i].SET_SF();
                    } else {
                        self.map_scu[idx + i].CLR_SF();
                    }

                    self.depth[idx + i] = cud as i8;

                    self.map_cu_mode[idx + i].SET_LOGW(log2_cuw as u32);
                    self.map_cu_mode[idx + i].SET_LOGH(log2_cuh as u32);

                    if cu_mode == PredMode::MODE_INTRA {
                        self.ipm[0][idx + i] = ipm[0];
                        self.mv[idx + i][REFP_0][MV_X] = 0;
                        self.mv[idx + i][REFP_0][MV_Y] = 0;
                        self.mv[idx + i][REFP_1][MV_X] = 0;
                        self.mv[idx + i][REFP_1][MV_Y] = 0;
                        self.refi[idx + i][REFP_0] = -1;
                        self.refi[idx + i][REFP_1] = -1;
                    } else {
                        self.refi[idx + i][REFP_0] = mi.refi[REFP_0];
                        self.refi[idx + i][REFP_1] = mi.refi[REFP_1];
                        self.mvp_idx[idx + i][REFP_0] = mi.mvp_idx[REFP_0];
                        self.mvp_idx[idx + i][REFP_1] = mi.mvp_idx[REFP_1];

                        {
                            self.mv[idx + i][REFP_0][MV_X] = mi.mv[REFP_0][MV_X];
                            self.mv[idx + i][REFP_0][MV_Y] = mi.mv[REFP_0][MV_Y];
                            self.mv[idx + i][REFP_1][MV_X] = mi.mv[REFP_1][MV_X];
                            self.mv[idx + i][REFP_1][MV_Y] = mi.mv[REFP_1][MV_Y];
                        }

                        self.mvd[idx + i][REFP_0][MV_X] = mi.mvd[REFP_0][MV_X];
                        self.mvd[idx + i][REFP_0][MV_Y] = mi.mvd[REFP_0][MV_Y];
                        self.mvd[idx + i][REFP_1][MV_X] = mi.mvd[REFP_1][MV_X];
                        self.mvd[idx + i][REFP_1][MV_Y] = mi.mvd[REFP_1][MV_Y];
                    }
                    #[cfg(feature = "trace_cudata")]
                    {
                        self.trace_idx[idx + i] = core_trace_idx;
                    }
                }

                idx += (cuw as usize) >> MIN_CU_LOG2;
            }

            #[cfg(feature = "trace_cudata")]
            {
                let w = PEL2SCU(cuw as usize);
                let h = PEL2SCU(cuh as usize);
                let mut idx = 0;
                for j in 0..h {
                    for i in 0..w {
                        assert_eq!(self.trace_idx[idx + i], core_trace_idx);
                    }
                    idx += w;
                }
            }
        }
        if evc_check_chroma(tree_cons) {
            let size = (cuw as usize * cuh as usize) >> 2;

            /* copy coef */
            self.coef[U_C][0..size].copy_from_slice(&coef_src.data[U_C][0..size]);
            self.coef[V_C][0..size].copy_from_slice(&coef_src.data[V_C][0..size]);

            /* copy reco */
            self.reco[U_C][0..size].copy_from_slice(&rec_src.data[U_C][0..size]);
            self.reco[V_C][0..size].copy_from_slice(&rec_src.data[V_C][0..size]);

            /* copy mode info */
            let mut idx = 0;
            for j in 0..(cuh as usize) >> MIN_CU_LOG2 {
                for i in 0..(cuw as usize) >> MIN_CU_LOG2 {
                    self.nnz[U_C][idx + i] = nnz[U_C];
                    self.nnz[V_C][idx + i] = nnz[V_C];

                    self.qp_u[idx + i] = qp_u;
                    self.qp_v[idx + i] = qp_v;

                    if cu_mode == PredMode::MODE_INTRA {
                        self.ipm[1][idx + i] = ipm[1];
                    }
                }
                idx += (cuw as usize) >> MIN_CU_LOG2;
            }
        }
    }
}

/*****************************************************************************
 * mode decision structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvceMode {
    /* CU count in a CU row in a LCU (== log2_max_cuwh - MIN_CU_LOG2) */
    log2_culine: u8,
    /* reference indices */
    pub(crate) refi: [i8; REFP_NUM],
    /* MVP indices */
    pub(crate) mvp_idx: [u8; REFP_NUM],
    /* MVR indices */
    //u8    mvr_idx;
    bi_idx: u8,
    /* mv difference */
    pub(crate) mvd: [[i16; MV_D]; REFP_NUM],

    /* mv */
    pub(crate) mv: [[i16; MV_D]; REFP_NUM],

    pub(crate) inter_best_idx: usize, //pel  *pred_y_best;

    cu_mode: MCU,

    #[cfg(feature = "trace_cudata")]
    pub(crate) trace_cu_idx: u64,
}

impl EvceMode {
    fn get_cu_pred_data(
        &mut self,
        src: &EvceCUData,
        x: u16,
        y: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        log2_cus: u8,
        cud: u16,
    ) {
        let cx = x as usize >> MIN_CU_LOG2; //x = position in LCU, cx = 4x4 CU horizontal index
        let cy = y as usize >> MIN_CU_LOG2; //y = position in LCU, cy = 4x4 CU vertical index

        let cuw = (1 << log2_cuw) as usize; //current CU width
        let cuh = (1 << log2_cuh) as usize; //current CU height
        let cus = (1 << log2_cus) as usize; //current CU buffer stride (= current CU width)
        let cuw_scu = (1 << log2_cuw) as usize - MIN_CU_LOG2; //4x4 CU number in width
        let cuh_scu = (1 << log2_cuh) as usize - MIN_CU_LOG2; //4x4 CU number in height
        let cus_scu = (1 << log2_cus) as usize - MIN_CU_LOG2; //4x4 CU number in stride

        // only copy src's first row of 4x4 CUs to dis's all 4x4 CUs
        let idx_src = cy * cus_scu + cx;

        self.cu_mode = (src.pred_mode[idx_src] as u32).into();
        self.mv[REFP_0][MV_X] = src.mv[idx_src][REFP_0][MV_X];
        self.mv[REFP_0][MV_Y] = src.mv[idx_src][REFP_0][MV_Y];
        self.mv[REFP_1][MV_X] = src.mv[idx_src][REFP_1][MV_X];
        self.mv[REFP_1][MV_Y] = src.mv[idx_src][REFP_1][MV_Y];

        self.refi[REFP_0] = src.refi[idx_src][REFP_0];
        self.refi[REFP_1] = src.refi[idx_src][REFP_1];

        #[cfg(feature = "trace_cudata")]
        {
            self.trace_cu_idx = src.trace_idx[idx_src];
            assert_ne!(self.trace_cu_idx, 0);
        }
    }
}

impl EvceCtx {
    pub(crate) fn mode_init_frame(&mut self) {
        let mi = &mut self.mode;
        /* set default values to mode information */
        mi.log2_culine = self.log2_max_cuwh - MIN_CU_LOG2 as u8;

        self.pintra_init_frame();
        self.pinter_init_frame();
    }

    pub(crate) fn mode_analyze_frame(&mut self) {
        self.pintra_analyze_frame();
        self.pinter_analyze_frame();
    }

    pub(crate) fn mode_init_lcu(&mut self) {
        self.pintra_init_lcu();
        self.pinter_init_lcu();
    }

    pub(crate) fn mode_analyze_lcu(&mut self) {
        let mut split_mode_child = [false, false, false, false]; //&mut self.core.split_mode_child;
        let mut parent_split_allow = [false, false, false, false, false, true]; //&mut self.core.parent_split_allow;

        let mi = &mut self.mode;

        /* initialize cu data */
        self.core.cu_data_best[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
            .init(
                self.log2_max_cuwh,
                self.log2_max_cuwh,
                self.qp,
                self.qp,
                self.qp,
            );
        self.core.cu_data_temp[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
            .init(
                self.log2_max_cuwh,
                self.log2_max_cuwh,
                self.qp,
                self.qp,
                self.qp,
            );

        for i in 0..REFP_NUM {
            mi.mvp_idx[i] = 0;
        }
        for i in 0..REFP_NUM {
            for j in 0..MV_D {
                mi.mvd[i][j] = 0;
            }
        }

        /* decide mode */
        self.mode_coding_tree(
            self.core.x_pel,
            self.core.y_pel,
            0,
            self.log2_max_cuwh as usize,
            self.log2_max_cuwh as usize,
            0,
            true,
            0,
            self.qp,
            evc_get_default_tree_cons(),
        );

        #[cfg(feature = "trace_cudata")]
        {
            let h = 1usize << (self.log2_max_cuwh - MIN_CU_LOG2 as u8);
            let w = 1usize << (self.log2_max_cuwh - MIN_CU_LOG2 as u8);
            for j in 0..h {
                let y_pos = self.core.y_pel as usize + (j << MIN_CU_LOG2);
                for i in 0..w {
                    let x_pos = self.core.x_pel as usize + (i << MIN_CU_LOG2);
                    if x_pos < self.w as usize && y_pos < self.h as usize {
                        assert_ne!(
                            self.core.cu_data_best[self.log2_max_cuwh as usize - 2]
                                [self.log2_max_cuwh as usize - 2]
                                .trace_idx[i + h * j],
                            0
                        );
                    }
                }
            }
        }

        self.update_to_ctx_map();
        self.map_cu_data[self.core.lcu_num as usize].copy(
            &self.core.cu_data_best[self.log2_max_cuwh as usize - 2]
                [self.log2_max_cuwh as usize - 2],
            0,
            0,
            self.log2_max_cuwh,
            self.log2_max_cuwh,
            self.log2_max_cuwh,
            0,
            &evc_get_default_tree_cons(),
        );

        #[cfg(feature = "trace_cudata")]
        {
            let h = 1usize << (self.log2_max_cuwh - MIN_CU_LOG2 as u8);
            let w = 1usize << (self.log2_max_cuwh - MIN_CU_LOG2 as u8);
            for j in 0..h {
                let y_pos = self.core.y_pel as usize + (j << MIN_CU_LOG2);
                for i in 0..w {
                    let x_pos = self.core.x_pel as usize + (i << MIN_CU_LOG2);
                    if x_pos < self.w as usize && y_pos < self.h as usize {
                        assert_ne!(
                            self.core.cu_data_best[self.log2_max_cuwh as usize - 2]
                                [self.log2_max_cuwh as usize - 2]
                                .trace_idx[i + h * j],
                            0
                        );
                        assert_ne!(
                            self.map_cu_data[self.core.lcu_num as usize].trace_idx[i + h * j],
                            0
                        );
                    }
                }
            }
        }

        /* Reset all coded flag for the current lcu */
        self.core.x_scu = PEL2SCU(self.core.x_pel as usize) as u16;
        self.core.y_scu = PEL2SCU(self.core.y_pel as usize) as u16;

        let mut map_scu =
            &mut self.map_scu[(self.core.y_scu * self.w_scu + self.core.x_scu) as usize..];
        let w = std::cmp::min(
            1 << (self.log2_max_cuwh - MIN_CU_LOG2 as u8),
            self.w_scu - self.core.x_scu,
        );
        let h = std::cmp::min(
            1 << (self.log2_max_cuwh - MIN_CU_LOG2 as u8),
            self.h_scu - self.core.y_scu,
        );

        for i in 0..h {
            for j in 0..w {
                map_scu[j as usize].CLR_COD();
            }
            if i + 1 < h {
                map_scu = &mut map_scu[self.w_scu as usize..];
            }
        }
    }

    fn mode_coding_tree(
        &mut self,
        x0: u16,
        y0: u16,
        cup: u16,
        log2_cuw: usize,
        log2_cuh: usize,
        cud: u16,
        mut next_split: bool,
        qt_depth: u8,
        qp: u8,
        tree_cons: TREE_CONS,
    ) -> f64 {
        // x0 = CU's left up corner horizontal index in entrie frame
        // y0 = CU's left up corner vertical index in entire frame
        // cuw = CU width, log2_cuw = CU width in log2
        // cuh = CU height, log2_chu = CU height in log2
        // self.w = frame width, self.h = frame height
        let cuw = 1 << log2_cuw;
        let cuh = 1 << log2_cuh;
        let mut best_split_mode = SplitMode::NO_SPLIT;
        let mut bit_cnt = 0;
        let mut cost_best = MAX_COST;
        let mut cost_temp = MAX_COST;
        let mut s_temp_depth = EvceSbac::default();
        let mut c_temp_depth = EvcSbacCtx::default();
        let boundary = !(x0 + cuw <= self.w && y0 + cuh <= self.h);
        let mut split_allow = vec![false; MAX_SPLIT_NUM];
        let avail_lr = evc_check_nev_avail(
            PEL2SCU(x0 as usize) as u16,
            PEL2SCU(y0 as usize) as u16,
            cuw,
            self.w_scu,
            &self.map_scu,
        );
        let mut split_mode = SplitMode::NO_SPLIT;
        let mut do_split = false;
        let mut do_curr = false;
        let mut best_split_cost = MAX_COST;
        let best_curr_cost = MAX_COST;
        let mut split_mode_child = vec![false; 4];
        let mut curr_split_allow = vec![false; MAX_SPLIT_NUM];
        let remaining_split = 0;
        let mut num_split_tried = 0;
        let mut num_split_to_try = 0;
        let mut nev_max_depth = 0;
        let eval_parent_node_first = 1;
        let mut nbr_map_skip_flag = false;
        let cud_min = cud;
        let cud_max = cud;
        let cud_avg = cud;
        let mut dqp_temp_depth = EvceDQP::default();
        let mut best_dqp = qp;
        let mut min_qp = 0;
        let mut max_qp = 0;
        let mut cost_temp_dqp = 0.0f64;
        let mut cost_best_dqp = MAX_COST;
        let mut dqp_coded = 0;
        let mut cu_mode_dqp = PredMode::MODE_INTRA;
        let mut dist_cu_best_dqp = 0;

        self.core.tree_cons = tree_cons;
        self.core.avail_lr = avail_lr;

        self.core.s_curr_before_split[log2_cuw - 2][log2_cuh - 2] =
            self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
        self.core.c_curr_before_split[log2_cuw - 2][log2_cuh - 2] =
            self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

        //decide allowed split modes for the current node
        //based on CU size located at boundary
        if cuw > self.min_cuwh || cuh > self.min_cuwh {
            /***************************** Step 1: decide normatively allowed split modes ********************************/
            let boundary_b = boundary && (y0 + cuh > self.h) && !(x0 + cuw > self.w);
            let boundary_r = boundary && (x0 + cuw > self.w) && !(y0 + cuh > self.h);
            evc_check_split_mode(&mut split_allow);
            //save normatively allowed split modes, as it will be used in in child nodes for entropy coding of split mode
            curr_split_allow.copy_from_slice(&split_allow);
            for i in 1..MAX_SPLIT_NUM {
                num_split_to_try += if split_allow[i] { 1 } else { 0 };
            }

            /***************************** Step 2: reduce split modes by fast algorithm ********************************/
            do_split = true;
            do_curr = true;
            if !boundary {
                assert!(evc_check_luma(&self.core.tree_cons));
                nev_max_depth = self.check_nev_block(
                    x0,
                    y0,
                    log2_cuw as u8,
                    log2_cuh as u8,
                    &mut do_curr,
                    &mut do_split,
                    cud,
                    &mut nbr_map_skip_flag,
                );
                do_split = true;
                do_curr = true;
            }

            self.check_run_split(
                log2_cuw,
                log2_cuh,
                cup,
                next_split,
                do_curr,
                do_split,
                &mut split_allow,
                boundary,
                &tree_cons,
            );
        } else {
            split_allow[0] = true;
            for i in 1..MAX_SPLIT_NUM {
                split_allow[i] = false;
            }
        }

        if !boundary {
            cost_temp = 0.0;

            self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].init(
                log2_cuw as u8,
                log2_cuh as u8,
                self.qp,
                self.qp,
                self.qp,
            );

            self.sh.qp_prev_mode = self.core.dqp_data[log2_cuw - 2][log2_cuh - 2].prev_QP as u8;
            best_dqp = self.sh.qp_prev_mode;

            split_mode = SplitMode::NO_SPLIT;
            if split_allow[split_mode as usize] {
                if (cuw > self.min_cuwh || cuh > self.min_cuwh)
                    && evc_check_luma(&self.core.tree_cons)
                {
                    /* consider CU split mode */
                    self.core.s_temp_run = self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
                    self.core.c_temp_run = self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

                    self.core.s_temp_run.bit_reset();
                    evc_set_split_mode(
                        &mut self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].split_mode,
                        SplitMode::NO_SPLIT,
                        cud,
                        0,
                        cuw,
                        cuh,
                        cuw,
                    );
                    let split_mode_buf = if self.core.s_temp_run.is_bitcount {
                        &self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].split_mode
                    } else {
                        &self.map_cu_data[self.core.lcu_num as usize].split_mode
                    };
                    evce_eco_split_mode(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        x0,
                        y0,
                        cud,
                        0,
                        cuw,
                        cuh,
                        cuw,
                        split_mode_buf,
                    );

                    bit_cnt = self.core.s_temp_run.get_bit_number();
                    cost_temp += self.lambda[0] * bit_cnt as f64;

                    self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2] = self.core.s_temp_run;
                    self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2] = self.core.c_temp_run;
                }

                self.core.cup = cup as u32;
                let mut is_dqp_set = false;
                self.get_min_max_qp(
                    &mut min_qp,
                    &mut max_qp,
                    &mut is_dqp_set,
                    split_mode,
                    cuw,
                    cuh,
                    qp,
                    x0,
                    y0,
                );
                for dqp in min_qp..=max_qp {
                    self.core.qp = GET_QP(qp as i8, dqp as i8 - qp as i8) as u8;
                    self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].curr_QP = self.core.qp;
                    if self.core.cu_qp_delta_code_mode != 2 || is_dqp_set {
                        self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].cu_qp_delta_code =
                            1 + if is_dqp_set { 1 } else { 0 };
                        self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].cu_qp_delta_is_coded =
                            false;
                    }
                    cost_temp_dqp = cost_temp;
                    self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].init(
                        log2_cuw as u8,
                        log2_cuh as u8,
                        self.qp,
                        self.qp,
                        self.qp,
                    );

                    self.clear_map_scu(x0, y0, cuw, cuh);
                    cost_temp_dqp += self.mode_coding_unit(x0, y0, log2_cuw, log2_cuh, cud);

                    if cost_best > cost_temp_dqp {
                        cu_mode_dqp = self.core.cu_mode;
                        dist_cu_best_dqp = self.core.dist_cu_best;
                        /* backup the current best data */
                        self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2].copy(
                            &self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2],
                            0,
                            0,
                            log2_cuw as u8,
                            log2_cuh as u8,
                            log2_cuw as u8,
                            cud,
                            &self.core.tree_cons,
                        );
                        cost_best = cost_temp_dqp;
                        best_split_mode = SplitMode::NO_SPLIT;

                        s_temp_depth = self.core.s_next_best[log2_cuw - 2][log2_cuh - 2];
                        c_temp_depth = self.core.c_next_best[log2_cuw - 2][log2_cuh - 2];

                        dqp_temp_depth = self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2];

                        if let Some(pic) = &self.pic[PIC_IDX_MODE] {
                            let cu_data_best =
                                &mut self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2];
                            cu_data_best.mode_cpy_rec_to_ref(
                                &mut self.core.bs_temp.tracer,
                                x0 as usize,
                                y0 as usize,
                                cuw as usize,
                                cuh as usize,
                                &mut pic.borrow().frame.borrow_mut().planes,
                                &self.core.tree_cons,
                            );
                        }

                        if evc_check_luma(&self.core.tree_cons) {
                            // update history MV list
                            // in mode_coding_unit, self.fn_pinter_analyze_cu will store the best MV in mi
                            // if the cost_temp has been update above, the best MV is in mi
                            self.mode.get_cu_pred_data(
                                &self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2],
                                0,
                                0,
                                log2_cuw as u8,
                                log2_cuh as u8,
                                log2_cuw as u8,
                                cud,
                            );
                        }
                    }
                }
                if is_dqp_set && self.core.cu_qp_delta_code_mode == 2 {
                    self.core.cu_qp_delta_code_mode = 0;
                }
                cost_temp = cost_best;
                self.core.cu_mode = cu_mode_dqp;
                self.core.dist_cu_best = dist_cu_best_dqp;

                EVC_TRACE_COUNTER(&mut self.core.bs_temp.tracer);
                EVC_TRACE(&mut self.core.bs_temp.tracer, "Block [");
                EVC_TRACE(&mut self.core.bs_temp.tracer, x0);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " , ");
                EVC_TRACE(&mut self.core.bs_temp.tracer, y0);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " ]x(");
                EVC_TRACE(&mut self.core.bs_temp.tracer, cuw);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " x");
                EVC_TRACE(&mut self.core.bs_temp.tracer, cuh);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " ) split_type ");
                EVC_TRACE(&mut self.core.bs_temp.tracer, SplitMode::NO_SPLIT as u32);
                EVC_TRACE(&mut self.core.bs_temp.tracer, "  cost is ");
                EVC_TRACE(&mut self.core.bs_temp.tracer, cost_temp as i64);
                EVC_TRACE(&mut self.core.bs_temp.tracer, " \n");
            } else {
                cost_temp = MAX_COST;
            }
        }

        if cost_best != MAX_COST
            && cud
                >= if self.poc.poc_val % 2 != 0 {
                    ENC_ECU_DEPTH_B - 2
                } else {
                    ENC_ECU_DEPTH_B
                }
            && self.core.cu_mode == PredMode::MODE_SKIP
        {
            next_split = false;
        }

        if cost_best != MAX_COST && self.sh.slice_type == SliceType::EVC_ST_I {
            let dist_cu = self.core.dist_cu_best;
            let dist_cu_th = 1 << (log2_cuw + log2_cuh + 7);

            if dist_cu < dist_cu_th {
                let mut bits_inc_by_split = 0;
                bits_inc_by_split += if log2_cuw + log2_cuh >= 6 { 2 } else { 0 }; //two split flags
                bits_inc_by_split += 8; //one more (intra dir + cbf + edi_flag + mtr info) + 1-bit penalty, approximately 8 bits

                if (dist_cu as f64) < self.lambda[0] * bits_inc_by_split as f64 {
                    next_split = false;
                }
            }
        }

        if (cuw > MIN_CU_SIZE as u16 || cuh > MIN_CU_SIZE as u16) && next_split {
            split_mode = SplitMode::SPLIT_QUAD;
            if split_allow[split_mode as usize] {
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

                let mut prev_log2_sub_cuw = split_struct.log_cuw[0] as usize;
                let mut prev_log2_sub_cuh = split_struct.log_cuh[0] as usize;

                self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].init(
                    log2_cuw as u8,
                    log2_cuh as u8,
                    self.qp,
                    self.qp,
                    self.qp,
                );
                self.clear_map_scu(x0, y0, cuw, cuh);

                let mut cost_temp = 0.0;

                if x0 + cuw <= self.w && y0 + cuh <= self.h {
                    /* consider CU split flag */
                    self.core.s_temp_run =
                        self.core.s_curr_before_split[log2_cuw - 2][log2_cuh - 2];
                    self.core.c_temp_run =
                        self.core.c_curr_before_split[log2_cuw - 2][log2_cuh - 2];

                    self.core.s_temp_run.bit_reset();
                    evc_set_split_mode(
                        &mut self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].split_mode,
                        split_mode,
                        cud,
                        0,
                        cuw,
                        cuh,
                        cuw,
                    );

                    let split_mode_buf = if self.core.s_temp_run.is_bitcount {
                        &self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].split_mode
                    } else {
                        &self.map_cu_data[self.core.lcu_num as usize].split_mode
                    };
                    evce_eco_split_mode(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        x0,
                        y0,
                        cud,
                        0,
                        cuw,
                        cuh,
                        cuw,
                        split_mode_buf,
                    );

                    bit_cnt = self.core.s_temp_run.get_bit_number();
                    cost_temp += (self.lambda[0] * bit_cnt as f64);

                    self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2] = self.core.s_temp_run;
                    self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2] = self.core.c_temp_run;
                }

                let mut min_qp = 0i8;
                let mut max_qp = 0i8;
                let mut is_dqp_set = false;
                self.get_min_max_qp(
                    &mut min_qp,
                    &mut max_qp,
                    &mut is_dqp_set,
                    split_mode,
                    cuw,
                    cuh,
                    qp,
                    x0,
                    y0,
                );

                let mut loop_counter = 0;
                if is_dqp_set {
                    loop_counter = (max_qp - min_qp).abs();
                }
                cost_best_dqp = MAX_COST;
                for dqp_loop in 0..=loop_counter {
                    let dqp = min_qp + dqp_loop;
                    self.core.qp = GET_QP(qp as i8, dqp - qp as i8) as u8;
                    if is_dqp_set {
                        self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].cu_qp_delta_code = 2;
                        self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].cu_qp_delta_is_coded =
                            false;
                        self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].curr_QP = self.core.qp;
                    }

                    cost_temp_dqp = cost_temp;
                    self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].init(
                        log2_cuw as u8,
                        log2_cuh as u8,
                        self.qp,
                        self.qp,
                        self.qp,
                    );
                    self.clear_map_scu(x0, y0, cuw, cuh);

                    //#if TRACE_ENC_CU_DATA_CHECK
                    //                  static int counter_in[MAX_CU_LOG2 - MIN_CU_LOG2][MAX_CU_LOG2 - MIN_CU_LOG2] = { 0, };
                    //                  counter_in[log2_cuw - MIN_CU_LOG2][log2_cuh - MIN_CU_LOG2]++;
                    // #endif

                    for part_num in 0..split_struct.part_count {
                        let cur_part_num = part_num;
                        let log2_sub_cuw = split_struct.log_cuw[cur_part_num] as usize;
                        let log2_sub_cuh = split_struct.log_cuh[cur_part_num] as usize;
                        let x_pos = split_struct.x_pos[cur_part_num];
                        let y_pos = split_struct.y_pos[cur_part_num];
                        let cur_cuw = split_struct.width[cur_part_num];
                        let cur_cuh = split_struct.height[cur_part_num];

                        if (x_pos < self.w) && (y_pos < self.h) {
                            if part_num == 0 {
                                self.core.s_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] =
                                    self.core.s_curr_best[log2_cuw - 2][log2_cuh - 2];
                                self.core.c_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] =
                                    self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];

                                self.core.dqp_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] =
                                    self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2];
                            } else {
                                self.core.s_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] = self
                                    .core
                                    .s_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];
                                self.core.c_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] = self
                                    .core
                                    .c_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];

                                self.core.dqp_curr_best[log2_sub_cuw - 2][log2_sub_cuh - 2] = self
                                    .core
                                    .dqp_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];
                            }
                            cost_temp_dqp += self.mode_coding_tree(
                                x_pos,
                                y_pos,
                                split_struct.cup[cur_part_num],
                                log2_sub_cuw as usize,
                                log2_sub_cuh as usize,
                                split_struct.cud[cur_part_num],
                                true,
                                split_mode.inc_qt_depth(qt_depth),
                                self.core.qp,
                                split_struct.tree_cons,
                            );

                            self.core.qp = GET_QP(qp as i8, dqp - qp as i8) as u8;

                            self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2].copy(
                                &self.core.cu_data_best[log2_sub_cuw - 2][log2_sub_cuh - 2],
                                x_pos - split_struct.x_pos[0],
                                y_pos - split_struct.y_pos[0],
                                log2_sub_cuw as u8,
                                log2_sub_cuh as u8,
                                log2_cuw as u8,
                                cud,
                                &split_struct.tree_cons,
                            );

                            self.update_map_scu(x_pos, y_pos, cur_cuw, cur_cuh);
                            prev_log2_sub_cuw = log2_sub_cuw;
                            prev_log2_sub_cuh = log2_sub_cuh;
                        }
                        self.core.tree_cons = tree_cons;
                    }

                    EVC_TRACE_COUNTER(&mut self.core.bs_temp.tracer);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, "Block [");
                    EVC_TRACE(&mut self.core.bs_temp.tracer, x0);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, " , ");
                    EVC_TRACE(&mut self.core.bs_temp.tracer, y0);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, " ]x(");
                    EVC_TRACE(&mut self.core.bs_temp.tracer, cuw);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, " x");
                    EVC_TRACE(&mut self.core.bs_temp.tracer, cuh);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, " ) split_type ");
                    EVC_TRACE(
                        &mut self.core.bs_temp.tracer,
                        if split_mode == SplitMode::NO_SPLIT {
                            0
                        } else {
                            5
                        },
                    );
                    EVC_TRACE(&mut self.core.bs_temp.tracer, "  cost is ");
                    EVC_TRACE(&mut self.core.bs_temp.tracer, cost_temp as i64);
                    EVC_TRACE(&mut self.core.bs_temp.tracer, " \n");

                    if cost_best_dqp > cost_temp_dqp {
                        cost_best_dqp = cost_temp_dqp;
                    }

                    if cost_best - 0.0001 > cost_temp_dqp {
                        /* backup the current best data */
                        self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2].copy(
                            &self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2],
                            0,
                            0,
                            log2_cuw as u8,
                            log2_cuh as u8,
                            log2_cuw as u8,
                            cud,
                            &self.core.tree_cons,
                        );
                        cost_best = cost_temp_dqp;
                        best_dqp = self.core.dqp_data[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2]
                            .prev_QP;
                        dqp_temp_depth =
                            self.core.dqp_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];

                        s_temp_depth =
                            self.core.s_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];
                        c_temp_depth =
                            self.core.c_next_best[prev_log2_sub_cuw - 2][prev_log2_sub_cuh - 2];

                        best_split_mode = split_mode;
                    }

                    cost_temp = cost_best_dqp;

                    if is_dqp_set {
                        self.core.cu_qp_delta_code_mode = 0;
                    }

                    if split_mode != SplitMode::NO_SPLIT && cost_temp < best_split_cost {
                        best_split_cost = cost_temp;
                    }
                }
            }
        }

        if let Some(pic) = &self.pic[PIC_IDX_MODE] {
            let cu_data_best = &mut self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2];
            cu_data_best.mode_cpy_rec_to_ref(
                &mut self.core.bs_temp.tracer,
                x0 as usize,
                y0 as usize,
                cuw as usize,
                cuh as usize,
                &mut pic.borrow().frame.borrow_mut().planes,
                &self.core.tree_cons,
            );
        }

        /* restore best data */
        evc_set_split_mode(
            &mut self.core.cu_data_best[log2_cuw - 2][log2_cuh - 2].split_mode,
            best_split_mode,
            cud,
            0,
            cuw,
            cuh,
            cuw,
        );

        self.core.s_next_best[log2_cuw - 2][log2_cuh - 2] = s_temp_depth;
        self.core.c_next_best[log2_cuw - 2][log2_cuh - 2] = c_temp_depth;

        self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2] = dqp_temp_depth;

        assert_ne!(cost_best, MAX_COST);

        if cost_best > MAX_COST {
            MAX_COST
        } else {
            cost_best
        }
    }

    fn check_nev_block(
        &mut self,
        x0: u16,
        y0: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        do_curr: &mut bool,
        do_split: &mut bool,
        cud: u16,
        nbr_map_skip_flag: &mut bool,
    ) -> i32 {
        let mut nbr_map_skipcnt = 0;
        let mut nbr_map_cnt = 0;

        let x_scu = (x0 >> MIN_CU_LOG2);
        let y_scu = (y0 >> MIN_CU_LOG2);

        let cup = y_scu as u32 * self.w_scu as u32 + x_scu as u32;

        let log2_scuw = log2_cuw - MIN_CU_LOG2 as u8;
        let log2_scuh = log2_cuh - MIN_CU_LOG2 as u8;
        let scuw = 1 << log2_scuw;
        let scuh = 1 << log2_scuh;

        let mut size_cnt = vec![0; MAX_CU_DEPTH];

        *do_curr = true;
        *do_split = true;
        let avail_cu = evc_get_avail_block(
            x_scu,
            y_scu,
            self.w_scu,
            self.h_scu,
            cup,
            log2_cuw,
            log2_cuh,
            &self.map_scu,
        );

        let mut min_depth = MAX_CU_DEPTH as i8;
        let mut max_depth = 0;

        if IS_AVAIL(avail_cu, AVAIL_UP) {
            for w in 0..scuw {
                let pos = (cup - self.w_scu as u32 + w) as usize;

                let tmp = self.map_depth[pos];
                min_depth = if tmp < min_depth { tmp } else { min_depth };
                max_depth = if tmp > max_depth { tmp } else { max_depth };

                nbr_map_skipcnt += if self.map_scu[pos].GET_SF() != 0 {
                    1
                } else {
                    0
                };
                nbr_map_cnt += 1;
            }
        }

        if IS_AVAIL(avail_cu, AVAIL_UP_RI) {
            let pos = (cup - self.w_scu as u32 + scuw) as usize;

            let tmp = self.map_depth[pos];
            min_depth = if tmp < min_depth { tmp } else { min_depth };
            max_depth = if tmp > max_depth { tmp } else { max_depth };
        }

        if IS_AVAIL(avail_cu, AVAIL_LE) {
            for h in 0..scuh {
                let pos = (cup - 1 + (h * self.w_scu) as u32) as usize;

                let tmp = self.map_depth[pos];
                min_depth = if tmp < min_depth { tmp } else { min_depth };
                max_depth = if tmp > max_depth { tmp } else { max_depth };

                nbr_map_skipcnt += if self.map_scu[pos].GET_SF() != 0 {
                    1
                } else {
                    0
                };
                nbr_map_cnt += 1;
            }
        }

        if IS_AVAIL(avail_cu, AVAIL_LO_LE) {
            let pos = (cup + (self.w_scu * scuh) as u32 - 1) as usize;

            let tmp = self.map_depth[pos];
            min_depth = if tmp < min_depth { tmp } else { min_depth };
            max_depth = if tmp > max_depth { tmp } else { max_depth };
        }

        if IS_AVAIL(avail_cu, AVAIL_UP_LE) {
            let pos = (cup - self.w_scu as u32 - 1) as usize;

            let tmp = self.map_depth[pos];
            min_depth = if tmp < min_depth { tmp } else { min_depth };
            max_depth = if tmp > max_depth { tmp } else { max_depth };
        }

        if IS_AVAIL(avail_cu, AVAIL_RI) {
            for h in 0..scuh {
                let pos = (cup + scuw + (h * self.w_scu) as u32) as usize;

                let tmp = self.map_depth[pos];
                min_depth = if tmp < min_depth { tmp } else { min_depth };
                max_depth = if tmp > max_depth { tmp } else { max_depth };

                nbr_map_skipcnt += if self.map_scu[pos].GET_SF() != 0 {
                    1
                } else {
                    0
                };
                nbr_map_cnt += 1;
            }
        }

        if IS_AVAIL(avail_cu, AVAIL_LO_RI) {
            let pos = (cup + (self.w_scu * scuh) as u32 + scuw) as usize;

            let tmp = self.map_depth[pos];
            min_depth = if tmp < min_depth { tmp } else { min_depth };
            max_depth = if tmp > max_depth { tmp } else { max_depth };
        }

        if avail_cu != 0 {
            if cud < (min_depth - 1) as u16 {
                if log2_cuw > MIN_CU_LOG2 as u8 && log2_cuh > MIN_CU_LOG2 as u8 {
                    *do_curr = false;
                } else {
                    *do_curr = true;
                }
            }

            if cud > (max_depth + 1) as u16 {
                *do_split = if *do_curr { false } else { true };
            }
        } else {
            max_depth = MAX_CU_DEPTH as i8;
            min_depth = 0;
        }

        *nbr_map_skip_flag = false;
        if self.slice_type != SliceType::EVC_ST_I && nbr_map_skipcnt > (nbr_map_cnt / 2) {
            *nbr_map_skip_flag = true;
        }

        return max_depth as i32;
    }

    fn check_run_split(
        &mut self,
        log2_cuw: usize,
        log2_cuh: usize,
        cup: u16,
        next_split: bool,
        do_curr: bool,
        do_split: bool,
        split_allow: &mut [bool],
        boundary: bool,
        tree_cons: &TREE_CONS,
    ) {
        let min_cost = MAX_COST;
        let mut run_list = vec![false; MAX_SPLIT_NUM]; //a smaller set of allowed split modes based on a save & load technique

        if !next_split {
            split_allow[0] = true;

            for i in 1..MAX_SPLIT_NUM {
                split_allow[i] = false;
            }

            return;
        }

        for i in 0..MAX_SPLIT_NUM {
            run_list[i] = true;
        }

        run_list[0] = run_list[0] && do_curr;
        for i in 1..MAX_SPLIT_NUM {
            run_list[i] = run_list[i] && do_split;
        }

        //modified split_allow by the save & load decision
        let mut num_run = 0;
        split_allow[0] = run_list[0];
        for i in 1..MAX_SPLIT_NUM {
            split_allow[i] = run_list[i] && split_allow[i];
            num_run += if split_allow[i] { 1 } else { 0 };
        }

        //if all further splitting modes are not tried, at least we need try NO_SPLIT
        if num_run == 0 {
            split_allow[0] = true;
        }
    }

    fn get_min_max_qp(
        &mut self,
        min_qp: &mut i8,
        max_qp: &mut i8,
        is_dqp_set: &mut bool,
        split_mode: SplitMode,
        cuw: u16,
        cuh: u16,
        qp: u8,
        x0: u16,
        y0: u16,
    ) {
        *is_dqp_set = false;
        if !self.pps.cu_qp_delta_enabled_flag {
            *min_qp = self.sh.qp as i8; // Clip?
            *max_qp = self.sh.qp as i8;
        } else {
            if !self.sps.dquant_flag {
                if split_mode != SplitMode::NO_SPLIT {
                    *min_qp = qp as i8; // Clip?
                    *max_qp = qp as i8;
                } else {
                    *min_qp = self.sh.qp as i8;
                    *max_qp = self.sh.qp as i8 + self.sh.dqp;
                }
            } else {
                *min_qp = qp as i8; // Clip?
                *max_qp = qp as i8;
                if split_mode == SplitMode::NO_SPLIT
                    && CONV_LOG2(cuw as usize) + CONV_LOG2(cuh as usize)
                        >= self.pps.cu_qp_delta_area
                    && self.core.cu_qp_delta_code_mode != 2
                {
                    self.core.cu_qp_delta_code_mode = 1;
                    *min_qp = self.sh.qp as i8;
                    *max_qp = self.sh.qp as i8 + self.sh.dqp;

                    if CONV_LOG2(cuw as usize) == 7 || CONV_LOG2(cuh as usize) == 7 {
                        *is_dqp_set = true;
                        self.core.cu_qp_delta_code_mode = 2;
                    } else {
                        *is_dqp_set = false;
                    }
                } else if (CONV_LOG2(cuw as usize) + CONV_LOG2(cuh as usize)
                    == self.pps.cu_qp_delta_area + 1)
                    || (CONV_LOG2(cuh as usize) + CONV_LOG2(cuw as usize)
                        == self.pps.cu_qp_delta_area
                        && self.core.cu_qp_delta_code_mode != 2)
                {
                    self.core.cu_qp_delta_code_mode = 2;
                    *is_dqp_set = true;
                    *min_qp = self.sh.qp as i8;
                    *max_qp = self.sh.qp as i8 + self.sh.dqp;
                }
            }
        }
    }

    fn update_to_ctx_map(&mut self) {
        let cu_data = &self.core.cu_data_best[self.log2_max_cuwh as usize - 2]
            [self.log2_max_cuwh as usize - 2];
        let mut cuw = self.max_cuwh;
        let mut cuh = self.max_cuwh;
        let x = self.core.x_pel;
        let y = self.core.y_pel;

        if x + cuw > self.w {
            cuw = self.w - x;
        }

        if y + cuh > self.h {
            cuh = self.h - y;
        }

        let w = (cuw as usize) >> MIN_CU_LOG2;
        let h = (cuh as usize) >> MIN_CU_LOG2;

        /* copy mode info */
        let mut core_idx = 0usize;
        let mut ctx_idx = ((y >> MIN_CU_LOG2) * self.w_scu + (x >> MIN_CU_LOG2)) as usize;

        if let (Some(map_refi), Some(map_mv)) = (&mut self.map_refi, &mut self.map_mv) {
            let (mut map_refi, mut map_mv) = (map_refi.borrow_mut(), map_mv.borrow_mut());
            let mut map_ipm = &mut self.map_ipm;

            for i in 0..h {
                for j in 0..w {
                    if cu_data.pred_mode[core_idx + j] == PredMode::MODE_INTRA {
                        map_ipm[ctx_idx + j] = cu_data.ipm[0][core_idx + j];
                        map_mv[ctx_idx + j][REFP_0][MV_X] = 0;
                        map_mv[ctx_idx + j][REFP_0][MV_Y] = 0;
                        map_mv[ctx_idx + j][REFP_1][MV_X] = 0;
                        map_mv[ctx_idx + j][REFP_1][MV_Y] = 0;
                    } else {
                        map_refi[ctx_idx + j][REFP_0] = cu_data.refi[core_idx + j][REFP_0];
                        map_refi[ctx_idx + j][REFP_1] = cu_data.refi[core_idx + j][REFP_1];
                        map_mv[ctx_idx + j][REFP_0][MV_X] = cu_data.mv[core_idx + j][REFP_0][MV_X];
                        map_mv[ctx_idx + j][REFP_0][MV_Y] = cu_data.mv[core_idx + j][REFP_0][MV_Y];
                        map_mv[ctx_idx + j][REFP_1][MV_X] = cu_data.mv[core_idx + j][REFP_1][MV_X];
                        map_mv[ctx_idx + j][REFP_1][MV_Y] = cu_data.mv[core_idx + j][REFP_1][MV_Y];
                    }
                }
                ctx_idx += self.w_scu as usize;
                core_idx += (self.max_cuwh >> MIN_CU_LOG2) as usize;
            }
        }

        self.update_map_scu(
            self.core.x_pel,
            self.core.y_pel,
            self.max_cuwh,
            self.max_cuwh,
        );
    }

    fn update_map_scu(&mut self, x: u16, y: u16, src_cuw: u16, src_cuh: u16) {
        let scu_x = x as usize >> MIN_CU_LOG2;
        let scu_y = y as usize >> MIN_CU_LOG2;
        let log2_src_cuw = CONV_LOG2(src_cuw as usize) as usize;
        let log2_src_cuh = CONV_LOG2(src_cuh as usize) as usize;
        let pos = scu_y * self.w_scu as usize + scu_x;

        let mut map_scu = &mut self.map_scu[pos..];
        let mut src_map_scu =
            &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].map_scu[..];

        let mut map_ipm = &mut self.map_ipm[pos..];
        let mut src_map_ipm =
            &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].ipm[0][..];

        let mut map_depth = &mut self.map_depth[pos..];
        let mut src_depth = &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].depth[..];

        let mut map_cu_mode = &mut self.map_cu_mode[pos..];
        let mut src_map_cu_mode =
            &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].map_cu_mode[..];

        let w = if x + src_cuw > self.w {
            (self.w - x) >> MIN_CU_LOG2
        } else {
            (src_cuw >> MIN_CU_LOG2)
        } as usize;

        let h = if y + src_cuh > self.h {
            (self.h - y) >> MIN_CU_LOG2
        } else {
            (src_cuh >> MIN_CU_LOG2)
        } as usize;

        if let (Some(map_refi), Some(map_mv)) = (&mut self.map_refi, &mut self.map_mv) {
            let (mut map_refi, mut map_mv) = (map_refi.borrow_mut(), map_mv.borrow_mut());

            let mut map_refi = &mut map_refi[pos..];
            let mut src_map_refi =
                &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].refi[..];

            let mut map_mv = &mut map_mv[pos..];
            let mut src_map_mv = &self.core.cu_data_best[log2_src_cuw - 2][log2_src_cuh - 2].mv[..];

            for i in 0..h {
                map_scu[..w].copy_from_slice(&src_map_scu[..w]);
                map_ipm[..w].copy_from_slice(&src_map_ipm[..w]);
                map_depth[..w].copy_from_slice(&src_depth[..w]);
                map_cu_mode[..w].copy_from_slice(&src_map_cu_mode[..w]);
                map_refi[..w].copy_from_slice(&src_map_refi[..w]);
                map_mv[..w].copy_from_slice(&src_map_mv[..w]);

                if i + 1 < h {
                    map_depth = &mut map_depth[self.w_scu as usize..];
                    src_depth = &src_depth[(src_cuw >> MIN_CU_LOG2) as usize..];

                    map_scu = &mut map_scu[self.w_scu as usize..];
                    src_map_scu = &src_map_scu[(src_cuw >> MIN_CU_LOG2) as usize..];

                    map_ipm = &mut map_ipm[self.w_scu as usize..];
                    src_map_ipm = &src_map_ipm[(src_cuw >> MIN_CU_LOG2) as usize..];

                    map_mv = &mut map_mv[self.w_scu as usize..];
                    src_map_mv = &src_map_mv[(src_cuw >> MIN_CU_LOG2) as usize..];

                    map_refi = &mut map_refi[self.w_scu as usize..];
                    src_map_refi = &src_map_refi[(src_cuw >> MIN_CU_LOG2) as usize..];

                    map_cu_mode = &mut map_cu_mode[self.w_scu as usize..];
                    src_map_cu_mode = &src_map_cu_mode[(src_cuw >> MIN_CU_LOG2) as usize..];
                }
            }
        }
    }

    fn clear_map_scu(&mut self, x: u16, y: u16, mut cuw: u16, mut cuh: u16) {
        let map_cu_mode = &mut self.map_cu_mode
            [((y >> MIN_CU_LOG2) * self.w_scu + (x >> MIN_CU_LOG2)) as usize..];
        let map_scu =
            &mut self.map_scu[((y >> MIN_CU_LOG2) * self.w_scu + (x >> MIN_CU_LOG2)) as usize..];

        if x + cuw > self.w {
            cuw = self.w - x;
        }

        if y + cuh > self.h {
            cuh = self.h - y;
        }

        let w = (cuw >> MIN_CU_LOG2) as usize;
        let h = (cuh >> MIN_CU_LOG2) as usize;

        for j in 0..h {
            for i in 0..w {
                map_scu[j * self.w_scu as usize + i] = MCU::default();
                map_cu_mode[j * self.w_scu as usize + i] = MCU::default();
            }
        }
    }

    fn mode_coding_unit(
        &mut self,
        x: u16,
        y: u16,
        log2_cuw: usize,
        log2_cuh: usize,
        cud: u16,
    ) -> f64 {
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

        assert!((log2_cuw as i8 - log2_cuh as i8).abs() <= 2);
        self.mode_cu_init(x, y, log2_cuw as u8, log2_cuh as u8, cud);

        self.core.avail_lr = evc_check_nev_avail(
            self.core.x_scu,
            self.core.y_scu,
            (1 << log2_cuw),
            self.w_scu,
            &self.map_scu,
        );

        let mut cost = MAX_COST;
        let mut cost_best = MAX_COST;
        self.core.cost_best = MAX_COST;

        /* inter *************************************************************/
        if self.slice_type != SliceType::EVC_ST_I {
            self.core.avail_cu = evc_get_avail_inter(
                self.core.x_scu as usize,
                self.core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                self.core.scup as usize,
                self.core.cuw as usize,
                self.core.cuh as usize,
                &self.map_scu,
            );
            cost = self.pinter_analyze_cu(
                x as usize,
                y as usize,
                log2_cuw as usize,
                log2_cuh as usize,
            );

            if cost < cost_best {
                cost_best = cost;

                #[cfg(feature = "trace_cudata")]
                {
                    self.mode.trace_cu_idx = self.core.trace_idx;
                    assert_ne!(self.core.trace_idx, 0);
                }

                if self.pps.cu_qp_delta_enabled_flag {
                    self.evce_set_qp(self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2].prev_QP);
                }

                let cu_data = &mut self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2];
                cu_data.copy_to_cu_data(
                    self.core.cu_mode,
                    self.core.cuw,
                    self.core.cuh,
                    self.core.cud,
                    &self.core.ctmp,
                    &self.pinter.rec[self.mode.inter_best_idx],
                    &self.core.tree_cons,
                    self.slice_num,
                    &self.core.ipm,
                    &self.mode,
                    self.core.qp,
                    self.core.qp_y,
                    self.core.qp_u,
                    self.core.qp_v,
                    &self.core.nnz,
                    #[cfg(feature = "trace_cudata")]
                    self.core.trace_idx,
                );
            }
        }

        /* intra *************************************************************/
        if (self.slice_type == SliceType::EVC_ST_I
            || self.core.nnz[Y_C] != 0
            || self.core.nnz[U_C] != 0
            || self.core.nnz[V_C] != 0
            || cost_best == MAX_COST)
            && !evc_check_only_inter(&self.core.tree_cons)
        {
            self.core.cost_best = cost_best;
            self.core.dist_cu_best = i32::MAX;

            if self.core.cost_best != MAX_COST {
                if let Some(pic) = &self.pintra.pic_o {
                    let frame = &pic.borrow().frame;
                    let planes = &frame.borrow().planes;
                    self.core.inter_satd = evce_satd_16b(
                        x as usize,
                        y as usize,
                        1 << log2_cuw as usize,
                        1 << log2_cuh as usize,
                        &planes[Y_C].as_region(),
                        &self.pinter.pred[self.mode.inter_best_idx][0].data[Y_C],
                    );
                }
            } else {
                self.core.inter_satd = u32::MAX;
            }
            if self.pps.cu_qp_delta_enabled_flag {
                self.evce_set_qp(self.core.dqp_curr_best[log2_cuw - 2][log2_cuh - 2].curr_QP as u8);
            }

            self.core.avail_cu = evc_get_avail_intra(
                self.core.x_scu as usize,
                self.core.y_scu as usize,
                self.w_scu as usize,
                self.h_scu as usize,
                self.core.scup as usize,
                log2_cuw as u8,
                log2_cuh as u8,
                &self.map_scu,
            );
            cost = self.pintra_analyze_cu(
                x as usize,
                y as usize,
                log2_cuw as usize,
                log2_cuh as usize,
            );

            if cost < cost_best {
                cost_best = cost;

                #[cfg(feature = "trace_cudata")]
                {
                    self.mode.trace_cu_idx = self.core.trace_idx;
                    assert_ne!(self.core.trace_idx, 0);
                }

                self.core.cu_mode = PredMode::MODE_INTRA;

                self.core.s_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.s_temp_best;
                self.core.c_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.c_temp_best;

                self.core.dqp_next_best[log2_cuw - 2][log2_cuh - 2] = self.core.dqp_temp_best;
                self.core.dist_cu_best = self.core.dist_cu;

                let cu_data = &mut self.core.cu_data_temp[log2_cuw - 2][log2_cuh - 2];
                cu_data.copy_to_cu_data(
                    self.core.cu_mode,
                    self.core.cuw,
                    self.core.cuh,
                    self.core.cud,
                    &self.core.ctmp,
                    &self.pintra.rec,
                    &self.core.tree_cons,
                    self.slice_num,
                    &self.core.ipm,
                    &self.mode,
                    self.core.qp,
                    self.core.qp_y,
                    self.core.qp_u,
                    self.core.qp_v,
                    &self.core.nnz,
                    #[cfg(feature = "trace_cudata")]
                    self.core.trace_idx,
                );
            }
        }

        cost_best
    }

    fn mode_cu_init(&mut self, x: u16, y: u16, log2_cuw: u8, log2_cuh: u8, cud: u16) {
        #[cfg(feature = "trace_cudata")]
        {
            self.core.trace_idx += 1;
        }

        self.core.cuw = 1 << log2_cuw;
        self.core.cuh = 1 << log2_cuh;
        self.core.log2_cuw = log2_cuw;
        self.core.log2_cuh = log2_cuh;
        self.core.x_scu = PEL2SCU(x as usize) as u16;
        self.core.y_scu = PEL2SCU(y as usize) as u16;
        self.core.scup = (self.core.y_scu as u32 * self.w_scu as u32) + self.core.x_scu as u32;
        self.core.avail_cu = 0;
        self.core.avail_lr = LR_10;

        self.core.nnz[Y_C] = 0;
        self.core.nnz[U_C] = 0;
        self.core.nnz[V_C] = 0;
        self.core.cud = cud;
        self.core.cu_mode = PredMode::MODE_INTRA;

        /* Getting the appropriate QP based on dqp table*/

        self.core.qp_y = GET_LUMA_QP(self.core.qp as i8) as u8;

        let qp_i_cb = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_u_offset,
        );
        let qp_i_cr = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_v_offset,
        );

        self.core.qp_u = (self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;
        self.core.qp_v = (self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;

        self.pinter.qp_y = self.core.qp_y;
        self.pinter.qp_u = self.core.qp_u;
        self.pinter.qp_v = self.core.qp_v;

        self.evce_rdoq_bit_est(log2_cuw as usize, log2_cuh as usize);
    }

    fn evce_rdoq_bit_est(&mut self, log2_cuw: usize, log2_cuh: usize) {
        let sbac_ctx = &self.core.c_curr_best[log2_cuw - 2][log2_cuh - 2];
        for bin in 0..2 {
            self.core.rdoq_est.cbf_luma[bin] = biari_no_bits(bin, sbac_ctx.cbf_luma[0]) as i64;
            self.core.rdoq_est.cbf_cb[bin] = biari_no_bits(bin, sbac_ctx.cbf_cb[0]) as i64;
            self.core.rdoq_est.cbf_cr[bin] = biari_no_bits(bin, sbac_ctx.cbf_cr[0]) as i64;
            self.core.rdoq_est.cbf_all[bin] = biari_no_bits(bin, sbac_ctx.cbf_all[0]) as i64;
        }

        for ctx in 0..NUM_CTX_CC_RUN {
            for bin in 0..2 {
                self.core.rdoq_est.run[ctx][bin] = biari_no_bits(bin, sbac_ctx.run[ctx]);
            }
        }

        for ctx in 0..NUM_CTX_CC_LEVEL {
            for bin in 0..2 {
                self.core.rdoq_est.level[ctx][bin] = biari_no_bits(bin, sbac_ctx.level[ctx]);
            }
        }

        for ctx in 0..NUM_CTX_CC_LAST {
            for bin in 0..2 {
                self.core.rdoq_est.last[ctx][bin] = biari_no_bits(bin, sbac_ctx.last[ctx]);
            }
        }
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_intra_luma(&mut self, slice_type: SliceType) {
        let log2_cuw = self.core.log2_cuw;
        let log2_cuh = self.core.log2_cuh;

        if slice_type != SliceType::EVC_ST_I && evc_check_all_preds(&self.core.tree_cons) {
            self.core.s_temp_run.encode_bin(
                &mut self.core.bs_temp,
                &mut self.core.c_temp_run.skip_flag
                    [self.core.ctx_flags[CtxNevIdx::CNID_SKIP_FLAG as usize] as usize],
                0,
            ); /* skip_flag */
            evce_eco_pred_mode(
                &mut self.core.bs_temp,
                &mut self.core.s_temp_run,
                &mut self.core.c_temp_run,
                PredMode::MODE_INTRA,
                self.core.ctx_flags[CtxNevIdx::CNID_PRED_MODE as usize] as usize,
            );
        }

        evce_eco_intra_dir_b(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            self.core.ipm[0] as u8,
            self.core.mpm_b_list,
        );

        if self.pps.cu_qp_delta_enabled_flag {
            self.core.cu_qp_delta_code = self.core.dqp_temp_run.cu_qp_delta_code;
            self.core.cu_qp_delta_is_coded = self.core.dqp_temp_run.cu_qp_delta_is_coded;
            self.core.qp_prev_eco = self.core.dqp_temp_run.prev_QP;
        }

        evce_eco_coef(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            &self.core.ctmp, //&self.pintra.coef_tmp,
            log2_cuw,
            log2_cuh,
            PredMode::MODE_INTRA,
            &self.core.nnz,
            false,
            TQC_RUN::RUN_L as u8,
            false,
            self.core.qp,
            &self.core.tree_cons,
            self.sps.dquant_flag,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.cu_qp_delta_code,
            &mut self.core.cu_qp_delta_is_coded,
            &mut self.core.qp_prev_eco,
        );

        if self.pps.cu_qp_delta_enabled_flag {
            self.core.dqp_temp_run.cu_qp_delta_code = self.core.cu_qp_delta_code;
            self.core.dqp_temp_run.cu_qp_delta_is_coded = self.core.cu_qp_delta_is_coded;
            self.core.dqp_temp_run.prev_QP = self.core.qp_prev_eco;
            self.core.dqp_temp_run.curr_QP = self.core.qp;
        }
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_intra_chroma(&mut self, slice_type: SliceType) {
        let log2_cuw = self.core.log2_cuw;
        let log2_cuh = self.core.log2_cuh;

        evce_eco_coef(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            &self.core.ctmp,
            log2_cuw,
            log2_cuh,
            PredMode::MODE_INTRA,
            &self.core.nnz,
            false,
            TQC_RUN::RUN_CB as u8 | TQC_RUN::RUN_CR as u8,
            false,
            0,
            &self.core.tree_cons,
            self.sps.dquant_flag,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.cu_qp_delta_code,
            &mut self.core.cu_qp_delta_is_coded,
            &mut self.core.qp_prev_eco,
        );
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_intra(&mut self, slice_type: SliceType) {
        let log2_cuw = self.core.log2_cuw;
        let log2_cuh = self.core.log2_cuh;

        if slice_type != SliceType::EVC_ST_I && evc_check_all_preds(&self.core.tree_cons) {
            self.core.s_temp_run.encode_bin(
                &mut self.core.bs_temp,
                &mut self.core.c_temp_run.skip_flag
                    [self.core.ctx_flags[CtxNevIdx::CNID_SKIP_FLAG as usize] as usize],
                0,
            ); /* skip_flag */
            evce_eco_pred_mode(
                &mut self.core.bs_temp,
                &mut self.core.s_temp_run,
                &mut self.core.c_temp_run,
                PredMode::MODE_INTRA,
                self.core.ctx_flags[CtxNevIdx::CNID_PRED_MODE as usize] as usize,
            );
        }

        if evc_check_luma(&self.core.tree_cons) {
            evce_eco_intra_dir_b(
                &mut self.core.bs_temp,
                &mut self.core.s_temp_run,
                &mut self.core.c_temp_run,
                self.core.ipm[0] as u8,
                self.core.mpm_b_list,
            );
        }

        if self.pps.cu_qp_delta_enabled_flag {
            self.core.cu_qp_delta_code = self.core.dqp_temp_run.cu_qp_delta_code;
            self.core.cu_qp_delta_is_coded = self.core.dqp_temp_run.cu_qp_delta_is_coded;
            self.core.qp_prev_eco = self.core.dqp_temp_run.prev_QP;
        }

        evce_eco_coef(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            &self.core.ctmp,
            log2_cuw,
            log2_cuh,
            PredMode::MODE_INTRA,
            &self.core.nnz,
            false,
            TQC_RUN::RUN_L as u8 | TQC_RUN::RUN_CB as u8 | TQC_RUN::RUN_CR as u8,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.qp,
            &self.core.tree_cons,
            self.sps.dquant_flag,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.cu_qp_delta_code,
            &mut self.core.cu_qp_delta_is_coded,
            &mut self.core.qp_prev_eco,
        );

        if self.pps.cu_qp_delta_enabled_flag {
            self.core.dqp_temp_run.cu_qp_delta_code = self.core.cu_qp_delta_code;
            self.core.dqp_temp_run.cu_qp_delta_is_coded = self.core.cu_qp_delta_is_coded;
            self.core.dqp_temp_run.prev_QP = self.core.qp_prev_eco;
            self.core.dqp_temp_run.curr_QP = self.core.qp;
        }
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_inter(
        &mut self,
        slice_type: SliceType,
        cup: u32,
        pidx: usize,
        mvp_idx: &[u8],
        coef_idx: usize,
    ) {
        //refi=&self.pinter.refi[pidx],
        //mvd =&self.pinter.mvd[pidx],
        //coef=&self.pinter.coef[coef_idx],

        if slice_type != SliceType::EVC_ST_I {
            self.core.s_temp_run.encode_bin(
                &mut self.core.bs_temp,
                &mut self.core.c_temp_run.skip_flag
                    [self.core.ctx_flags[CtxNevIdx::CNID_SKIP_FLAG as usize] as usize],
                0,
            ); /* skip_flag */

            if evc_check_all_preds(&self.core.tree_cons) {
                evce_eco_pred_mode(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    PredMode::MODE_INTER,
                    self.core.ctx_flags[CtxNevIdx::CNID_PRED_MODE as usize] as usize,
                );
            }

            let dir_flag = pidx == InterPredDir::PRED_DIR as usize;

            evce_eco_direct_mode_flag(
                &mut self.core.bs_temp,
                &mut self.core.s_temp_run,
                &mut self.core.c_temp_run,
                dir_flag as u32,
            );

            if pidx != InterPredDir::PRED_DIR as usize {
                evce_eco_inter_pred_idc(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    &self.pinter.refi[pidx],
                    slice_type,
                );

                let refi0 = self.pinter.refi[pidx][REFP_0];
                let refi1 = self.pinter.refi[pidx][REFP_1];
                if slice_type.IS_INTER_SLICE() && REFI_IS_VALID(refi0) {
                    evce_eco_refi(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        self.rpm.num_refp[REFP_0],
                        refi0,
                    );
                    evce_eco_mvp_idx(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        mvp_idx[REFP_0] as u32,
                    );
                    evce_eco_mvd(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        &self.pinter.mvd[pidx][REFP_0],
                    );
                }

                if slice_type == SliceType::EVC_ST_B && REFI_IS_VALID(refi1) {
                    evce_eco_refi(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        self.rpm.num_refp[REFP_1],
                        refi1,
                    );
                    evce_eco_mvp_idx(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        mvp_idx[REFP_1] as u32,
                    );
                    evce_eco_mvd(
                        &mut self.core.bs_temp,
                        &mut self.core.s_temp_run,
                        &mut self.core.c_temp_run,
                        &self.pinter.mvd[pidx][REFP_1],
                    );
                }
            }
        }
        if self.pps.cu_qp_delta_enabled_flag {
            self.core.cu_qp_delta_code = self.core.dqp_temp_run.cu_qp_delta_code;
            self.core.cu_qp_delta_is_coded = self.core.dqp_temp_run.cu_qp_delta_is_coded;
            self.core.qp_prev_eco = self.core.dqp_temp_run.prev_QP;
        }
        evce_eco_coef(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            &self.pinter.coef[coef_idx],
            self.core.log2_cuw,
            self.core.log2_cuh,
            PredMode::MODE_INTER,
            &self.core.nnz,
            false,
            TQC_RUN::RUN_L as u8 | TQC_RUN::RUN_CB as u8 | TQC_RUN::RUN_CR as u8,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.qp,
            &self.core.tree_cons,
            self.sps.dquant_flag,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.cu_qp_delta_code,
            &mut self.core.cu_qp_delta_is_coded,
            &mut self.core.qp_prev_eco,
        );

        if self.pps.cu_qp_delta_enabled_flag {
            self.core.dqp_temp_run.cu_qp_delta_code = self.core.cu_qp_delta_code;
            self.core.dqp_temp_run.cu_qp_delta_is_coded = self.core.cu_qp_delta_is_coded;
            self.core.dqp_temp_run.prev_QP = self.core.qp_prev_eco;
            self.core.dqp_temp_run.curr_QP = self.core.qp;
        }
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_inter_comp(
        &mut self,
        ch_type: usize,
        pidx: usize,
        coef_idx: usize,
    ) {
        //coef=&self.pinter.coef[coef_idx],
        //int* nnz = self.core.nnz;
        //EVCE_SBAC* sbac = &self.core.s_temp_run;
        let log2_cuw = self.core.log2_cuw;
        let log2_cuh = self.core.log2_cuh;
        let run_stats = match ch_type {
            Y_C => TQC_RUN::RUN_L as u8,
            U_C => TQC_RUN::RUN_CB as u8,
            V_C => TQC_RUN::RUN_CR as u8,
            _ => 0,
        };

        evce_eco_coef(
            &mut self.core.bs_temp,
            &mut self.core.s_temp_run,
            &mut self.core.c_temp_run,
            &self.pinter.coef[coef_idx],
            log2_cuw,
            log2_cuh,
            PredMode::MODE_INTER,
            &self.core.nnz,
            false,
            run_stats,
            false,
            if ch_type == Y_C { self.core.qp } else { 0 },
            &self.core.tree_cons,
            self.sps.dquant_flag,
            self.pps.cu_qp_delta_enabled_flag,
            self.core.cu_qp_delta_code,
            &mut self.core.cu_qp_delta_is_coded,
            &mut self.core.qp_prev_eco,
        );
    }

    pub(crate) fn evce_rdo_bit_cnt_cu_skip(
        &mut self,
        slice_type: SliceType,
        mvp_idx0: u32,
        mvp_idx1: u32,
    ) {
        if slice_type != SliceType::EVC_ST_I {
            self.core.s_temp_run.encode_bin(
                &mut self.core.bs_temp,
                &mut self.core.c_temp_run.skip_flag
                    [self.core.ctx_flags[CtxNevIdx::CNID_SKIP_FLAG as usize] as usize],
                1,
            ); /* skip_flag */

            evce_eco_mvp_idx(
                &mut self.core.bs_temp,
                &mut self.core.s_temp_run,
                &mut self.core.c_temp_run,
                mvp_idx0,
            );

            if slice_type == SliceType::EVC_ST_B {
                evce_eco_mvp_idx(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    mvp_idx1,
                );
            }
        }
    }

    pub(crate) fn evce_rdo_bit_cnt_mvp(
        &mut self,
        slice_type: SliceType,
        //refi: &[i8],
        mvd: &[[i16; MV_D]],
        pidx: usize,
        mvp_idx: u8,
    ) {
        let refi = &self.pinter.refi[pidx];

        if pidx != InterPredDir::PRED_DIR as usize {
            let refi0 = refi[REFP_0];
            let refi1 = refi[REFP_1];
            if slice_type.IS_INTER_SLICE() && REFI_IS_VALID(refi0) {
                evce_eco_mvp_idx(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    mvp_idx as u32,
                );
                evce_eco_mvd(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    &mvd[REFP_0],
                );
            }
            if slice_type == SliceType::EVC_ST_B && REFI_IS_VALID(refi1) {
                evce_eco_mvp_idx(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    mvp_idx as u32,
                );
                evce_eco_mvd(
                    &mut self.core.bs_temp,
                    &mut self.core.s_temp_run,
                    &mut self.core.c_temp_run,
                    &mvd[REFP_1],
                );
            }
        }
    }

    pub(crate) fn calc_delta_dist_filter_boundary(
        &mut self, /*, EVC_PIC *pic_rec, EVC_PIC *pic_org, int cuw, int cuh,
                   pel(*src)[MAX_CU_DIM], int s_src, int x, int y, u16 avail_lr, u8 intra_flag,
                   u8 cbf_l, s8 *refi, s16(*mv)[MV_D], u8 is_mv_from_mvf*/
    ) {
        /*int i, j;
        int log2_cuw = CONV_LOG2(cuw);
        int log2_cuh = CONV_LOG2(cuh);
        int x_offset = 8; //for preparing deblocking filter taps
        int y_offset = 8;
        int x_tm = 4; //for calculating template dist
        int y_tm = 4; //must be the same as x_tm
        int log2_x_tm = CONV_LOG2(x_tm);
        int log2_y_tm = CONV_LOG2(y_tm);
        EVC_PIC * pic_dbk = ctx -> pic_dbk;
        int s_l_dbk = pic_dbk ->s_l;
        int s_c_dbk = pic_dbk -> s_c;
        int s_l_org = pic_org -> s_l;
        int s_c_org = pic_org-> s_c;
        pel * dst_y = pic_dbk -> y + y * s_l_dbk + x;
        pel * dst_u = pic_dbk -> u + (y > > 1) * s_c_dbk + (x > > 1);
        pel * dst_v = pic_dbk -> v + (y > > 1) * s_c_dbk + (x > > 1);
        pel * org_y = pic_org-> y + y * s_l_org + x;
        pel* org_u = pic_org -> u + (y >> 1) * s_c_org + (x > > 1);
        pel * org_v = pic_org -> v + (y > > 1) * s_c_org + (x > > 1);
        int x_scu = x > > MIN_CU_LOG2;
        int y_scu = y >> MIN_CU_LOG2;
        int t = x_scu + y_scu * ctx -> w_scu;
        //cu info to save
        u8 intra_flag_save, cbf_l_save;*/
        //let do_filter = false;
        //int y_begin = ((ctx -> tile[self.core. tile_num].ctba_rs_first) / ctx -> w_lcu) < < ctx -> log2_max_cuwh;
        //int y_begin_uv = (((ctx -> tile[core -> tile_num].ctba_rs_first) / ctx -> w_lcu) << ctx -> log2_max_cuwh) > > 1;

        if !self.sh.deblocking_filter_on {
            self.core.delta_dist[Y_C] = 0;
            self.core.delta_dist[U_C] = 0;
            self.core.delta_dist[V_C] = 0;
            return; //if no filter is applied, just return delta_dist as 0
        }

        //unimplemented!();
    }
}
