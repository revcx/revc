use super::*;
use crate::api::*;
use crate::def::*;
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

    /* MV predictor */
    s16  mvp[REFP_NUM][MAX_NUM_MVP][MV_D];

    s16  mv[PRED_NUM][REFP_NUM][MV_D];
    s16  mvd[PRED_NUM][REFP_NUM][MV_D];

    s16  org_bi[MAX_CU_DIM];
    s32  mot_bits[REFP_NUM];
    /* temporary prediction buffer (only used for ME)*/
    pel  pred[PRED_NUM+1][2][N_C][MAX_CU_DIM];

    /* reconstruction buffer */
    pel  rec[PRED_NUM][N_C][MAX_CU_DIM];
    /* last one buffer used for RDO */
    s16  coef[PRED_NUM+1][N_C][MAX_CU_DIM];

    s16  residue[N_C][MAX_CU_DIM];
    int  nnz_best[PRED_NUM][N_C];

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
    qp_y: u8,
    /* QP for chroma of current encoding CU */
    qp_u: u8,
    qp_v: u8,
    lambda_mv: u32,
    /* reference pictures */
    refp: Option<Rc<RefCell<Vec<Vec<EvcRefP>>>>>,
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
        pi.refp = Some(Rc::clone(&self.refp));
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
}
