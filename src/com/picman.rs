use super::*;
use crate::api::frame::*;
use crate::api::util::*;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

/* picture store structure */
//#[derive(Default)]
pub(crate) struct EvcPic<T: Pixel> {
    pub(crate) frame: Frame<T>,

    /* presentation temporal reference of this picture */
    pub(crate) poc: u32,
    /* 0: not used for reference buffer, reference picture type */
    pub(crate) is_ref: bool,
    /* needed for output? */
    pub(crate) need_for_out: bool,
    /* scalable layer id */
    pub(crate) temporal_id: u8,
    /*
        s16            (*map_mv)[REFP_NUM][MV_D];
    #if DMVR_LAG
        s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
    #endif
        s8             (*map_refi)[REFP_NUM];
        */
    pub(crate) list_poc: [u32; MAX_NUM_REF_PICS],

    pub(crate) pic_deblock_alpha_offset: i8,
    pub(crate) pic_deblock_beta_offset: i8,
    pub(crate) pic_qp_u_offset: i8,
    pub(crate) pic_qp_v_offset: i8,
    pub(crate) digest: [[u8; 16]; N_C],
}

impl<T: Pixel> EvcPic<T> {
    fn new(width: usize, height: usize, chroma_sampling: ChromaSampling) -> Self {
        EvcPic {
            frame: Frame::new(width, height, chroma_sampling),
            poc: 0,
            is_ref: false,
            need_for_out: false,
            temporal_id: 0,
            list_poc: [0; MAX_NUM_REF_PICS],
            pic_deblock_alpha_offset: 0,
            pic_deblock_beta_offset: 0,
            pic_qp_u_offset: 0,
            pic_qp_v_offset: 0,
            digest: [[0; 16]; N_C],
        }
    }
}

/* reference picture structure */
//#[derive(Default)]
pub(crate) struct EvcRefP<T: Pixel> {
    /* address of reference picture */
    pub(crate) pic: Option<Rc<RefCell<EvcPic<T>>>>,
    /* POC of reference picture */
    pub(crate) poc: u32,
    /*s16            (*map_mv)[REFP_NUM][MV_D];
    s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
    s8             (*map_refi)[REFP_NUM];
    u32             *list_poc;*/
}

impl<T: Pixel> EvcRefP<T> {
    pub(crate) fn new() -> Self {
        EvcRefP { pic: None, poc: 0 }
    }

    fn set_refp(&mut self, pic_ref: Rc<RefCell<EvcPic<T>>>) {
        /*refp->map_mv   = pic_ref->map_mv;
        refp->map_unrefined_mv = pic_ref->map_mv;
        refp->map_refi = pic_ref->map_refi;
        refp->list_poc = pic_ref->list_poc;*/
        self.poc = pic_ref.borrow().poc;
        self.pic = Some(pic_ref);
    }

    fn copy_refp(&mut self, refp_src: &EvcRefP<T>) {
        /*
            refp_dst->map_mv   = refp_src->map_mv;
        #if DMVR_LAG
            refp_dst->map_unrefined_mv = refp_src->map_mv;
        #endif
            refp_dst->map_refi = refp_src->map_refi;
            refp_dst->list_poc = refp_src->list_poc;*/

        self.poc = refp_src.poc;
        self.pic = if let Some(pic) = &refp_src.pic {
            Some(Rc::clone(pic))
        } else {
            None
        };
    }
}

/*****************************************************************************
 * picture manager for DPB in decoder and RPB in encoder
 *****************************************************************************/
//#[derive(Default)]
pub(crate) struct EvcPm<T: Pixel> {
    /* picture store (including reference and non-reference) */
    pub(crate) pic: Vec<Option<Rc<RefCell<EvcPic<T>>>>>, //[Option<Rc<RefCell<EvcPic<T>>>>; MAX_PB_SIZE],
    /* address of reference pictures */
    pub(crate) pic_ref: Vec<Option<Rc<RefCell<EvcPic<T>>>>>, //[Option<Rc<RefCell<EvcPic<T>>>>; MAX_NUM_REF_PICS],
    /* maximum reference picture count */
    pub(crate) max_num_ref_pics: u8,
    /* current count of available reference pictures in PB */
    pub(crate) cur_num_ref_pics: u8,
    /* number of reference pictures */
    pub(crate) num_refp: [u8; REFP_NUM],
    /* next output POC */
    pub(crate) poc_next_output: u32,
    /* POC increment */
    pub(crate) poc_increase: u8,
    /* max number of picture buffer */
    pub(crate) max_pb_size: u8,
    /* current picture buffer size */
    pub(crate) cur_pb_size: u8,
    /* address of leased picture for current decoding/encoding buffer */
    pub(crate) pic_lease: Option<Rc<RefCell<EvcPic<T>>>>,
    /* picture buffer allocator */
    //PICBUF_ALLOCATOR pa;
}

