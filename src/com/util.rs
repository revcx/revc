use super::tbl::*;
use super::*;

use crate::dec::LcuSplitModeArray;
use std::cmp::*;

/* clipping within min and max */
pub(crate) fn EVC_CLIP3<T: Ord>(min_x: T, max_x: T, value: T) -> T {
    max(min_x, min(max_x, value))
}

pub(crate) fn CONV_LOG2(v: usize) -> u8 {
    evc_tbl_log2[v]
}

pub(crate) fn evc_poc_derivation(sps: &EvcSps, tid: u8, poc: &mut EvcPoc) {
    let sub_gop_length: i32 = (1 << sps.log2_sub_gop_length) as i32;
    let mut expected_tid = 0;

    if tid == 0 {
        poc.poc_val = poc.prev_poc_val as i32 + sub_gop_length;
        poc.prev_doc_offset = 0;
        poc.prev_poc_val = poc.poc_val as u32;
        return;
    }
    let mut doc_offset = (poc.prev_doc_offset + 1) % sub_gop_length;
    if doc_offset == 0 {
        poc.prev_poc_val += sub_gop_length as u32;
    } else {
        expected_tid = 1 + (doc_offset as f32).log2() as u8;
    }
    while tid != expected_tid {
        doc_offset = (doc_offset + 1) % sub_gop_length as i32;
        if doc_offset == 0 {
            expected_tid = 0;
        } else {
            expected_tid = 1 + (doc_offset as f32).log2() as u8;
        }
    }
    let poc_offset: i32 = sub_gop_length * ((2 * doc_offset + 1) / (1 << tid as i32) - 2);
    poc.poc_val = poc.prev_poc_val as i32 + poc_offset;
    poc.prev_doc_offset = doc_offset;
}

pub(crate) fn evc_set_split_mode(
    split_mode_buf: &mut LcuSplitModeArray,
    split_mode: SplitMode,
    cud: u16,
    cup: u16,
    cuw: u16,
    cuh: u16,
    lcu_s: u16,
) {
    let pos = cup
        + (((cuh >> 1) >> MIN_CU_LOG2 as u16) * (lcu_s >> MIN_CU_LOG2 as u16)
            + ((cuw >> 1) >> MIN_CU_LOG2 as u16));
    let shape = BlockShape::SQUARE as u8 + (CONV_LOG2(cuw as usize) - CONV_LOG2(cuh as usize));

    if cuw >= 8 || cuh >= 8 {
        split_mode_buf.array[cud as usize][shape as usize][pos as usize] = split_mode;
    }
}

pub(crate) const SPLIT_MAX_PART_COUNT: usize = 4;

#[derive(Default)]
pub(crate) struct EvcSplitStruct {
    pub(crate) part_count: usize,
    pub(crate) cud: [u16; SPLIT_MAX_PART_COUNT],
    pub(crate) width: [u16; SPLIT_MAX_PART_COUNT],
    pub(crate) height: [u16; SPLIT_MAX_PART_COUNT],
    pub(crate) log_cuw: [u8; SPLIT_MAX_PART_COUNT],
    pub(crate) log_cuh: [u8; SPLIT_MAX_PART_COUNT],
    pub(crate) x_pos: [u16; SPLIT_MAX_PART_COUNT],
    pub(crate) y_pos: [u16; SPLIT_MAX_PART_COUNT],
    pub(crate) cup: [u16; SPLIT_MAX_PART_COUNT],
    //tree_cons: TREE_CONS,
}

pub(crate) fn evc_split_get_part_structure(
    split_mode: SplitMode,
    x0: u16,
    y0: u16,
    cuw: u16,
    cuh: u16,
    cup: u16,
    cud: u16,
    log2_culine: u8,
) -> EvcSplitStruct {
    let mut split_struct = EvcSplitStruct::default();

    split_struct.part_count = split_mode.part_count();
    let log_cuw = CONV_LOG2(cuw as usize);
    let log_cuh = CONV_LOG2(cuh as usize);
    split_struct.x_pos[0] = x0;
    split_struct.y_pos[0] = y0;
    split_struct.cup[0] = cup;

    if split_mode == SplitMode::NO_SPLIT {
        split_struct.width[0] = cuw;
        split_struct.height[0] = cuh;
        split_struct.log_cuw[0] = log_cuw;
        split_struct.log_cuh[0] = log_cuh;
    } else {
        split_struct.width[0] = cuw >> 1;
        split_struct.height[0] = cuh >> 1;
        split_struct.log_cuw[0] = log_cuw - 1;
        split_struct.log_cuh[0] = log_cuh - 1;
        for i in 1..split_struct.part_count {
            split_struct.width[i] = split_struct.width[0];
            split_struct.height[i] = split_struct.height[0];
            split_struct.log_cuw[i] = split_struct.log_cuw[0];
            split_struct.log_cuh[i] = split_struct.log_cuh[0];
        }
        split_struct.x_pos[1] = x0 + split_struct.width[0];
        split_struct.y_pos[1] = y0;
        split_struct.x_pos[2] = x0;
        split_struct.y_pos[2] = y0 + split_struct.height[0];
        split_struct.x_pos[3] = split_struct.x_pos[1];
        split_struct.y_pos[3] = split_struct.y_pos[2];
        let cup_w = (split_struct.width[0] >> MIN_CU_LOG2 as u16);
        let cup_h = ((split_struct.height[0] >> MIN_CU_LOG2 as u16) << log2_culine as u16);
        split_struct.cup[1] = cup + cup_w;
        split_struct.cup[2] = cup + cup_h;
        split_struct.cup[3] = split_struct.cup[1] + cup_h;
        split_struct.cud[0] = cud + 2;
        split_struct.cud[1] = cud + 2;
        split_struct.cud[2] = cud + 2;
        split_struct.cud[3] = cud + 2;
    }

    split_struct
}

pub(crate) fn evc_check_nev_avail(
    x_scu: u16,
    y_scu: u16,
    cuw: u16,
    //cuh: u16,
    w_scu: u16,
    //h_scu: u16,
    map_scu: &[MCU],
) -> u16 {
    let scup = y_scu * w_scu + x_scu;
    let scuw = cuw >> MIN_CU_LOG2 as u16;
    let mut avail_lr = 0;
    //let curr_scup = x_scu + y_scu * w_scu;

    if x_scu > 0 && map_scu[scup as usize - 1].GET_COD() != 0 {
        avail_lr += 1;
    }

    if x_scu + scuw < w_scu && map_scu[(scup + scuw) as usize].GET_COD() != 0 {
        avail_lr += 2;
    }

    return avail_lr;
}

pub(crate) fn evc_check_only_intra(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eOnlyIntra
}

pub(crate) fn evc_check_only_inter(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eOnlyInter
}

pub(crate) fn evc_check_all_preds(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eAll
}
