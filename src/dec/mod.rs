use super::api::frame::*;
use super::api::*;
use super::def::*;
use super::df::*;
use super::ipred::*;
use super::itdq::*;
use super::mc::*;
use super::picman::*;
use super::recon::*;
use super::tbl::*;
use super::tracer::*;
use super::util::*;

use std::cell::RefCell;
use std::rc::Rc;

mod bsr;
mod eco;
mod sbac;

use bsr::*;
use eco::*;
use sbac::*;

/*****************************************************************************
 * CORE information used for decoding process.
 *
 * The variables in this structure are very often used in decoding process.
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdCore {
    /************** LCU-based processing **************/
    top_mcu: Vec<MCU>,                         //[Width/MIN_CU_SIZE]
    lft_mcu: [MCU; MAX_CU_SIZE / MIN_CU_SIZE], //[MAX_CU_SIZE/MIN_CU_SIZE]
    // intra prediction pixel line buffer
    top_pel: Vec<Vec<pel>>, //[N_C][Width]
    lft_pel: Vec<Vec<pel>>, //[N_C][MAX_CU_SIZE]

    /* coefficient buffer of current CU */
    coef: CUBuffer<i16>, //[[i16; MAX_CU_DIM]; N_C], //[N_C][MAX_CU_DIM]
    /* pred buffer of current CU */
    /* [1] is used for bi-pred. */
    pred: [CUBuffer<pel>; 2], //[[[pel; MAX_CU_DIM]; N_C]; 2], //[2][N_C][MAX_CU_DIM]

    // deblocking line buffer
    //TODO:

    /************** Frame-based processing **************/

    /************** current CU **************/


    /* neighbor pixel buffer for intra prediction */
    nb: NBBuffer<pel>, // [N_C][N_REF][MAX_CU_SIZE * 3];
    /* reference index for current CU */
    refi: [i8; REFP_NUM],
    /* motion vector for current CU */
    mv: [[i16; MV_D]; REFP_NUM],

    /* neighbor CUs availability of current CU */
    avail_cu: u16,
    /* Left, right availability of current CU */
    avail_lr: u16,
    /* intra prediction direction of current CU */
    ipm: [IntraPredDir; 2],
    /* most probable mode for intra prediction */
    mpm_b_list: &'static [u8],
    //mpm: [u8; 2],
    //mpm_ext: [u8; 8],
    //pims: [IntraPredDir; IntraPredDir::IPD_CNT_B as usize], /* probable intra mode set*/
    /* prediction mode of current CU: INTRA, INTER, ... */
    pred_mode: PredMode,

    /* is there coefficient? */
    is_coef: [bool; N_C],

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
    split_mode: LcuSplitMode,

    mvp_idx: [u8; REFP_NUM],
    mvd: [[i16; MV_D]; REFP_NUM],
    inter_dir: InterPredDir,
    ctx_flags: [u8; NUM_CNID],

    evc_tbl_qp_chroma_dynamic_ext: Vec<Vec<i8>>, // [[i8; MAX_QP_TABLE_SIZE_EXT]; 2],
}

/******************************************************************************
 * CONTEXT used for decoding process.
 *
 * All have to be stored are in this structure.
 *****************************************************************************/
pub(crate) struct EvcdCtx {
    /* input packet */
    pkt: Option<Packet>,

    /* CORE information used for fast operation */
    core: EvcdCore,

    /* MAPS *******************************************************************/
    /* SCU map for CU information */
    map_scu: Vec<MCU>,
    /* LCU split information */
    map_split: Vec<LcuSplitMode>,
    /* decoded motion vector for every blocks */
    map_mv: Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    /* reference frame indices */
    map_refi: Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    /* intra prediction modes */
    map_ipm: Vec<IntraPredDir>,
    /* new coding tool flag*/
    map_cu_mode: Vec<MCU>,