impl<T: Pixel> EvcPm<T> {
    pub(crate) fn new() -> Self {
        let mut pic = vec![];
        for i in 0..MAX_PB_SIZE {
            pic.push(None);
        }
        let mut pic_ref = vec![];
        for i in 0..MAX_NUM_REF_PICS {
            pic_ref.push(None);
        }

        EvcPm {
            pic,     //[None; MAX_PB_SIZE],
            pic_ref, //[None; MAX_NUM_REF_PICS],
            max_num_ref_pics: 0,
            cur_num_ref_pics: 0,
            num_refp: [0; REFP_NUM],
            poc_next_output: 0,
            poc_increase: 0,
            max_pb_size: 0,
            cur_pb_size: 0,
            pic_lease: None,
        }
    }

    #[inline]
    fn PRINT_DPB(&self) {
        print!(
            "current num_ref = {}, dpb_size = {}\n",
            self.cur_num_ref_pics,
            self.picman_get_num_allocated_pics()
        );
    }

    fn picman_get_num_allocated_pics(&self) -> u8 {
        let mut cnt = 0;
        for i in 0..MAX_PB_SIZE {
            /* this is coding order */
            if self.pic[i].is_some() {
                cnt += 1;
            }
        }
        cnt
    }

    fn picman_move_pic(pic: &mut [Option<Rc<RefCell<EvcPic<T>>>>], from: usize, to: usize) {
        for i in from..to {
            pic.swap(i, i + 1);
        }
    }

    fn pic_marking_no_rpl(&mut self, ref_pic_gap_length: u32) {
        // mark all pics with layer id > 0 as unused for reference
        /* this is coding order */
        let mut cur_num_ref_pics = 0;
        let mut i = 0;
        let mut tbm = vec![false; MAX_PB_SIZE];
        for pic in &self.pic {
            if let Some(p) = &pic {
                let mut p = p.borrow_mut();
                if p.is_ref
                    && (p.temporal_id > 0
                        || (i > 0 && ref_pic_gap_length > 0 && p.poc % ref_pic_gap_length != 0))
                {
                    p.is_ref = false;
                    tbm[i] = true;
                }

                if p.is_ref {
                    cur_num_ref_pics += 1;
                }
            }
            i += 1;
        }
        for i in 0..tbm.len() {
            if tbm[i] {
                EvcPm::picman_move_pic(&mut self.pic, i, MAX_PB_SIZE - 1);
                tbm[i] = false;
            }
        }

        // TODO: change to signalled num ref pics
        while cur_num_ref_pics >= MAX_NUM_ACTIVE_REF_FRAME {
            for pic in &self.pic {
                if let Some(p) = &pic {
                    let mut p = p.borrow_mut();
                    if p.is_ref {
                        p.is_ref = false;
                        tbm[i] = true;

                        cur_num_ref_pics -= 1;

                        break;
                    }
                }
            }
        }
        for i in 0..tbm.len() {
            if tbm[i] {
                EvcPm::picman_move_pic(&mut self.pic, i, MAX_PB_SIZE - 1);
            }
        }

        self.cur_num_ref_pics = cur_num_ref_pics as u8;
    }

    fn picman_flush_pb(&mut self) {
        /* mark all frames unused */
        for i in 0..MAX_PB_SIZE {
            if let Some(pic) = &self.pic[i] {
                pic.borrow_mut().is_ref = false;
            }
        }
        self.cur_num_ref_pics = 0;
    }

    fn picman_update_pic_ref(&mut self) {
        let mut j = 0;
        for i in 0..MAX_PB_SIZE {
            if let Some(pic) = &self.pic[i] {
                if pic.borrow().is_ref {
                    self.pic_ref[j] = Some(Rc::clone(pic));
                    j += 1;
                }
            }
        }
        let cnt = j;
        while j < MAX_NUM_REF_PICS {
            self.pic_ref[j] = None;
            j += 1;
        }

        /* descending order sort based on POC */
        self.pic_ref[0..cnt].sort_by_key(|k| -(k.as_ref().unwrap().borrow().poc as i32));
    }

