use super::api::*;
use super::com::tbl::*;
use super::com::util::*;
use super::com::*;

mod bsr;
mod eco;
mod sbac;

use bsr::*;
use eco::*;
use sbac::*;

/* evc decoder magic code */
pub(crate) const EVCD_MAGIC_CODE: u32 = 0x45565944; /* EVYD */

#[derive(Clone)]
pub(crate) struct LcuSplitModeArray {
    pub(crate) array:
        [[[SplitMode; MAX_CU_CNT_IN_LCU]; BlockShape::NUM_BLOCK_SHAPE as usize]; NUM_CU_DEPTH],
}

impl Default for LcuSplitModeArray {
    fn default() -> Self {
        LcuSplitModeArray {
            array: [[[SplitMode::NO_SPLIT; MAX_CU_CNT_IN_LCU];
                BlockShape::NUM_BLOCK_SHAPE as usize]; NUM_CU_DEPTH],
        }
    }
}
/*****************************************************************************
 * CORE information used for decoding process.
 *
 * The variables in this structure are very often used in decoding process.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCore {
    /************** current CU **************/
    /* coefficient buffer of current CU */
    //coef: [[i16; MAX_CU_DIM]; N_C], //[N_C][MAX_CU_DIM]
    /* pred buffer of current CU */
    /* [1] is used for bi-pred. */
    //pred: [[[pel; MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]

    /* neighbor pixel buffer for intra prediction */
    //nb: [[[pel; MAX_CU_SIZE * 3]; N_REF]; N_C],
    /* reference index for current CU */
    refi: [i8; REFP_NUM],
    /* motion vector for current CU */
    mv: [[i16; MV_D]; REFP_NUM],

    /* CU position in current frame in SCU unit */
    scup: u32,
    /* CU position X in a frame in SCU unit */
    x_scu: u16,
    /* CU position Y in a frame in SCU unit */
    y_scu: u16,
    /* neighbor CUs availability of current CU */
    avail_cu: u16,
    /* Left, right availability of current CU */
    avail_lr: u16,
    /* intra prediction direction of current CU */
    ipm: [u8; 2],
    /* most probable mode for intra prediction */
    mpm_b_list: Vec<u8>,
    mpm: [u8; 2],
    mpm_ext: [u8; 8],
    //pims: [u8; IPD_CNT], /* probable intra mode set*/
    /* prediction mode of current CU: INTRA, INTER, ... */
    pred_mode: PredMode,
    DMVRenable: u8,
    /* log2 of cuw */
    log2_cuw: u8,
    /* log2 of cuh */
    log2_cuh: u8,
    /* is there coefficient? */
    is_coef: [bool; N_C],
    is_coef_sub: [[bool; MAX_SUB_TB_NUM]; N_C],

    /* QP for Luma of current encoding MB */
    qp_y: u8,
    /* QP for Chroma of current encoding MB */
    qp_u: u8,
    qp_v: u8,

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
    /* split mode map for current LCU */
    split_mode: LcuSplitModeArray,

    /* platform specific data, if needed */
    //void          *pf;
    //s16            mmvd_idx;
    //u8             mmvd_flag;

    /* temporal pixel buffer for inter prediction */
    //pel            eif_tmp_buffer[ (MAX_CU_SIZE + 2) * (MAX_CU_SIZE + 2) ];
    mvr_idx: u8,

    mvp_idx: [u8; REFP_NUM],
    mvd: [[i16; MV_D]; REFP_NUM],
    inter_dir: i16,
    bi_idx: i16,
    ctx_flags: [u8; CtxNevIdx::NUM_CNID as usize],
    tree_cons: TREE_CONS,
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
    sbac_ctx: EvcSbacCtx,
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
    map_scu: Vec<MCU>,
    /* LCU split information */
    map_split: Vec<LcuSplitModeArray>,
    /* decoded motion vector for every blocks */
    /*s16                  (* map_mv)[REFP_NUM][MV_D];
    /* decoded motion vector for every blocks */
    s16                  (* map_unrefined_mv)[REFP_NUM][MV_D];
    /* reference frame indices */
    s8                   (* map_refi)[REFP_NUM];
    /* intra prediction modes */
    s8                    * map_ipm;*/
    /* new coding tool flag*/
    map_cu_mode: Vec<u32>,
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
    fn sequence_init(&mut self) {
        if self.sps.pic_width_in_luma_samples != self.w
            || self.sps.pic_height_in_luma_samples != self.h
        {
            /* resolution was changed */
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

        /* alloc SCU map */
        self.map_scu = vec![MCU::default(); self.f_scu as usize];

        /* alloc cu mode SCU map */
        self.map_cu_mode = vec![0; self.f_scu as usize];

        /* alloc map for CU split flag */
        self.map_split = vec![LcuSplitModeArray::default(); self.f_lcu as usize];
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
        /*evc_mset_x64a(self.map_scu, 0, sizeof(u32) * self.f_scu);
        evc_mset_x64a(self.map_affine, 0, sizeof(u32) * self.f_scu);
        evc_mset_x64a(self.map_ats_inter, 0, sizeof(u8) * self.f_scu);
        evc_mset_x64a(self.map_cu_mode, 0, sizeof(u32) * self.f_scu);*/

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

    fn evcd_eco_cu(&mut self) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        core.pred_mode = PredMode::MODE_INTRA;

        let cuw = 1 << core.log2_cuw as u16;
        let cuh = 1 << core.log2_cuh as u16;
        core.avail_lr = evc_check_nev_avail(
            core.x_scu,
            core.y_scu,
            cuw,
            //cuh,
            self.w_scu,
            //self.h_scu,
            &self.map_scu,
        );

        //evc_get_ctx_some_flags

        if !evc_check_only_intra(&core.tree_cons) {
            /* CU skip flag */
            let cu_skip_flag = evcd_eco_cu_skip_flag(bs, sbac, sbac_ctx, &core.ctx_flags)?;
            if cu_skip_flag != 0 {
                core.pred_mode = PredMode::MODE_SKIP;
            }
        }

        /* parse prediction info */
        if core.pred_mode == PredMode::MODE_SKIP {
            core.mvp_idx[REFP_0] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
            if self.sh.slice_type == SliceType::EVC_ST_B {
                core.mvp_idx[REFP_1] = evcd_eco_mvp_idx(bs, sbac, sbac_ctx)?;
            }
        } else {
            core.pred_mode =
                evcd_eco_pred_mode(bs, sbac, sbac_ctx, &core.ctx_flags, &core.tree_cons)?;
        }

        Ok(())
    }

    fn evcd_eco_unit(
        &mut self,
        x: u16,
        y: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        tree_cons: TREE_CONS_NEW,
    ) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;

        core.tree_cons = TREE_CONS {
            changed: false,
            tree_type: tree_cons.tree_type,
            mode_cons: tree_cons.mode_cons,
        };

        core.log2_cuw = log2_cuw;
        core.log2_cuh = log2_cuh;
        core.x_scu = PEL2SCU(x as usize) as u16;
        core.y_scu = PEL2SCU(y as usize) as u16;
        core.scup = core.x_scu as u32 + core.y_scu as u32 * self.w_scu as u32;

        let cuw = 1 << log2_cuw as u16;
        let cuh = 1 << log2_cuh as u16;

        EVC_TRACE_COUNTER(&mut bs.tracer);
        EVC_TRACE(&mut bs.tracer, "poc: ");
        EVC_TRACE(&mut bs.tracer, self.poc.poc_val);
        EVC_TRACE(&mut bs.tracer, " x pos ");
        EVC_TRACE(&mut bs.tracer, x);
        EVC_TRACE(&mut bs.tracer, " y pos ");
        EVC_TRACE(&mut bs.tracer, y);
        EVC_TRACE(&mut bs.tracer, " width ");
        EVC_TRACE(&mut bs.tracer, cuw);
        EVC_TRACE(&mut bs.tracer, " height ");
        EVC_TRACE(&mut bs.tracer, cuh);

        if self.sh.slice_type != SliceType::EVC_ST_I && self.sps.sps_btt_flag {
            EVC_TRACE(&mut bs.tracer, " tree status ");
            EVC_TRACE(&mut bs.tracer, core.tree_cons.tree_type as u8);
            EVC_TRACE(&mut bs.tracer, " mode status ");
            EVC_TRACE(&mut bs.tracer, core.tree_cons.mode_cons as u8);
        }
        EVC_TRACE(&mut bs.tracer, " \n");

        core.avail_lr = evc_check_nev_avail(
            core.x_scu,
            core.y_scu,
            cuw,
            //cuh,
            self.w_scu,
            //self.h_scu,
            &self.map_scu,
        );

        // evc_get_ctx_some_flags

        /* parse CU info */
        //self.evcd_eco_cu()?;

        Ok(())
    }

    fn evcd_eco_tree(
        &mut self,
        x0: u16,
        y0: u16,
        log2_cuw: u8,
        log2_cuh: u8,
        cup: u16,
        cud: u16,
        next_split: bool,
        qt_depth: u8,
        mut cu_qp_delta_code: u8,
        mut mode_cons: MODE_CONS,
    ) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        let cuw = 1 << log2_cuw as u16;
        let cuh = 1 << log2_cuh as u16;
        let mut split_mode = SplitMode::NO_SPLIT;
        if cuw > self.min_cuwh || cuh > self.min_cuwh {
            if x0 + cuw <= self.w && y0 + cuh <= self.h {
                if next_split {
                    split_mode = evcd_eco_split_mode(bs, sbac, sbac_ctx, cuw, cuh)?;
                    EVC_TRACE_COUNTER(&mut bs.tracer);
                    EVC_TRACE(&mut bs.tracer, "x pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.x_pel
                            + (cup % (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " y pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.y_pel
                            + (cup / (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " width ");
                    EVC_TRACE(&mut bs.tracer, cuw);
                    EVC_TRACE(&mut bs.tracer, " height ");
                    EVC_TRACE(&mut bs.tracer, cuh);
                    EVC_TRACE(&mut bs.tracer, " depth ");
                    EVC_TRACE(&mut bs.tracer, cud);
                    EVC_TRACE(&mut bs.tracer, " split mode ");
                    EVC_TRACE(&mut bs.tracer, split_mode as u8);
                    EVC_TRACE(&mut bs.tracer, " \n");
                } else {
                    split_mode = SplitMode::NO_SPLIT;
                }
            } else {
                split_mode = evcd_eco_split_mode(bs, sbac, sbac_ctx, cuw, cuh)?;
                EVC_TRACE_COUNTER(&mut bs.tracer);
                EVC_TRACE(&mut bs.tracer, "x pos ");
                EVC_TRACE(
                    &mut bs.tracer,
                    core.x_pel
                        + (cup % (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " y pos ");
                EVC_TRACE(
                    &mut bs.tracer,
                    core.y_pel
                        + (cup / (self.max_cuwh >> MIN_CU_LOG2 as u16) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " width ");
                EVC_TRACE(&mut bs.tracer, cuw);
                EVC_TRACE(&mut bs.tracer, " height ");
                EVC_TRACE(&mut bs.tracer, cuh);
                EVC_TRACE(&mut bs.tracer, " depth ");
                EVC_TRACE(&mut bs.tracer, cud);
                EVC_TRACE(&mut bs.tracer, " split mode ");
                EVC_TRACE(&mut bs.tracer, split_mode as u8);
                EVC_TRACE(&mut bs.tracer, " \n");
            }
        } else {
            split_mode = SplitMode::NO_SPLIT;
        }

        if self.pps.cu_qp_delta_enabled_flag && self.sps.dquant_flag {
            if split_mode == SplitMode::NO_SPLIT
                && (log2_cuh + log2_cuw >= self.pps.cu_qp_delta_area)
                && cu_qp_delta_code != 2
            {
                if log2_cuh == 7 || log2_cuw == 7 {
                    cu_qp_delta_code = 2;
                } else {
                    cu_qp_delta_code = 1;
                }
                core.cu_qp_delta_is_coded = false;
            }
        }

        evc_set_split_mode(
            &mut core.split_mode,
            split_mode,
            cud,
            cup,
            cuw,
            cuh,
            self.max_cuwh,
        );

        if split_mode != SplitMode::NO_SPLIT {
            let split_struct: EvcSplitStruct = evc_split_get_part_structure(
                split_mode,
                x0,
                y0,
                cuw,
                cuh,
                cup,
                cud,
                self.log2_max_cuwh - MIN_CU_LOG2 as u8,
            );

            for cur_part_num in 0..split_struct.part_count {
                let log2_sub_cuw = split_struct.log_cuw[cur_part_num];
                let log2_sub_cuh = split_struct.log_cuh[cur_part_num];
                let x_pos = split_struct.x_pos[cur_part_num];
                let y_pos = split_struct.y_pos[cur_part_num];

                if x_pos < self.w && y_pos < self.h {
                    self.evcd_eco_tree(
                        x_pos,
                        y_pos,
                        log2_sub_cuw,
                        log2_sub_cuh,
                        split_struct.cup[cur_part_num],
                        split_struct.cud[cur_part_num],
                        true,
                        split_mode.inc_qt_depth(qt_depth),
                        cu_qp_delta_code,
                        mode_cons,
                    )?;
                }
            }
        } else {
            core.cu_qp_delta_code = cu_qp_delta_code;

            let tree_type = if mode_cons == MODE_CONS::eOnlyIntra {
                TREE_TYPE::TREE_L
            } else {
                TREE_TYPE::TREE_LC
            };

            if self.sh.slice_type == SliceType::EVC_ST_I {
                mode_cons = MODE_CONS::eOnlyIntra;
            }

            self.evcd_eco_unit(
                x0,
                y0,
                log2_cuw,
                log2_cuh,
                TREE_CONS_NEW {
                    tree_type,
                    mode_cons,
                },
            )?;
        }

        Ok(())
    }

    fn decode_slice(&mut self) -> Result<(), EvcError> {
        // Initialize CABAC at each tile
        self.sbac_dec.reset(
            &mut self.bs,
            &mut self.sbac_ctx,
            self.sh.slice_type,
            self.sh.qp,
        );

        //TODO: move x_lcu/y_lcu=0 to pic init
        self.core.x_lcu = 0; //entry point lcu's x location
        self.core.y_lcu = 0; // entry point lcu's y location
        while self.num_ctb > 0 {
            self.update_core_loc_param();

            //LCU decoding with in a tile
            evc_assert_rv(
                (self.core.lcu_num as u32) < self.f_lcu,
                EvcError::EVC_ERR_UNEXPECTED,
            )?;

            // invoke coding_tree() recursion
            for i in 0..NUM_CU_DEPTH {
                for j in 0..BlockShape::NUM_BLOCK_SHAPE as usize {
                    for k in 0..MAX_CU_CNT_IN_LCU {
                        self.core.split_mode.array[i][j][k] = SplitMode::NO_SPLIT;
                    }
                }
            }

            self.evcd_eco_tree(
                self.core.x_pel,
                self.core.y_pel,
                self.log2_max_cuwh,
                self.log2_max_cuwh,
                0,
                0,
                true,
                0,
                0,
                MODE_CONS::eAll,
            )?;
            // set split flags to map
            self.map_split[self.core.lcu_num as usize].clone_from(&self.core.split_mode);

            self.num_ctb -= 1;
            // read end_of_picture_flag
            if (self.num_ctb == 0) {
                evcd_eco_tile_end_flag(&mut self.bs, &mut self.sbac_dec)?;
            } else {
                self.core.x_lcu += 1;
                if self.core.x_lcu >= self.w_lcu {
                    self.core.x_lcu = 0;
                    self.core.y_lcu += 1;
                }
            }
        }

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
                //evc_picman_refp_init(&self.dpm, self.sps.max_num_ref_pics, sh->slice_type, self.poc.poc_val, self.nalu.nuh_temporal_id, self.last_intra_poc, self.refp);
            }

            if self.num_ctb == self.f_lcu {
                /* get available frame buffer for decoded image */
                //self.pic = evc_picman_get_empty_pic(&self.dpm)?;

                /* get available frame buffer for decoded image */
                //self.map_refi = self.pic->map_refi;
                //self.map_mv = self.pic->map_mv;
            }

            /* decode slice layer */
            self.decode_slice()?;
        } else if nalu_type == NaluType::EVC_SEI_NUT {
        } else {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }

        Ok(EvcdStat {
            read: nalu_size_field_in_bytes + self.bs.get_read_byte() as usize,
            nalu_type,
            stype: self.sh.slice_type,
            fnum: -1,
            ..Default::default()
        })
    }
}
