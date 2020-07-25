use super::*;
use crate::api::*;
use crate::def::*;
use crate::picman::*;

use std::cell::RefCell;
use std::rc::Rc;

/*****************************************************************************
 * intra prediction structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcePIntra {
    /* temporary prediction buffer */
    pred: CUBuffer<pel>, //[N_C][MAX_CU_DIM];
    //pred_cache: [[pel; MAX_CU_DIM]; IntraPredDir::IPD_CNT_B as usize], // only for luma

    /* reconstruction buffer */
    rec: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    coef_tmp: CUBuffer<i16>,  //[N_C][MAX_CU_DIM];
    coef_best: CUBuffer<i16>, //[N_C][MAX_CU_DIM];
    nnz_best: [u16; N_C],
    rec_best: CUBuffer<pel>, //[N_C][MAX_CU_DIM];

    /* original (input) picture buffer */
    pic_o: Option<Rc<RefCell<EvcPic>>>,
    /* mode picture buffer */
    pic_m: Option<Rc<RefCell<EvcPic>>>,

    /* QP for luma */
    qp_y: u8,
    /* QP for chroma */
    qp_u: u8,
    qp_v: u8,

    slice_type: SliceType,

    complexity: i64,
    //void              * pdata[4];
    //int               * ndata[4];
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
}