    fn picman_remove_pic_from_pb(&mut self, pos: usize) -> Option<Rc<RefCell<EvcPic<T>>>> {
        let pic_rem = self.pic[pos].take();

        /* fill empty pic buffer */
        for i in pos..MAX_PB_SIZE - 1 {
            self.pic.swap(i, i + 1);
        }
        self.pic[MAX_PB_SIZE - 1] = None;

        self.cur_pb_size -= 1;

        return pic_rem;
    }

    fn picman_set_pic_to_pb(
        &mut self,
        pic: Rc<RefCell<EvcPic<T>>>,
        refp: &mut Vec<Vec<EvcRefP<T>>>,
        pos: isize,
    ) {
        for i in 0..self.num_refp[REFP_0] as usize {
            pic.borrow_mut().list_poc[i] = refp[i][REFP_0].poc;
        }
        if pos >= 0 {
            assert!(self.pic[pos as usize].is_none());
            self.pic[pos as usize] = Some(pic);
        } else
        /* pos < 0 */
        {
            /* search empty pic buffer position */
            let mut i = (MAX_PB_SIZE - 1) as isize;
            while i >= 0 {
                if self.pic[i as usize].is_none() {
                    self.pic[i as usize] = Some(pic);
                    break;
                }
            }
            if i < 0 {
                print!("i={}\n", i);
                assert!(i >= 0);
            }
        }
        self.cur_pb_size += 1;
    }

    fn picman_get_empty_pic_from_list(&self) -> Result<usize, EvcError> {
        for i in 0..MAX_PB_SIZE {
            if let Some(pic) = &self.pic[i] {
                let p = pic.borrow();
                if !p.is_ref && !p.need_for_out {
                    //imgb = pic -> imgb;
                    //evc_assert(imgb != NULL);

                    /* check reference count */
                    //if (1 == imgb -> getref(imgb))
                    {
                        return Ok(i); /* this is empty buffer */
                    }
                }
            }
        }

        Err(EvcError::EVC_ERR)
    }

    pub(crate) fn check_copy_refp(
        refp: &mut [[EvcRefP<T>; REFP_NUM]; MAX_NUM_REF_PICS],
        cnt: usize,
        lidx: usize,
        refp_src: &EvcRefP<T>,
    ) -> Result<(), EvcError> {
        for i in 0..cnt {
            if refp[i][lidx].poc == refp_src.poc {
                return Err(EvcError::EVC_ERR);
            }
        }
        refp[cnt][lidx].copy_refp(refp_src);

        Ok(())
    }

    pub(crate) fn evc_picman_get_empty_pic(
        &mut self,
    ) -> Result<Option<Rc<RefCell<EvcPic<T>>>>, EvcError> {
        /* try to find empty picture buffer in list */
        if let Ok(pos) = self.picman_get_empty_pic_from_list() {
            self.pic_lease = self.picman_remove_pic_from_pb(pos);
            if let Some(pic) = &self.pic_lease {
                return Ok(Some(Rc::clone(pic)));
            }
        }
        /* else if available, allocate picture buffer */
        self.cur_pb_size = self.picman_get_num_allocated_pics();

        if self.cur_pb_size < self.max_pb_size {
            /* create picture buffer */
            //pic = pm->pa.fn_alloc(&pm->pa, &ret);
            //evc_assert_gv(pic != NULL, ret, EVC_ERR_OUT_OF_MEMORY, ERR);
            //TODO: add width, height, CS
            self.pic_lease = Some(Rc::new(RefCell::new(EvcPic::new(
                0,
                0,
                ChromaSampling::Cs400,
            ))));
            if let Some(pic) = &self.pic_lease {
                return Ok(Some(Rc::clone(pic)));
            }
        }

        Err(EvcError::EVC_ERR_UNKNOWN)
    }

    /*This is the implementation of reference picture marking based on RPL*/
    pub(crate) fn evc_picman_refpic_marking(&mut self, sh: &EvcSh, poc_val: u32) {}

