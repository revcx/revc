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

impl EvceMode {
    pub(crate) fn mode_init_frame(&mut self, log2_max_cuwh: u8) {
        /* set default values to mode information */
        self.log2_culine = log2_max_cuwh - MIN_CU_LOG2 as u8;

        /* initialize pintra */
        //pintra_init_frame(ctx)?;

        /* initialize pinter */
        //pinter_init_frame(ctx)?;
    }
}