    /* *******************************************************************/
    /* current decoding bitstream */
    bs: EvcdBsr,
    /* current nalu header */
    nalu: EvcNalu,
    /* current slice header */
    sh: EvcSh,
    /* decoded picture buffer management */
    dpm: Option<EvcPm>,
    /* create descriptor */
    //EVCD_CDSC               cdsc;
    /* sequence parameter set */
    sps: EvcSps,
    /* picture parameter set */
    pps: EvcPps,
    /* current decoded (decoding) picture buffer */
    pic: Option<Rc<RefCell<EvcPic>>>,
    /* SBAC */
    sbac_dec: EvcdSbac,
    sbac_ctx: EvcSbacCtx,
    /* decoding picture width */
    w: u16,
    /* decoding picture height */
    h: u16,
    /* decoding chroma sampling */
    cs: ChromaSampling,

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
    /* the number of currently decoded pictures */
    pic_cnt: u32,
    /* flag whether current picture is refecened picture or not */
    slice_ref_flag: bool,
    /* distance between ref pics in addition to closest ref ref pic in LD*/
    ref_pic_gap_length: u32,
    /* reference picture (0: foward, 1: backward) */
    refp: Vec<Vec<EvcRefP>>, //[[EvcRefP; REFP_NUM]; MAX_NUM_REF_PICS],
    /* flag to indicate opl decoder output */
    num_ctb: u32,
}

impl EvcdCtx {
    pub(crate) fn new(cfg: &Config) -> Self {
        let mut refp = Vec::with_capacity(MAX_NUM_REF_PICS);
        for j in 0..MAX_NUM_REF_PICS {
            let mut refp1d = Vec::with_capacity(REFP_NUM);
            for i in 0..REFP_NUM {
                refp1d.push(EvcRefP::new());
            }
            refp.push(refp1d);
        }

        EvcdCtx {
            pkt: None,

            /* CORE information used for fast operation */
            core: EvcdCore::default(),
            /* MAPS *******************************************************************/
            /* SCU map for CU information */
            map_scu: vec![],
            /* LCU split information */
            map_split: vec![],
            /* decoded motion vector for every blocks */
            map_mv: None,
            /* reference frame indices */
            map_refi: None,
            /* intra prediction modes */
            map_ipm: vec![],
            /* new coding tool flag*/
            map_cu_mode: vec![],

            /**************************************************************************/
            /* current decoding bitstream */
            bs: EvcdBsr::default(),
            /* current nalu header */
            nalu: EvcNalu::default(),
            /* current slice header */
            sh: EvcSh::default(),
            /* decoded picture buffer management */
            dpm: None,
            /* create descriptor */
            //EVCD_CDSC               cdsc;
            /* sequence parameter set */
            sps: EvcSps::default(),
            /* picture parameter set */
            pps: EvcPps::default(),
            /* current decoded (decoding) picture buffer */
            pic: None,
            /* SBAC */
            sbac_dec: EvcdSbac::default(),
            sbac_ctx: EvcSbacCtx::default(),
            /* decoding picture width */
            w: 0,
            /* decoding picture height */
            h: 0,
            cs: ChromaSampling::Cs400,

            /**************************************************************************/
            /* current slice number, which is increased whenever decoding a slice.
            when receiving a slice for new picture, this value is set to zero.
            this value can be used for distinguishing b/w slices */
            slice_num: 0,
            /* last coded intra picture's picture order count */
            last_intra_poc: 0,
            /* picture width in LCU unit */
            w_lcu: 0,
            /* picture height in LCU unit */
            h_lcu: 0,
            /* picture size in LCU unit (= w_lcu * h_lcu) */
            f_lcu: 0,
            /* picture width in SCU unit */
            w_scu: 0,
            /* picture height in SCU unit */
            h_scu: 0,
            /* picture size in SCU unit (= w_scu * h_scu) */
            f_scu: 0,
            /* the picture order count value */
            poc: EvcPoc::default(),
            /* the number of currently decoded pictures */
            pic_cnt: 0,
            /* flag whether current picture is refecened picture or not */
            slice_ref_flag: false,
            /* distance between ref pics in addition to closest ref ref pic in LD*/
            ref_pic_gap_length: 0,
            /* reference picture (0: foward, 1: backward) */
            refp,
            /* flag to indicate opl decoder output */
            num_ctb: 0,
        }
    }

