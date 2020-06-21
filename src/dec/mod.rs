use super::api::*;
use super::com::tbl::*;
use super::com::util::*;
use super::com::*;

mod bsr;
mod eco;
mod sbac;

use bsr::EvcdBsr;
use eco::*;
use sbac::*;

/* evc decoder magic code */
pub(crate) const EVCD_MAGIC_CODE: u32 = 0x45565944; /* EVYD */

/*****************************************************************************
 * CORE information used for decoding process.
 *
 * The variables in this structure are very often used in decoding process.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCore {
    /*
    /************** current CU **************/
    /* coefficient buffer of current CU */
                coef: [[s16;MAX_CU_DIM]; N_C], //[N_C][MAX_CU_DIM]
    /* pred buffer of current CU */
    /* [1] is used for bi-pred. */
                pred: [[[pel;MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]
                dmvr_template:[pel; MAX_CU_DIM], //[MAX_CU_DIM]
    pel            dmvr_half_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + 1) * (MAX_CU_SIZE + 1)];
    pel            dmvr_ref_pred_interpolated[REFP_NUM][(MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT)) * (MAX_CU_SIZE + ((DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT))];

    #if DMVR_PADDING
    pel  dmvr_padding_buf[2][N_C][PAD_BUFFER_STRIDE * PAD_BUFFER_STRIDE];
    #endif
    /* neighbor pixel buffer for intra prediction */
    pel            nb[N_C][N_REF][MAX_CU_SIZE * 3];
    /* reference index for current CU */
    s8             refi[REFP_NUM];
    /* motion vector for current CU */
    s16            mv[REFP_NUM][MV_D];
    #if DMVR_LAG
    /* dmvr refined motion vector for current CU */
    s16             dmvr_mv[MAX_CU_CNT_IN_LCU][REFP_NUM][MV_D];
    #endif
    /* CU position in current frame in SCU unit */
    u32            scup;
    /* CU position X in a frame in SCU unit */

     */
    x_scu: u16,
    /* CU position Y in a frame in SCU unit */
    y_scu: u16,
    /*
    /* neighbor CUs availability of current CU */
    u16            avail_cu;
    /* Left, right availability of current CU */
    u16            avail_lr;
    /* intra prediction direction of current CU */
    u8             ipm[2];
    /* most probable mode for intra prediction */
    u8             * mpm_b_list;
    u8             mpm[2];
    u8             mpm_ext[8];
    u8             pims[IPD_CNT]; /* probable intra mode set*/
    /* prediction mode of current CU: INTRA, INTER, ... */
    u8             pred_mode;
    u8             DMVRenable;
    /* log2 of cuw */
    u8             log2_cuw;
    /* log2 of cuh */
    u8             log2_cuh;
    /* is there coefficient? */
    int            is_coef[N_C];
    int            is_coef_sub[N_C][MAX_SUB_TB_NUM];

         */
    /* QP for Luma of current encoding MB */
    qp_y: u8,
    /* QP for Chroma of current encoding MB */
    qp_u: u8,
    qp_v: u8,

    //s16            affine_mv[REFP_NUM][VER_NUM][MV_D];
    //u8             affine_flag;

    //u8             ibc_flag;
    //u8             ibc_skip_flag;
    //u8             ibc_merge_flag;
    qp: u8,
    cu_qp_delta_code: u8,
    cu_qp_delta_is_coded: bool,

    /************** current LCU *************/
    /* address of current LCU,  */
    lcu_num: u16,
    /* X address of current LCU */
    x_lcu: u16,
    /* Y address of current LCU */
    y_lcu: u16,
    /* left pel position of current LCU */
    x_pel: u16,
    /* top pel position of current LCU */
    y_pel: u16,
    /*
    /* split mode map for current LCU */
    s8             split_mode[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    /* SUCO flag for current LCU */
    s8             suco_flag[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    /* platform specific data, if needed */
    void          *pf;
    s16            mmvd_idx;
    u8             mmvd_flag;
    /* ATS_INTRA flags */
    u8             ats_intra_cu;
    u8             ats_intra_mode_h;
    u8             ats_intra_mode_v;

    /* ATS_INTER info (index + position)*/
    u8             ats_inter_info;
    /* temporal pixel buffer for inter prediction */
    pel            eif_tmp_buffer[ (MAX_CU_SIZE + 2) * (MAX_CU_SIZE + 2) ];
    u8             mvr_idx;
    #if DMVR_FLAG
    u8            dmvr_flag;
    #endif

    /* history-based motion vector prediction candidate list */
    EVC_HISTORY_BUFFER     history_buffer;
    #if AFFINE_UPDATE
    // spatial neighboring MV of affine block
    s8             refi_sp[REFP_NUM];
    s16            mv_sp[REFP_NUM][MV_D];
    #endif
    #if TRACE_ENC_CU_DATA
    u64            trace_idx;
    #endif
    int            mvp_idx[REFP_NUM];
    s16            mvd[REFP_NUM][MV_D];
    int            inter_dir;
    int            bi_idx;
    int            affine_bzero[REFP_NUM];
    s16            affine_mvd[REFP_NUM][3][MV_D];
    int            tile_num;
    u8             ctx_flags[NUM_CNID];
    #if M50761_CHROMA_NOT_SPLIT
    TREE_CONS      tree_cons;
    #endif
    */
}
/******************************************************************************
 * CONTEXT used for decoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCtx {
    /* magic code */
    pub(crate) magic: u32,

    /* EVCD identifier */
    //EVCD                    id;
    /* CORE information used for fast operation */
    core: EvcdCore,
    /* current decoding bitstream */
    bs: EvcdBsr,
    /* current nalu header */
    nalu: EvcNalu,
    /* current slice header */
    sh: EvcSh,
    /* decoded picture buffer management */
    // EVC_PM                  dpm;
    /* create descriptor */
    //EVCD_CDSC               cdsc;
    /* sequence parameter set */
    sps: EvcSps,
    /* picture parameter set */
    pps: EvcPps,
    /* current decoded (decoding) picture buffer */
    //EVC_PIC               * pic;
    /* SBAC */
    sbac_dec: EvcdSbac,
    /* decoding picture width */
    pub(crate) w: u16,
    /* decoding picture height */
    pub(crate) h: u16,
    /* maximum CU width and height */
    max_cuwh: u16,
    /* log2 of maximum CU width and height */
    log2_max_cuwh: u8,

    /* minimum CU width and height */
    min_cuwh: u16,
    /* log2 of minimum CU width and height */
    log2_min_cuwh: u8,
    /* MAPS *******************************************************************/
    /* SCU map for CU information */
    /*u32                   * map_scu;
    /* LCU split information */
    s8                   (* map_split)[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    s8                   (* map_suco)[NUM_CU_DEPTH][NUM_BLOCK_SHAPE][MAX_CU_CNT_IN_LCU];
    /* decoded motion vector for every blocks */
    s16                  (* map_mv)[REFP_NUM][MV_D];
    /* decoded motion vector for every blocks */
    s16                  (* map_unrefined_mv)[REFP_NUM][MV_D];
    /* reference frame indices */
    s8                   (* map_refi)[REFP_NUM];
    /* intra prediction modes */
    s8                    * map_ipm;
    u32                   * map_affine;
    /* new coding tool flag*/
    u32                   * map_cu_mode;
    /* ats_inter info map */
    u8                    * map_ats_inter;*/
    /**************************************************************************/
    /* current slice number, which is increased whenever decoding a slice.
    when receiving a slice for new picture, this value is set to zero.
    this value can be used for distinguishing b/w slices */
    slice_num: u16,
    /* last coded intra picture's picture order count */
    last_intra_poc: i32,
    /* picture width in LCU unit */
    w_lcu: u16,
    /* picture height in LCU unit */
    h_lcu: u16,
    /* picture size in LCU unit (= w_lcu * h_lcu) */
    f_lcu: u32,
    /* picture width in SCU unit */
    w_scu: u16,
    /* picture height in SCU unit */
    h_scu: u16,
    /* picture size in SCU unit (= w_scu * h_scu) */
    f_scu: u32,
    /* the picture order count value */
    poc: EvcPoc,
    /* the picture order count of the previous Tid0 picture */
    prev_pic_order_cnt_val: u32,
    /* the decoding order count of the previous picture */
    prev_doc_offset: u32,
    /* the number of currently decoded pictures */
    pic_cnt: u32,
    /* flag whether current picture is refecened picture or not */
    slice_ref_flag: bool,
    /* distance between ref pics in addition to closest ref ref pic in LD*/
    ref_pic_gap_length: isize,
    /* bitstream has an error? */
    bs_err: u8,
    /* reference picture (0: foward, 1: backward) */
    //EVC_REFP                refp[MAX_NUM_REF_PICS][REFP_NUM];
    /* flag for picture signature enabling */
    use_pic_sign: u8,
    /* picture signature (MD5 digest 128bits) for each component */
    pic_sign: [[u8; 16]; N_C],
    /* flag to indicate picture signature existing or not */
    pic_sign_exist: u8,
    /* flag to indicate opl decoder output */
    use_opl: u8,
    num_ctb: u32,
}

const nalu_size_field_in_bytes: usize = 4;

impl EvcdCtx {
    fn sequence_deinit(&mut self) {
        /*evc_mfree(self.map_scu);
        evc_mfree(self.map_split);
        evc_mfree(self.map_ipm);
        evc_mfree(self.map_suco);
        evc_mfree(self.map_affine);
        evc_mfree(self.map_cu_mode);
        evc_mfree(self.map_ats_inter);
        evc_mfree_fast(self.map_tidx);
        evc_picman_deinit(&self.dpm);*/
    }

    fn sequence_init(&mut self) {
        if self.sps.pic_width_in_luma_samples != self.w
            || self.sps.pic_height_in_luma_samples != self.h
        {
            /* resolution was changed */
            self.sequence_deinit();

            self.w = self.sps.pic_width_in_luma_samples;
            self.h = self.sps.pic_height_in_luma_samples;

            assert_eq!(self.sps.sps_btt_flag, false);

            self.max_cuwh = 1 << 6;
            self.min_cuwh = 1 << 2;

            self.log2_max_cuwh = CONV_LOG2(self.max_cuwh as usize);
            self.log2_min_cuwh = CONV_LOG2(self.min_cuwh as usize);
        }

        let size = self.max_cuwh;
        self.w_lcu = (self.w + (size - 1)) / size;
        self.h_lcu = (self.h + (size - 1)) / size;
        self.f_lcu = (self.w_lcu * self.h_lcu) as u32;
        self.w_scu = (self.w + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.h_scu = (self.h + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.f_scu = (self.w_scu * self.h_scu) as u32;
    }

    fn slice_init(&mut self) {
        self.core.lcu_num = 0;
        self.core.x_lcu = 0;
        self.core.y_lcu = 0;
        self.core.x_pel = 0;
        self.core.y_pel = 0;
        self.core.qp_y = self.sh.qp + (6 * (BIT_DEPTH - 8)) as u8;
        self.core.qp_u = (p_evc_tbl_qp_chroma_dynamic[0][self.sh.qp_u as usize]
            + (6 * (BIT_DEPTH - 8)) as isize) as u8;
        self.core.qp_v = (p_evc_tbl_qp_chroma_dynamic[1][self.sh.qp_v as usize]
            + (6 * (BIT_DEPTH - 8)) as isize) as u8;

        /* clear maps */
        /*evc_mset_x64a(ctx->map_scu, 0, sizeof(u32) * ctx->f_scu);
        evc_mset_x64a(ctx->map_affine, 0, sizeof(u32) * ctx->f_scu);
        evc_mset_x64a(ctx->map_ats_inter, 0, sizeof(u8) * ctx->f_scu);
        evc_mset_x64a(ctx->map_cu_mode, 0, sizeof(u32) * ctx->f_scu);*/

        if self.sh.slice_type == SliceType::EVC_ST_I {
            self.last_intra_poc = self.poc.poc_val;
        }
    }

    fn update_core_loc_param(&mut self) {
        self.core.x_pel = self.core.x_lcu << self.log2_max_cuwh as u16; // entry point's x location in pixel
        self.core.y_pel = self.core.y_lcu << self.log2_max_cuwh as u16; // entry point's y location in pixel
        self.core.x_scu = self.core.x_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2) as u16; // set x_scu location
        self.core.y_scu = self.core.y_lcu << (MAX_CU_LOG2 - MIN_CU_LOG2) as u16; // set y_scu location
        self.core.lcu_num = self.core.x_lcu + self.core.y_lcu * self.w_lcu; // Init the first lcu_num in tile
    }

    fn decode_slice(&mut self) -> Result<(), EvcError> {
        /*
        let sbac = GET_SBAC_DEC(bs);

        // Initialize CABAC at each tile
        evcd_eco_sbac_reset(bs, self.sh.slice_type, self.sh.qp, self.sps.tool_cm_init);

        self.core.x_lcu = 0; //entry point lcu's x location
        self.core.y_lcu = 0; // entry point lcu's y location
        while self.num_ctb > 0 {
            self.update_core_loc_param();

            //LCU decoding with in a tile
            let mut same_layer_split = vec![0; 4];
            let mut split_allow = vec![0, 0, 0, 0, 0, 1];
            //evc_assert_rv(core->lcu_num < ctx->f_lcu, EVC_ERR_UNEXPECTED);

            // invoke coding_tree() recursion
            //evc_mset(self.core.split_mode, 0, sizeof(s8) * NUM_CU_DEPTH * NUM_BLOCK_SHAPE * MAX_CU_CNT_IN_LCU);

            self.evcd_eco_tree(
                self.core.x_pel,
                self.corey_pel,
                self.log2_max_cuwh,
                self.log2_max_cuwh,
                0,
                0,
                bs,
                sbac,
                1,
                0,
                SplitMode::NO_SPLIT,
                same_layer_split,
                0,
                split_allow,
                0,
                0,
                0,
                ModeCons::eAll,
            )?;
            // set split flags to map
            //evc_mcpy(ctx->map_split[core->lcu_num], core->split_mode, sizeof(s8) * NUM_CU_DEPTH * NUM_BLOCK_SHAPE * MAX_CU_CNT_IN_LCU);
            //evc_mcpy(ctx->map_suco[core->lcu_num], core->suco_flag, sizeof(s8) * NUM_CU_DEPTH * NUM_BLOCK_SHAPE * MAX_CU_CNT_IN_LCU);

            self.num_ctb -= 1;
            // read end_of_picture_flag
            if (self.num_ctb == 0) {
                evcd_eco_tile_end_flag(&mut self.bs, sbac)?;
            } else {
                self.core.x_lcu += 1;
                if self.core.x_lcu >= self.w_ctb {
                    self.core.x_lcu = 0;
                    self.core.y_lcu += 1;
                }
            }
        }
         */

        Ok(())
    }

    pub(crate) fn decode_nalu(&mut self, pkt: &mut Packet) -> Result<EvcdStat, EvcError> {
        let data = pkt.data.take();
        let buf = if let Some(b) = data {
            b
        } else {
            return Err(EvcError::EVC_ERR_EMPTY_PACKET);
        };

        /* bitstream reader initialization */
        self.bs = EvcdBsr::new(buf);

        /* parse nalu header */
        evcd_eco_nalu(&mut self.bs, &mut self.nalu)?;

        let nalu_type = self.nalu.nal_unit_type;
        if nalu_type == NaluType::EVC_SPS_NUT {
            evcd_eco_sps(&mut self.bs, &mut self.sps)?;

            self.sequence_init();
        } else if nalu_type == NaluType::EVC_PPS_NUT {
            evcd_eco_pps(&mut self.bs, &self.sps, &mut self.pps)?;
        } else if nalu_type < NaluType::EVC_SPS_NUT {
            /* decode slice header */
            self.sh.num_ctb = self.f_lcu as u16;

            evcd_eco_sh(&mut self.bs, &self.sps, &self.pps, &mut self.sh, nalu_type)?;

            if self.num_ctb == 0 {
                self.num_ctb = self.f_lcu;
            }

            /* POC derivation process */
            assert_eq!(self.sps.tool_pocs, false);
            if !self.sps.tool_pocs {
                //sps_pocs_flag == 0
                if nalu_type == NaluType::EVC_IDR_NUT {
                    self.sh.poc_lsb = 0;
                    self.poc.prev_doc_offset = -1;
                    self.poc.prev_poc_val = 0;
                    self.slice_ref_flag = (self.nalu.nuh_temporal_id == 0
                        || self.nalu.nuh_temporal_id < self.sps.log2_sub_gop_length);
                    self.poc.poc_val = 0;
                } else {
                    self.slice_ref_flag = (self.nalu.nuh_temporal_id == 0
                        || self.nalu.nuh_temporal_id < self.sps.log2_sub_gop_length);
                    evc_poc_derivation(&self.sps, self.nalu.nuh_temporal_id, &mut self.poc);
                    self.sh.poc_lsb = self.poc.poc_val;
                }
            }

            self.slice_init();

            if self.num_ctb == 0 {
                self.num_ctb = self.f_lcu;
                self.slice_num = 0;
            } else {
                self.slice_num += 1;
            }

            if !self.sps.tool_rpl {
                /* initialize reference pictures */
                //evc_picman_refp_init(&ctx->dpm, ctx->sps.max_num_ref_pics, sh->slice_type, ctx->poc.poc_val, ctx->nalu.nuh_temporal_id, ctx->last_intra_poc, ctx->refp);
            }

            if self.num_ctb == self.f_lcu {
                /* get available frame buffer for decoded image */
                //self.pic = evc_picman_get_empty_pic(&self.dpm)?;

                /* get available frame buffer for decoded image */
                //ctx->map_refi = ctx->pic->map_refi;
                //ctx->map_mv = ctx->pic->map_mv;
            }

            /* decode slice layer */
            self.decode_slice()?;
        } else if nalu_type == NaluType::EVC_SEI_NUT {
        } else {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }

        Ok(EvcdStat {
            read: nalu_size_field_in_bytes + self.bs.EVC_BSR_GET_READ_BYTE() as usize,
            nalu_type,
            stype: self.sh.slice_type,
            fnum: -1,
            ..Default::default()
        })
    }
}
