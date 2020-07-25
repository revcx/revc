use super::*;
use crate::api::*;

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
        let split_mode_child = &mut self.core.split_mode_child;
        let parent_split_allow = &mut self.core.parent_split_allow;
        for i in 0..6 {
            parent_split_allow[i] = false;
            if i == 5 {
                parent_split_allow[i] = true;
            }
        }

        let mi = &mut self.mode;

        /* initialize cu data */
        self.core.cu_data_best[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
            .init(
                self.log2_max_cuwh as usize,
                self.log2_max_cuwh as usize,
                self.qp,
                self.qp,
                self.qp,
            );
        self.core.cu_data_temp[self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize - 2]
            .init(
                self.log2_max_cuwh as usize,
                self.log2_max_cuwh as usize,
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
        /*self.mode_coding_tree(
            self.core.x_pel,
            self.core.y_pel,
            0,
            self.log2_max_cuwh,
            self.log2_max_cuwh,
            0,
            mi,
            1,
            0,
            SplitMode::NO_SPLIT,
            split_mode_child,
            0,
            parent_split_allow,
            0,
            0,
            evc_get_default_tree_cons(),
        );*/

        /*#if TRACE_ENC_CU_DATA_CHECK
                let h = 1 << (self.log2_max_cuwh - MIN_CU_LOG2);
                    let w = 1 << (self.log2_max_cuwh - MIN_CU_LOG2);
                for j in 0..h {
                    let y_pos = self.core.y_pel + (j << MIN_CU_LOG2);
                    for i in 0..w {
                        let x_pos = self.core.x_pel + (i << MIN_CU_LOG2);
                        if x_pos < self.w && y_pos < self.h {
                            assert!(self.core.cu_data_best
                            [self.log2_max_cuwh as usize - 2][self.log2_max_cuwh as usize- 2].trace_idx[i + h * j] != 0);
                        }
                    }
                }
        #endif*/
    }
}
