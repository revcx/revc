use super::*;
use crate::api::frame::*;
use crate::api::util::*;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

/* picture store structure */
#[derive(Default)]
pub(crate) struct EvcPic {
    pub(crate) frame: Frame<pel>,

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

/* reference picture structure */
#[derive(Default)]
pub(crate) struct EvcRefP {
    /* address of reference picture */
    pub(crate) pic: Option<Rc<RefCell<EvcPic>>>,
    /* POC of reference picture */
    pub(crate) poc: u32,
    /*s16            (*map_mv)[REFP_NUM][MV_D];
    s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
    s8             (*map_refi)[REFP_NUM];
    u32             *list_poc;*/
}

impl EvcRefP {
    fn set_refp(&mut self, pic_ref: Rc<RefCell<EvcPic>>) {
        /*refp->map_mv   = pic_ref->map_mv;
        refp->map_unrefined_mv = pic_ref->map_mv;
        refp->map_refi = pic_ref->map_refi;
        refp->list_poc = pic_ref->list_poc;*/
        self.poc = pic_ref.borrow().poc;
        self.pic = Some(pic_ref);
    }
}

/*****************************************************************************
 * picture manager for DPB in decoder and RPB in encoder
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPm {
    /* picture store (including reference and non-reference) */
    pub(crate) pic: [Option<Rc<RefCell<EvcPic>>>; MAX_PB_SIZE],
    /* address of reference pictures */
    pub(crate) pic_ref: [Option<Rc<RefCell<EvcPic>>>; MAX_NUM_REF_PICS],
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
    pub(crate) pic_lease: Option<Rc<RefCell<EvcPic>>>,
    /* picture buffer allocator */
    //PICBUF_ALLOCATOR pa;
}

impl EvcPm {
    fn picman_get_num_allocated_pics(&self) -> i32 {
        let mut cnt = 0;
        for i in 0..MAX_PB_SIZE {
            /* this is coding order */
            if self.pic[i].is_some() {
                cnt += 1;
            }
        }
        cnt
    }

    fn picman_move_pic(pic: &mut [Option<Rc<RefCell<EvcPic>>>], from: usize, to: usize) {
        for i in from..to {
            pic.swap(i, i + 1);
        }
    }

    fn pic_marking_no_rpl(&mut self, ref_pic_gap_length: u32) {
        // mark all pics with layer id > 0 as unused for reference
        /* this is coding order */
        self.pic
            .iter()
            .scan(-1, |i, pic| {
                *i += 1;
                if let Some(p) = pic {
                    let mut p = p.borrow_mut();
                    if p.is_ref
                        && (p.temporal_id > 0
                            || (*i > 0
                                && ref_pic_gap_length > 0
                                && p.poc % ref_pic_gap_length != 0))
                    {
                        p.is_ref = false;
                    }
                }

                Some(pic)
            })
            .collect::<Vec<&Option<Rc<RefCell<EvcPic>>>>>();

        let tbm = self
            .pic
            .iter()
            .map(|x| x.is_some() && x.as_ref().unwrap().borrow().is_ref)
            .collect::<Vec<bool>>();

        assert_eq!(tbm.len(), MAX_PB_SIZE);
        for i in 0..tbm.len() {
            if tbm[i] {
                EvcPm::picman_move_pic(&mut self.pic, i, MAX_PB_SIZE - 1);
            }
        }

        let cur_num_ref_pics = self.pic.iter().fold(0, |acc, x| {
            acc + if x.is_some() && x.as_ref().unwrap().borrow().is_ref {
                1
            } else {
                0
            }
        });

        // TODO: change to signalled num ref pics
        if cur_num_ref_pics >= MAX_NUM_ACTIVE_REF_FRAME {
            self.pic
                .iter()
                .scan(cur_num_ref_pics, |i, pic| {
                    if *i < MAX_NUM_ACTIVE_REF_FRAME {
                        None
                    } else {
                        if let Some(p) = pic {
                            let mut p = p.borrow_mut();
                            if p.is_ref {
                                p.is_ref = false;
                                *i -= 1;
                            }
                        }
                        Some(pic)
                    }
                })
                .collect::<Vec<&Option<Rc<RefCell<EvcPic>>>>>();

            let tbm = self
                .pic
                .iter()
                .map(|x| x.is_some() && x.as_ref().unwrap().borrow().is_ref)
                .collect::<Vec<bool>>();

            assert_eq!(tbm.len(), MAX_PB_SIZE);
            for i in 0..tbm.len() {
                if tbm[i] {
                    EvcPm::picman_move_pic(&mut self.pic, i, MAX_PB_SIZE - 1);
                }
            }
        }

        self.cur_num_ref_pics = self.pic.iter().fold(0, |acc, x| {
            acc + if x.is_some() && x.as_ref().unwrap().borrow().is_ref {
                1
            } else {
                0
            }
        });
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

    pub(crate) fn evc_picman_refp_init(
        &mut self,
        max_num_ref_pics: u8,
        slice_type: SliceType,
        poc: u32,
        layer_id: u8,
        last_intra: i32,
        refp: &mut [[EvcRefP; REFP_NUM]; MAX_NUM_REF_PICS],
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
