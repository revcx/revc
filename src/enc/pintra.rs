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

impl EvcePIntra {
    pub(crate) fn pintra_init_frame(
        &mut self,
        slice_type: SliceType,
        pic_orig: &Option<Rc<RefCell<EvcPic>>>,
        pic_mode: &Option<Rc<RefCell<EvcPic>>>,
    ) {
        self.slice_type = slice_type;
        if let Some(pic) = pic_orig {
            self.pic_o = Some(Rc::clone(pic));
        }
        if let Some(pic) = pic_mode {
            self.pic_m = Some(Rc::clone(pic));
        }
    }
}