    pub(crate) fn evc_picman_put_pic(
        &mut self,
        pic: &Option<Rc<RefCell<EvcPic<T>>>>,
        is_idr: bool,
        poc: u32,
        temporal_id: u8,
        need_for_output: bool,
        refp: &mut Vec<Vec<EvcRefP<T>>>,
        ref_pic: bool,
        tool_rpl: bool,
        ref_pic_gap_length: u32,
    ) {
        /* manage RPB */
        if is_idr {
            self.picman_flush_pb();
        }
        //Perform picture marking if RPL approach is not used
        else if !tool_rpl {
            if temporal_id == 0 {
                self.pic_marking_no_rpl(ref_pic_gap_length);
            }
        }

        if let Some(pic) = pic {
            let mut is_ref = {
                let mut p = pic.borrow_mut();
                if !ref_pic {
                    p.is_ref = false;
                } else {
                    p.is_ref = true;
                }

                p.temporal_id = temporal_id;
                p.poc = poc;
                p.need_for_out = need_for_output;
                p.is_ref
            };

            /* put picture into listed RPB */
            if is_ref {
                self.picman_set_pic_to_pb(Rc::clone(pic), refp, self.cur_num_ref_pics as isize);
                self.cur_num_ref_pics += 1;
            } else {
                self.picman_set_pic_to_pb(Rc::clone(pic), refp, -1);
            }
        }

        if self.pic_lease.is_some()
            && pic.is_some()
            && self.pic_lease.as_ref().unwrap().borrow().poc == pic.as_ref().unwrap().borrow().poc
        {
            self.pic_lease = None;
        }

        //self.PRINT_DPB();
    }