    fn sequence_init(&mut self) -> Result<(), EvcError> {
        if self.sps.pic_width_in_luma_samples != self.w
            || self.sps.pic_height_in_luma_samples != self.h
        {
            /* resolution was changed */
            self.w = self.sps.pic_width_in_luma_samples;
            self.h = self.sps.pic_height_in_luma_samples;
            self.cs = self.sps.chroma_format_idc.into();
            assert_eq!(self.sps.sps_btt_flag, false);
        }

        self.w_lcu = (self.w + (MAX_CU_SIZE as u16 - 1)) / MAX_CU_SIZE as u16;
        self.h_lcu = (self.h + (MAX_CU_SIZE as u16 - 1)) / MAX_CU_SIZE as u16;
        self.f_lcu = (self.w_lcu * self.h_lcu) as u32;
        self.w_scu = (self.w + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.h_scu = (self.h + ((1 << MIN_CU_LOG2) - 1) as u16) >> MIN_CU_LOG2 as u16;
        self.f_scu = (self.w_scu * self.h_scu) as u32;

        // TOP LINE BUFFERS
        self.core.top_mcu = vec![MCU::default(); self.w_scu as usize];
        self.core.top_pel = vec![
            vec![0; self.w as usize],
            vec![0; (self.w >> 1) as usize],
            vec![0; (self.w >> 1) as usize],
        ];

        /* alloc SCU map */
        self.map_scu = vec![MCU::default(); self.f_scu as usize];

        /* alloc cu mode SCU map */
        self.map_cu_mode = vec![MCU::default(); self.f_scu as usize];

        /* alloc map for CU split flag */
        self.map_split = vec![LcuSplitMode::default(); self.f_lcu as usize];

        /* alloc map for intra prediction mode */
        self.map_ipm = vec![IntraPredDir::default(); self.f_scu as usize];

        /* initialize reference picture manager */
        self.ref_pic_gap_length = (1 << self.sps.log2_ref_pic_gap_length) as u32;

        /* initialize decode picture manager */
        let mut dpm = EvcPm::new(self.w as usize, self.h as usize, self.cs);
        dpm.evc_picman_init(MAX_PB_SIZE as u8, MAX_NUM_REF_PICS as u8)?;
        self.dpm = Some(dpm);

        if self.sps.chroma_qp_table_struct.chroma_qp_table_present_flag {
            self.core.evc_tbl_qp_chroma_dynamic_ext =
                evc_derived_chroma_qp_mapping_tables(&self.sps.chroma_qp_table_struct);
        } else {
            self.core.evc_tbl_qp_chroma_dynamic_ext = vec![];
            self.core
                .evc_tbl_qp_chroma_dynamic_ext
                .push(evc_tbl_qp_chroma_ajudst_base.to_vec());
            self.core
                .evc_tbl_qp_chroma_dynamic_ext
                .push(evc_tbl_qp_chroma_ajudst_base.to_vec());
        }
        Ok(())
    }

    fn slice_init(&mut self) {
        self.core.lcu_num = 0;
        self.core.x_lcu = 0;
        self.core.y_lcu = 0;
        self.core.x_pel = 0;
        self.core.y_pel = 0;
        self.core.qp = self.sh.qp;
        self.core.qp_y = self.sh.qp + (6 * (BIT_DEPTH - 8)) as u8;
        self.core.qp_u = (self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_u as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;
        self.core.qp_v = (self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + self.sh.qp_v as i8) as usize]
            + (6 * (BIT_DEPTH - 8)) as i8) as u8;

        /* clear maps */
        for i in 0..self.f_scu as usize {
            self.map_scu[i] = MCU::default();
            self.map_cu_mode[i] = MCU::default();
        }

        if self.sh.slice_type == SliceType::EVC_ST_I {
            self.last_intra_poc = self.poc.poc_val;
        }
    }

