use super::*;

/* picture store structure */
#[derive(Default)]
pub(crate) struct EvcPic {
    /*
    /* Address of Y buffer (include padding) */
    pel             *buf_y;
    /* Address of U buffer (include padding) */
    pel             *buf_u;
    /* Address of V buffer (include padding) */
    pel             *buf_v;
    /* Start address of Y component (except padding) */
    pel             *y;
    /* Start address of U component (except padding)  */
    pel             *u;
    /* Start address of V component (except padding)  */
    pel             *v;
    /* Stride of luma picture */
    int              s_l;
    /* Stride of chroma picture */
    int              s_c;
    /* Width of luma picture */
    int              w_l;
    /* Height of luma picture */
    int              h_l;
    /* Width of chroma picture */
    int              w_c;
    /* Height of chroma picture */
    int              h_c;
    /* padding size of luma */
    int              pad_l;
    /* padding size of chroma */
    int              pad_c;
    /* image buffer */
    EVC_IMGB       * imgb;
    /* presentation temporal reference of this picture */

     */
    poc: u32,
    /* 0: not used for reference buffer, reference picture type */
    is_ref: bool,
    /* needed for output? */
    need_for_out: bool,
    /* scalable layer id */
    temporal_id: u8,
    /*
        s16            (*map_mv)[REFP_NUM][MV_D];
    #if DMVR_LAG
        s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
    #endif
        s8             (*map_refi)[REFP_NUM];
        u32              list_poc[MAX_NUM_REF_PICS];
        u8               m_alfCtuEnableFlag[3][510]; //510 = 30*17 -> class A1 resolution with CU ~ 128
        int              pic_deblock_alpha_offset;
        int              pic_deblock_beta_offset;
        int              pic_qp_u_offset;
        int              pic_qp_v_offset;
        u8               digest[N_C][16];
        */
}

/* reference picture structure */
#[derive(Default)]
pub(crate) struct EvcRefP {
    /* address of reference picture */
/*EVC_PIC        * pic;
/* POC of reference picture */
             poc: u32,
s16            (*map_mv)[REFP_NUM][MV_D];
s16            (*map_unrefined_mv)[REFP_NUM][MV_D];
s8             (*map_refi)[REFP_NUM];
u32             *list_poc;*/}

/*****************************************************************************
 * picture manager for DPB in decoder and RPB in encoder
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcPm {
    /* picture store (including reference and non-reference) */
    pic: [Box<EvcPic>; MAX_PB_SIZE],
    /* address of reference pictures */
    pic_ref: [Box<EvcPic>; MAX_NUM_REF_PICS],
    /* maximum reference picture count */
    max_num_ref_pics: u8,
    /* current count of available reference pictures in PB */
    cur_num_ref_pics: u8,
    /* number of reference pictures */
    pub(crate) num_refp: [u8; REFP_NUM],
    /* next output POC */
    poc_next_output: u32,
    /* POC increment */
    poc_increase: u8,
    /* max number of picture buffer */
    max_pb_size: u8,
    /* current picture buffer size */
    cur_pb_size: u8,
    /* address of leased picture for current decoding/encoding buffer */
    pic_lease: Box<EvcPic>,
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