    pub(crate) fn evc_picman_out_pic(
        &mut self,
    ) -> Result<Option<Rc<RefCell<EvcPic<T>>>>, EvcError> {
        let mut any_need_for_out = false;
        for i in 0..MAX_PB_SIZE {
            if let Some(pic) = &self.pic[i] {
                let mut ps = pic.borrow_mut();
                if ps.need_for_out {
                    any_need_for_out = true;

                    if ps.poc <= self.poc_next_output {
                        ps.need_for_out = false;
                        self.poc_next_output = ps.poc + self.poc_increase as u32;

                        return Ok(Some(Rc::clone(pic)));
                    }
                }
            }
        }
        if !any_need_for_out {
            Err(EvcError::EVC_ERR_UNEXPECTED)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn evc_picman_init(
        &mut self,
        max_pb_size: u8,
        max_num_ref_pics: u8,
        //PICBUF_ALLOCATOR * pa
    ) -> Result<(), EvcError> {
        if max_num_ref_pics > MAX_NUM_REF_PICS as u8 || max_pb_size > MAX_PB_SIZE as u8 {
            return Err(EvcError::EVC_ERR_UNSUPPORTED);
        }
        self.max_num_ref_pics = max_num_ref_pics;
        self.max_pb_size = max_pb_size;
        self.poc_increase = 1;
        self.pic_lease = None;

        //evc_mcpy(&pm->pa, pa, sizeof(PICBUF_ALLOCATOR));
        Ok(())
    }

    pub(crate) fn evc_picman_refp_init(
        &mut self,
        max_num_ref_pics: u8,
        slice_type: SliceType,
        poc: u32,
        layer_id: u8,
        last_intra: i32,
        refp: &mut Vec<Vec<EvcRefP<T>>>,
    ) -> Result<(), EvcError> {
        if slice_type == SliceType::EVC_ST_I {
            return Ok(());
        }

        self.picman_update_pic_ref();
        evc_assert_rv(self.cur_num_ref_pics > 0, EvcError::EVC_ERR_UNEXPECTED)?;

        for i in 0..MAX_NUM_REF_PICS {
            refp[i][REFP_0].pic = None;
            refp[i][REFP_1].pic = None;
        }
        self.num_refp[REFP_0] = 0;
        self.num_refp[REFP_1] = 0;

        let (mut i, mut cnt) = (0i8, 0usize);

        /* forward */
        if slice_type == SliceType::EVC_ST_P {
            if layer_id > 0 {
                i = 0;
                cnt = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics as usize {
                    if let Some(pic_ref) = &self.pic_ref[i as usize] {
                        let pr = pic_ref.borrow();
                        /* if(poc >= last_intra && pm->pic_ref[i]->poc < last_intra) continue; */
                        if layer_id == 1 {
                            if pr.poc < poc && pr.temporal_id <= layer_id {
                                refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                                cnt += 1;
                            }
                        } else if pr.poc < poc && cnt == 0 {
                            refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                            cnt += 1;
                        } else if cnt != 0 && pr.poc < poc && pr.temporal_id <= 1 {
                            refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                            cnt += 1;
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }
            } else
            /* layer_id == 0, non-scalable  */
            {
                i = 0;
                cnt = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics as usize {
                    if let Some(pic_ref) = &self.pic_ref[i as usize] {
                        let pr = pic_ref.borrow();
                        if poc >= last_intra as u32 && pr.poc < last_intra as u32 {
                            continue;
                        }

                        if pr.poc < poc {
                            refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                            cnt += 1;
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }
            }
        } else
        /* SLICE_B */
        {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = 0;
            cnt = 0;
            while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics as usize {
                if let Some(pic_ref) = &self.pic_ref[i as usize] {
                    let pr = pic_ref.borrow();
                    if poc >= last_intra as u32 && pr.poc < last_intra as u32 {
                        continue;
                    }

                    if pr.poc < poc && pr.temporal_id <= next_layer_id {
                        refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                        cnt += 1;
                        next_layer_id = std::cmp::max(pr.temporal_id - 1, 0);
                    }
                    i += 1;
                } else {
                    break;
                }
            }
        }

        if cnt < max_num_ref_pics as usize && slice_type == SliceType::EVC_ST_B {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = self.cur_num_ref_pics as i8 - 1;
            while i >= 0 && cnt < max_num_ref_pics as usize {
                if let Some(pic_ref) = &self.pic_ref[i as usize] {
                    let pr = pic_ref.borrow();
                    if poc >= last_intra as u32 && pr.poc < last_intra as u32 {
                        continue;
                    }

                    if pr.poc > poc && pr.temporal_id <= next_layer_id {
                        refp[cnt][REFP_0].set_refp(Rc::clone(pic_ref));
                        cnt += 1;
                        next_layer_id = std::cmp::max(pr.temporal_id - 1, 0);
                    }
                    i -= 1;
                } else {
                    break;
                }
            }
        }

        evc_assert_rv(cnt > 0, EvcError::EVC_ERR_UNEXPECTED)?;
        self.num_refp[REFP_0] = cnt as u8;

        /* backward */
        if slice_type == SliceType::EVC_ST_B {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = self.cur_num_ref_pics as i8 - 1;
            cnt = 0;
            while i >= 0 && cnt < max_num_ref_pics as usize {
                if let Some(pic_ref) = &self.pic_ref[i as usize] {
                    let pr = pic_ref.borrow();
                    if poc >= last_intra as u32 && pr.poc < last_intra as u32 {
                        continue;
                    }

                    if pr.poc > poc && pr.temporal_id <= next_layer_id {
                        refp[cnt][REFP_1].set_refp(Rc::clone(pic_ref));
                        cnt += 1;
                        next_layer_id = std::cmp::max(pr.temporal_id - 1, 0);
                    }
                    i -= 1;
                } else {
                    break;
                }
            }

            if cnt < max_num_ref_pics as usize {
                next_layer_id = std::cmp::max(layer_id - 1, 0);
                i = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics as usize {
                    if let Some(pic_ref) = &self.pic_ref[i as usize] {
                        let pr = pic_ref.borrow();
                        if poc >= last_intra as u32 && pr.poc < last_intra as u32 {
                            continue;
                        }

                        if pr.poc < poc && pr.temporal_id <= next_layer_id {
                            refp[cnt][REFP_1].set_refp(Rc::clone(pic_ref));
                            cnt += 1;
                            next_layer_id = std::cmp::max(pr.temporal_id - 1, 0);
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }
            }

            evc_assert_rv(cnt > 0, EvcError::EVC_ERR_UNEXPECTED)?;
            self.num_refp[REFP_1] = cnt as u8;
        }

        if slice_type == SliceType::EVC_ST_B {
            self.num_refp[REFP_0] = std::cmp::min(self.num_refp[REFP_0], max_num_ref_pics);
            self.num_refp[REFP_1] = std::cmp::min(self.num_refp[REFP_1], max_num_ref_pics);
        }

        Ok(())
    }
}