    fn update_core_loc_param(&mut self) {
        self.core.x_pel = self.core.x_lcu << MAX_CU_LOG2 as u16; // entry point's x location in pixel
        self.core.y_pel = self.core.y_lcu << MAX_CU_LOG2 as u16; // entry point's y location in pixel
        self.core.lcu_num = self.core.x_lcu + self.core.y_lcu * self.w_lcu; // Init the first lcu_num in tile
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
    ) -> Result<(), EvcError> {
        let core = &mut self.core;
        let bs = &mut self.bs;
        let sbac = &mut self.sbac_dec;
        let sbac_ctx = &mut self.sbac_ctx;

        let cuw = 1u16 << log2_cuw;
        let cuh = 1u16 << log2_cuh;
        let mut split_mode = SplitMode::NO_SPLIT;
        if cuw > MIN_CU_SIZE as u16 || cuh > MIN_CU_SIZE as u16 {
            if x0 + cuw <= self.w && y0 + cuh <= self.h {
                if next_split {
                    split_mode = evcd_eco_split_mode(bs, sbac, sbac_ctx, cuw, cuh)?;
                    EVC_TRACE_COUNTER(&mut bs.tracer);
                    EVC_TRACE(&mut bs.tracer, "x pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.x_pel
                            + (cup % (MAX_CU_SIZE as u16 >> MIN_CU_LOG2) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " y pos ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        core.y_pel
                            + (cup / (MAX_CU_SIZE as u16 >> MIN_CU_LOG2) << MIN_CU_LOG2 as u16),
                    );
                    EVC_TRACE(&mut bs.tracer, " width ");
                    EVC_TRACE(&mut bs.tracer, cuw);
                    EVC_TRACE(&mut bs.tracer, " height ");
                    EVC_TRACE(&mut bs.tracer, cuh);
                    EVC_TRACE(&mut bs.tracer, " depth ");
                    EVC_TRACE(&mut bs.tracer, cud);
                    EVC_TRACE(&mut bs.tracer, " split mode ");
                    EVC_TRACE(
                        &mut bs.tracer,
                        if split_mode == SplitMode::NO_SPLIT {
                            0
                        } else {
                            5
                        },
                    );
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
                    core.x_pel + (cup % (MAX_CU_SIZE as u16 >> MIN_CU_LOG2) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " y pos ");
                EVC_TRACE(
                    &mut bs.tracer,
                    core.y_pel + (cup / (MAX_CU_SIZE as u16 >> MIN_CU_LOG2) << MIN_CU_LOG2 as u16),
                );
                EVC_TRACE(&mut bs.tracer, " width ");
                EVC_TRACE(&mut bs.tracer, cuw);
                EVC_TRACE(&mut bs.tracer, " height ");
                EVC_TRACE(&mut bs.tracer, cuh);
                EVC_TRACE(&mut bs.tracer, " depth ");
                EVC_TRACE(&mut bs.tracer, cud);
                EVC_TRACE(&mut bs.tracer, " split mode ");
                EVC_TRACE(
                    &mut bs.tracer,
                    if split_mode == SplitMode::NO_SPLIT {
                        0
                    } else {
                        5
                    },
                );
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
            MAX_CU_SIZE as u16,
        );

        if split_mode != SplitMode::NO_SPLIT {
            let split_struct = evc_split_get_part_structure(
                split_mode,
                x0,
                y0,
                cuw,
                cuh,
                cup,
                cud,
                (MAX_CU_LOG2 - MIN_CU_LOG2) as u8,
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
                    )?;
                }
            }
        } else {
            core.cu_qp_delta_code = cu_qp_delta_code;
            evcd_eco_unit(
                bs,
                sbac,
                sbac_ctx,
                core,
                x0,
                y0,
                log2_cuw,
                log2_cuh,
                self.w_scu,
                self.h_scu,
                self.w,
                self.h,
                &self.map_mv,
                &self.refp,
                &self.map_scu,
                &self.map_ipm,
                &self.dpm,
                self.poc.poc_val,
                &self.pic,
                self.sps.dquant_flag,
                self.pps.cu_qp_delta_enabled_flag,
                self.pps.constrained_intra_pred_flag,
                self.sh.slice_type,
                self.sh.qp,
                self.sh.qp_u_offset,
                self.sh.qp_v_offset,
            )?;
            evcd_set_dec_info(
                core,
                x0,
                y0,
                log2_cuw,
                log2_cuh,
                self.w_scu as usize,
                self.pps.cu_qp_delta_enabled_flag,
                self.slice_num,
                &mut self.map_refi,
                &mut self.map_mv,
                &mut self.map_scu,
                &mut self.map_cu_mode,
                &mut self.map_ipm,
            );
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
                        self.core.split_mode.data[i][j][k] = SplitMode::NO_SPLIT;
                    }
                }
            }

            self.evcd_eco_tree(
                self.core.x_pel,
                self.core.y_pel,
                MAX_CU_LOG2 as u8,
                MAX_CU_LOG2 as u8,
                0,
                0,
                true,
                0,
                0,
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
            //eprint!("{} ", self.num_ctb);
        }

