use super::*;
use crate::api::frame::*;
use crate::api::util::*;

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
    pub(crate) pic: Box<EvcPic>,
    /* POC of reference picture */
    pub(crate) poc: u32,
    /*s16            (*map_mv)[REFP_NUM][MV_D];
    s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
    s8             (*map_refi)[REFP_NUM];
    u32             *list_poc;*/
}

/*****************************************************************************
 * picture manager for DPB in decoder and RPB in encoder
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPm {
    /* picture store (including reference and non-reference) */
    pub(crate) pic: [Box<EvcPic>; MAX_PB_SIZE],
    /* address of reference pictures */
    pub(crate) pic_ref: [Box<EvcPic>; MAX_NUM_REF_PICS],
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
    pub(crate) pic_lease: Box<EvcPic>,
    /* picture buffer allocator */
    //PICBUF_ALLOCATOR pa;
}

impl EvcPm {
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
        //self.pic_lease = NULL;

        //evc_mcpy(&pm->pa, pa, sizeof(PICBUF_ALLOCATOR));
        Ok(())
    }

    fn picman_update_pic_ref(&mut self) {
        /*EVC_PIC ** pic;
        EVC_PIC ** pic_ref;
        EVC_PIC  * pic_t;
        int i, j, cnt;

        pic = pm->pic;
        pic_ref = pm->pic_ref;

        for(i = 0, j = 0; i < MAX_PB_SIZE; i++)
        {
            if(pic[i] && IS_REF(pic[i]))
            {
                pic_ref[j++] = pic[i];
            }
        }
        cnt = j;
        while(j < MAX_NUM_REF_PICS) pic_ref[j++] = NULL;

        /* descending order sort based on POC */
        for(i = 0; i < cnt - 1; i++)
        {
            for(j = i + 1; j < cnt; j++)
            {
                if(pic_ref[i]->poc < pic_ref[j]->poc)
                {
                    pic_t = pic_ref[i];
                    pic_ref[i] = pic_ref[j];
                    pic_ref[j] = pic_t;
                }
            }
        }*/
    }

    pub(crate) fn evc_picman_refp_init(
        &mut self,
        max_num_ref_pics: u8,
        slice_type: SliceType,
        poc: u32,
        layer_id: u8,
        last_intra: i32,
        refp: &[[EvcRefP; REFP_NUM]; MAX_NUM_REF_PICS],
    ) -> Result<(), EvcError> {
        if slice_type == SliceType::EVC_ST_I {
            return Ok(());
        }

        //picman_update_pic_ref(pm);
        //evc_assert_rv(pm->cur_num_ref_pics > 0, EVC_ERR_UNEXPECTED);

        //for i = 0; i < MAX_NUM_REF_PICS; i++)
        //{
        //    refp[i][REFP_0].pic = refp[i][REFP_1].pic = NULL;
        //}
        self.num_refp[REFP_0] = 0;
        self.num_refp[REFP_1] = 0;

        let (mut i, mut cnt) = (0i8, 0);

        /* forward */
        if slice_type == SliceType::EVC_ST_P {
            if layer_id > 0 {
                i = 0;
                cnt = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics {
                    /* if(poc >= last_intra && pm->pic_ref[i]->poc < last_intra) continue; */
                    if layer_id == 1 {
                        if self.pic_ref[i as usize].poc < poc
                            && self.pic_ref[i as usize].temporal_id <= layer_id
                        {
                            //set_refp(&refp[cnt][REFP_0], self.pic_ref[i]);
                            cnt += 1;
                        }
                    } else if self.pic_ref[i as usize].poc < poc && cnt == 0 {
                        //set_refp(&refp[cnt][REFP_0], pm->pic_ref[i]);
                        cnt += 1;
                    } else if cnt != 0
                        && self.pic_ref[i as usize].poc < poc
                        && self.pic_ref[i as usize].temporal_id <= 1
                    {
                        //set_refp(&refp[cnt][REFP_0], pm->pic_ref[i]);
                        cnt += 1;
                    }
                    i += 1;
                }
            } else
            /* layer_id == 0, non-scalable  */
            {
                i = 0;
                cnt = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics {
                    if poc >= last_intra as u32 && self.pic_ref[i as usize].poc < last_intra as u32
                    {
                        continue;
                    }

                    if self.pic_ref[i as usize].poc < poc {
                        //set_refp(&refp[cnt][REFP_0], pm->pic_ref[i]);
                        cnt += 1;
                    }
                    i += 1;
                }
            }
        } else
        /* SLICE_B */
        {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = 0;
            cnt = 0;
            while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics {
                if poc >= last_intra as u32 && self.pic_ref[i as usize].poc < last_intra as u32 {
                    continue;
                }

                if self.pic_ref[i as usize].poc < poc
                    && self.pic_ref[i as usize].temporal_id <= next_layer_id
                {
                    //set_refp(&refp[cnt][REFP_0], pm->pic_ref[i]);
                    cnt += 1;
                    next_layer_id = std::cmp::max(self.pic_ref[i as usize].temporal_id - 1, 0);
                }
                i += 1;
            }
        }

        if cnt < max_num_ref_pics && slice_type == SliceType::EVC_ST_B {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = self.cur_num_ref_pics as i8 - 1;
            while i >= 0 && cnt < max_num_ref_pics {
                if poc >= last_intra as u32 && self.pic_ref[i as usize].poc < last_intra as u32 {
                    continue;
                }

                if self.pic_ref[i as usize].poc > poc
                    && self.pic_ref[i as usize].temporal_id <= next_layer_id
                {
                    //set_refp(&refp[cnt][REFP_0], pm->pic_ref[i]);
                    cnt += 1;
                    next_layer_id = std::cmp::max(self.pic_ref[i as usize].temporal_id - 1, 0);
                }
                i -= 1;
            }
        }

        evc_assert_rv(cnt > 0, EvcError::EVC_ERR_UNEXPECTED)?;
        self.num_refp[REFP_0] = cnt;

        /* backward */
        if slice_type == SliceType::EVC_ST_B {
            let mut next_layer_id = std::cmp::max(layer_id - 1, 0);
            i = self.cur_num_ref_pics as i8 - 1;
            cnt = 0;
            while i >= 0 && cnt < max_num_ref_pics {
                if poc >= last_intra as u32 && self.pic_ref[i as usize].poc < last_intra as u32 {
                    continue;
                }

                if self.pic_ref[i as usize].poc > poc
                    && self.pic_ref[i as usize].temporal_id <= next_layer_id
                {
                    //set_refp(&refp[cnt][REFP_1], pm->pic_ref[i]);
                    cnt += 1;
                    next_layer_id = std::cmp::max(self.pic_ref[i as usize].temporal_id - 1, 0);
                }
                i -= 1;
            }

            if cnt < max_num_ref_pics {
                next_layer_id = std::cmp::max(layer_id - 1, 0);
                i = 0;
                while i < self.cur_num_ref_pics as i8 && cnt < max_num_ref_pics {
                    if poc >= last_intra as u32 && self.pic_ref[i as usize].poc < last_intra as u32
                    {
                        continue;
                    }

                    if self.pic_ref[i as usize].poc < poc
                        && self.pic_ref[i as usize].temporal_id <= next_layer_id
                    {
                        //set_refp(&refp[cnt][REFP_1], pm->pic_ref[i]);
                        cnt += 1;
                        next_layer_id = std::cmp::max(self.pic_ref[i as usize].temporal_id - 1, 0);
                    }
                    i += 1;
                }
            }

            evc_assert_rv(cnt > 0, EvcError::EVC_ERR_UNEXPECTED)?;
            self.num_refp[REFP_1] = cnt;
        }

        if slice_type == SliceType::EVC_ST_B {
            self.num_refp[REFP_0] = std::cmp::min(self.num_refp[REFP_0], max_num_ref_pics);
            self.num_refp[REFP_1] = std::cmp::min(self.num_refp[REFP_1], max_num_ref_pics);
        }

        Ok(())
    }
}