        Ok(())
    }

    fn make_stat(&mut self, btype: NaluType) -> EvcStat {
        let mut stat = EvcStat {
            nalu_type: btype,
            stype: SliceType::EVC_ST_I,
            fnum: -1,
            bytes: NALU_SIZE_FIELD_IN_BYTES + self.bs.get_read_byte() as usize,
            ..Default::default()
        };

        if btype < NaluType::EVC_SPS_NUT {
            stat.fnum = self.pic_cnt as isize;
            stat.stype = self.sh.slice_type;

            /* increase decoded picture count */
            self.pic_cnt += 1;
            stat.poc = self.poc.poc_val as isize;
            stat.tid = self.nalu.nuh_temporal_id as isize;

            for i in 0..2 {
                stat.refpic_num[i] = self.dpm.as_ref().unwrap().num_refp[i];
                for j in 0..stat.refpic_num[i] as usize {
                    stat.refpic[i][j] = self.refp[j][i].poc as isize;
                }
            }
        }

        stat
    }

    fn deblock_tree(
        &mut self,
        x: u16,
        y: u16,
        cuw: u16,
        cuh: u16,
        cud: u16,
        cup: u16,
        is_hor_edge: bool,
    ) {
        let lcu_num = (x >> MAX_CU_LOG2) + (y >> MAX_CU_LOG2) * self.w_lcu;
        let split_mode = evc_get_split_mode(
            cud,
            cup,
            cuw,
            cuh,
            MAX_CU_SIZE as u16,
            &self.map_split[lcu_num as usize],
        );

        EVC_TRACE_COUNTER(&mut self.bs.tracer);
        EVC_TRACE(&mut self.bs.tracer, "split_mod ");
        EVC_TRACE(&mut self.bs.tracer, split_mode as u8);
        EVC_TRACE(&mut self.bs.tracer, " \n");

        if split_mode != SplitMode::NO_SPLIT {
            let split_struct = evc_split_get_part_structure(
                split_mode,
                x,
                y,
                cuw,
                cuh,
                cup,
                cud,
                (MAX_CU_LOG2 - MIN_CU_LOG2) as u8,
            );

            // In base profile we have small chroma blocks
            for part_num in 0..split_struct.part_count {
                let cur_part_num = part_num;
                let sub_cuw = split_struct.width[cur_part_num];
                let sub_cuh = split_struct.height[cur_part_num];
                let x_pos = split_struct.x_pos[cur_part_num];
                let y_pos = split_struct.y_pos[cur_part_num];

                if x_pos < self.w && y_pos < self.h {
                    self.deblock_tree(
                        x_pos,
                        y_pos,
                        sub_cuw,
                        sub_cuh,
                        split_struct.cud[cur_part_num],
                        split_struct.cup[cur_part_num],
                        is_hor_edge,
                    );
                }
            }
        } else if let (Some(pic), Some(map_refi), Some(map_mv)) =
            (&self.pic, &self.map_refi, &self.map_mv)
        {
            // deblock
            if is_hor_edge {
                if cuh > MAX_TR_SIZE as u16 {
                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize >> 1,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );

                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize + MAX_TR_SIZE,
                        cuw as usize,
                        cuh as usize >> 1,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );
                } else {
                    evc_deblock_cu_hor(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                    );
                }
            } else {
                if cuw > MAX_TR_SIZE as u16 {
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize >> 1,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize + MAX_TR_SIZE,
                        y as usize,
                        cuw as usize >> 1,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                } else {
                    evc_deblock_cu_ver(
                        &mut self.bs.tracer,
                        &*pic.borrow(),
                        x as usize,
                        y as usize,
                        cuw as usize,
                        cuh as usize,
                        &mut self.map_scu,
                        &*map_refi.borrow(),
                        &*map_mv.borrow(),
                        self.w_scu as usize,
                        &self.core.evc_tbl_qp_chroma_dynamic_ext,
                        self.w as usize,
                    );
                }
            }
        }
    }

    fn evcd_deblock(&mut self) -> Result<(), EvcError> {
        if let Some(pic) = &self.pic {
            let mut p = pic.borrow_mut();
            p.pic_qp_u_offset = self.sh.qp_u_offset;
            p.pic_qp_v_offset = self.sh.qp_v_offset;
        }

        let scu_in_lcu_wh = 1 << (MAX_CU_LOG2 - MIN_CU_LOG2);

        let x_l = 0; //entry point lcu's x location
        let y_l = 0; // entry point lcu's y location
        let x_r = x_l + self.w_lcu;
        let y_r = y_l + self.h_lcu;
        let l_scu = x_l * scu_in_lcu_wh;
        let r_scu = EVC_CLIP3(0, self.w_scu, x_r * scu_in_lcu_wh);
        let t_scu = y_l * scu_in_lcu_wh;
        let b_scu = EVC_CLIP3(0, self.h_scu, y_r * scu_in_lcu_wh);

        for j in t_scu..b_scu {
            for i in l_scu..r_scu {
                self.map_scu[(i + j * self.w_scu) as usize].CLR_COD();
            }
        }

        /* horizontal filtering */
        for j in y_l..y_r {
            for i in x_l..x_r {
                self.deblock_tree(
                    (i << MAX_CU_LOG2),
                    (j << MAX_CU_LOG2),
                    MAX_CU_SIZE as u16,
                    MAX_CU_SIZE as u16,
                    0,
                    0,
                    false, /*horizontal filtering of vertical edge*/
                );
            }
        }

        for j in t_scu..b_scu {
            for i in l_scu..r_scu {
                self.map_scu[(i + j * self.w_scu) as usize].CLR_COD();
            }
        }

        /* vertical filtering */
        for j in y_l..y_r {
            for i in x_l..x_r {
                self.deblock_tree(
                    (i << MAX_CU_LOG2),
                    (j << MAX_CU_LOG2),
                    MAX_CU_SIZE as u16,
                    MAX_CU_SIZE as u16,
                    0,
                    0,
                    true, /*vertical filtering of horizontal edge*/
                );
            }
        }

        Ok(())
    }

    pub(crate) fn push_pkt(&mut self, pkt: &mut Option<Packet>) -> Result<(), EvcError> {
        self.pkt = pkt.take();
        Ok(())
    }

    pub(crate) fn decode_nalu(&mut self) -> Result<EvcStat, EvcError> {
        if self.pkt.is_none() {
            return Err(EvcError::EVC_OK_FLUSH);
        }

        let pkt = self.pkt.take().ok_or(EvcError::EVC_ERR_EMPTY_PACKET)?;

        /* bitstream reader initialization */
        self.bs = EvcdBsr::new(pkt);

        /* parse nalu header */
        evcd_eco_nalu(&mut self.bs, &mut self.nalu)?;

        let nalu_type = self.nalu.nal_unit_type;
        if nalu_type == NaluType::EVC_SPS_NUT {
            evcd_eco_sps(&mut self.bs, &mut self.sps)?;

            self.sequence_init()?;
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

            /* initialize reference pictures */
            self.dpm.as_mut().unwrap().evc_picman_refp_init(
                self.sps.max_num_ref_pics,
                self.sh.slice_type,
                self.poc.poc_val as u32,
                self.nalu.nuh_temporal_id,
                self.last_intra_poc,
                &mut self.refp,
            );

            if self.num_ctb == self.f_lcu {
                /* get available frame buffer for decoded image */
                self.pic = self.dpm.as_mut().unwrap().evc_picman_get_empty_pic()?;

                /* get available frame buffer for decoded image */
                if let Some(pic) = &self.pic {
                    let p = pic.borrow();
                    self.map_refi = Some(Rc::clone(&p.map_refi));
                    self.map_mv = Some(Rc::clone(&p.map_mv));
                }
            }

            /* decode slice layer */
            self.decode_slice()?;

            /* deblocking filter */
            if self.sh.deblocking_filter_on {
                self.evcd_deblock()?;
            }

            if self.num_ctb == 0 {
                /* expand pixels to padding area */
                if let Some(pic) = &self.pic {
                    let frame = &pic.borrow().frame;
                    frame.borrow_mut().pad();
                }

                /* put decoded picture to DPB */
                self.dpm.as_mut().unwrap().evc_picman_put_pic(
                    &self.pic,
                    self.nalu.nal_unit_type == NaluType::EVC_IDR_NUT,
                    self.poc.poc_val as u32,
                    self.nalu.nuh_temporal_id,
                    true,
                    &mut self.refp,
                    self.slice_ref_flag,
                    self.ref_pic_gap_length,
                );
            }
        } else if nalu_type == NaluType::EVC_SEI_NUT {
        } else {
            return Err(EvcError::EVC_ERR_MALFORMED_BITSTREAM);
        }

        let mut stat = self.make_stat(nalu_type);
        if self.num_ctb > 0 {
            stat.fnum = -1;
        }

        Ok(stat)
    }

    pub(crate) fn pull_frm(&mut self) -> Result<Rc<RefCell<Frame<pel>>>, EvcError> {
        let pic = self.dpm.as_mut().unwrap().evc_picman_out_pic()?;
        if let Some(p) = &pic {
            Ok(Rc::clone(&p.borrow().frame))
        } else {
            Err(EvcError::EVC_OK_OUTPUT_NOT_AVAILABLE)
        }
    }
}
