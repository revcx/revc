use super::def::*;
use super::picman::*;
use super::tbl::*;
use crate::api::*;

use std::ops::Neg;

/* clipping within min and max */
#[inline]
pub(crate) fn EVC_CLIP3<T: PartialOrd>(min_x: T, max_x: T, value: T) -> T {
    if value < min_x {
        min_x
    } else if value > max_x {
        max_x
    } else {
        value
    }
}

#[inline]
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
    let poc_offset: i32 = (sub_gop_length as f32
        * ((2.0 * doc_offset as f32 + 1.0) / (1 << tid as i32) as f32 - 2.0))
        as i32;
    poc.poc_val = poc.prev_poc_val as i32 + poc_offset;
    poc.prev_doc_offset = doc_offset;
}

pub(crate) fn evc_set_split_mode(
    split_mode_buf: &mut LcuSplitMode,
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
        split_mode_buf.data[cud as usize][shape as usize][pos as usize] = split_mode;
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

    if y_scu > 0 && x_scu + scuw < w_scu && map_scu[(scup + scuw) as usize].GET_COD() != 0 {
        avail_lr += 2;
    }

    return avail_lr;
}

pub(crate) fn evc_get_avail_inter(
    x_scu: usize,
    y_scu: usize,
    w_scu: usize,
    h_scu: usize,
    scup: usize,
    cuw: usize,
    cuh: usize,
    map_scu: &[MCU],
) -> u16 {
    let mut avail = 0;
    let scuw = cuw >> MIN_CU_LOG2;
    let scuh = cuh >> MIN_CU_LOG2;
    let curr_scup = x_scu + y_scu * w_scu;

    if x_scu > 0 && map_scu[scup - 1].GET_IF() == 0 && map_scu[scup - 1].GET_COD() != 0 {
        SET_AVAIL(&mut avail, AVAIL_LE);

        if y_scu + scuh < h_scu
            && map_scu[scup + (scuh * w_scu) - 1].GET_COD() != 0
            && map_scu[scup + (scuh * w_scu) - 1].GET_IF() == 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_LE);
        }
    }

    if y_scu > 0 {
        if map_scu[scup - w_scu].GET_IF() == 0 {
            SET_AVAIL(&mut avail, AVAIL_UP);
        }

        if map_scu[scup - w_scu + scuw - 1].GET_IF() == 0 {
            SET_AVAIL(&mut avail, AVAIL_RI_UP);
        }

        if x_scu > 0
            && map_scu[scup - w_scu - 1].GET_IF() == 0
            && map_scu[scup - w_scu - 1].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_UP_LE);
        }
        // xxu check??
        if x_scu + scuw < w_scu
            && map_scu[scup - w_scu + scuw].IS_COD_NIF()
            && map_scu[scup - w_scu + scuw].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_UP_RI);
        }
    }

    if x_scu + scuw < w_scu
        && map_scu[scup + scuw].GET_IF() == 0
        && map_scu[scup + scuw].GET_COD() != 0
    {
        SET_AVAIL(&mut avail, AVAIL_RI);

        if y_scu + scuh < h_scu
            && map_scu[scup + (scuh * w_scu) + scuw].GET_COD() != 0
            && map_scu[scup + (scuh * w_scu) + scuw].GET_IF() == 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_RI);
        }
    }

    return avail;
}

pub(crate) fn evc_get_avail_intra(
    x_scu: usize,
    y_scu: usize,
    w_scu: usize,
    h_scu: usize,
    scup: usize,
    log2_cuw: u8,
    log2_cuh: u8,
    map_scu: &[MCU],
) -> u16 {
    let mut avail = 0;

    let log2_scuw = log2_cuw as usize - MIN_CU_LOG2;
    let log2_scuh = log2_cuh as usize - MIN_CU_LOG2;
    let scuw = 1 << log2_scuw;
    let scuh = 1 << log2_scuh;
    let curr_scup = x_scu + y_scu * w_scu;

    if x_scu > 0 && map_scu[(scup - 1) as usize].GET_COD() != 0 {
        SET_AVAIL(&mut avail, AVAIL_LE);

        if y_scu + scuh + scuw - 1 < h_scu
            && map_scu[(scup + (w_scu * (scuw + scuh)) - w_scu - 1) as usize].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_LE);
        }
    }

    if y_scu > 0 {
        SET_AVAIL(&mut avail, AVAIL_UP);
        SET_AVAIL(&mut avail, AVAIL_RI_UP);

        if x_scu > 0 && map_scu[(scup - w_scu - 1) as usize].GET_COD() != 0 {
            SET_AVAIL(&mut avail, AVAIL_UP_LE);
        }

        if x_scu + scuw < w_scu && map_scu[(scup - w_scu + scuw) as usize].GET_COD() != 0 {
            SET_AVAIL(&mut avail, AVAIL_UP_RI);
        }
    }

    if x_scu + scuw < w_scu && map_scu[(scup + scuw) as usize].GET_COD() != 0 {
        SET_AVAIL(&mut avail, AVAIL_RI);

        if y_scu + scuh + scuw - 1 < h_scu
            && map_scu[(scup + (w_scu * (scuw + scuh - 1)) + scuw) as usize].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_RI);
        }
    }

    return avail;
}

#[inline]
pub(crate) fn evc_check_only_intra(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eOnlyIntra
}

#[inline]
pub(crate) fn evc_check_only_inter(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eOnlyInter
}

#[inline]
pub(crate) fn evc_check_all_preds(tree_cons: &TREE_CONS) -> bool {
    tree_cons.mode_cons == MODE_CONS::eAll
}

#[inline]
pub(crate) fn evc_check_luma(tree_cons: &TREE_CONS) -> bool {
    tree_cons.tree_type != TREE_TYPE::TREE_C
}

#[inline]
pub(crate) fn evc_check_chroma(tree_cons: &TREE_CONS) -> bool {
    tree_cons.tree_type != TREE_TYPE::TREE_L
}

pub(crate) fn evc_block_copy(
    src: &[i16],
    src_stride: usize,
    dst: &mut [i16],
    dst_stride: usize,
    log2_copy_w: u8,
    log2_copy_h: u8,
) {
    for h in 0..(1 << log2_copy_h as usize) {
        for w in 0..(1 << log2_copy_w as usize) {
            dst[h * dst_stride + w] = src[h * src_stride + w];
        }
    }
}

#[inline]
pub(crate) fn check_bi_applicability(slice_type: SliceType) -> bool {
    if slice_type == SliceType::EVC_ST_B {
        true
    } else {
        false
    }
}

pub(crate) fn scan_tbl(size: i16) -> Box<[u16]> {
    let mut pos = 0;
    let num_line = size + size - 1;
    let mut scan = vec![0; (size * size) as usize].into_boxed_slice();
    /* starting point */
    scan[pos] = 0;
    pos += 1;

    /* loop */
    for l in 1..num_line {
        if l % 2 != 0 {
            /* decreasing loop */
            let mut x = std::cmp::min(l, size - 1);
            let mut y = std::cmp::max(0, l - (size - 1));

            while x >= 0 && y < size {
                scan[pos] = (y * size + x) as u16;
                pos += 1;
                x -= 1;
                y += 1;
            }
        } else
        /* increasing loop */
        {
            let mut y = std::cmp::min(l, size - 1);
            let mut x = std::cmp::max(0, l - (size - 1));
            while y >= 0 && x < size {
                scan[pos] = (y * size + x) as u16;
                pos += 1;
                x += 1;
                y -= 1;
            }
        }
    }

    scan
}

pub(crate) fn evc_init_multi_tbl(c: usize) -> Box<[i16]> {
    let mut tm = vec![0i16; c * c].into_boxed_slice();
    let s = (c as f64).sqrt() * 64.0;

    for k in 0..c {
        for n in 0..c {
            /* DCT-VIII */
            let a = std::f64::consts::PI * (k as f64 + 0.5) * (n as f64 + 0.5) / (c as f64 + 0.5);
            let b = 2.0 / (c as f64 + 0.5);
            let v = a.cos() * b.sqrt();
            tm[k * c + n] = (s * v + if v > 0.0 { 0.5 } else { -0.5 }) as i16;
        }
    }

    tm
}

pub(crate) fn evc_init_multi_inv_tbl(c: usize) -> Box<[i16]> {
    let mut tm = vec![0i16; c * c].into_boxed_slice();
    let s = (c as f64).sqrt() * 64.0;

    for k in 0..c {
        for n in 0..c {
            /* DCT-VIII */
            let a = std::f64::consts::PI * (k as f64 + 0.5) * (n as f64 + 0.5) / (c as f64 + 0.5);
            let b = 2.0 / (c as f64 + 0.5);
            let v = a.cos() * b.sqrt();
            tm[n * c + k] = (s * v + if v > 0.0 { 0.5 } else { -0.5 }) as i16;
        }
    }

    tm
}

pub(crate) fn evc_get_motion(
    scup: usize,
    lidx: usize,
    map_mv: &Vec<[[i16; MV_D]; REFP_NUM]>,
    refp: &Vec<Vec<EvcRefP>>,
    cuw: usize,
    cuh: usize,
    w_scu: usize,
    avail: u16,
    refi: &mut [i8; MAX_NUM_MVP],
    mvp: &mut [[i16; MV_D]; MAX_NUM_MVP],
) {
    if IS_AVAIL(avail, AVAIL_LE) {
        refi[0] = 0;
        mvp[0][MV_X] = map_mv[scup - 1][lidx][MV_X];
        mvp[0][MV_Y] = map_mv[scup - 1][lidx][MV_Y];
    } else {
        refi[0] = 0;
        mvp[0][MV_X] = 1;
        mvp[0][MV_Y] = 1;
    }

    if IS_AVAIL(avail, AVAIL_UP) {
        refi[1] = 0;
        mvp[1][MV_X] = map_mv[scup - w_scu][lidx][MV_X];
        mvp[1][MV_Y] = map_mv[scup - w_scu][lidx][MV_Y];
    } else {
        refi[1] = 0;
        mvp[1][MV_X] = 1;
        mvp[1][MV_Y] = 1;
    }

    if IS_AVAIL(avail, AVAIL_UP_RI) {
        refi[2] = 0;
        mvp[2][MV_X] = map_mv[scup - w_scu + (cuw >> MIN_CU_LOG2)][lidx][MV_X];
        mvp[2][MV_Y] = map_mv[scup - w_scu + (cuw >> MIN_CU_LOG2)][lidx][MV_Y];
    } else {
        refi[2] = 0;
        mvp[2][MV_X] = 1;
        mvp[2][MV_Y] = 1;
    }
    refi[3] = 0;

    if let Some(map_mv) = &refp[0][lidx].map_mv {
        let mv = map_mv.borrow();
        mvp[3][MV_X] = mv[scup][0][MV_X];
        mvp[3][MV_Y] = mv[scup][0][MV_Y];
    }
}

pub(crate) fn evc_get_mv_dir(
    refp: &[EvcRefP],
    poc: i32,
    scup: usize,
    c_scu: usize,
    w_scu: u16,
    h_scu: u16,
    mvp: &mut [[i16; MV_D]; REFP_NUM],
) {
    let mut mvc = [0i16; MV_D];

    if let Some(map_mv) = &refp[REFP_1].map_mv {
        let mv = map_mv.borrow();
        mvc[MV_X] = mv[scup][0][MV_X];
        mvc[MV_Y] = mv[scup][0][MV_Y];
    }

    let dpoc_co = refp[REFP_1].poc as i32 - refp[REFP_1].list_poc[0] as i32;
    let dpoc_L0 = poc - refp[REFP_0].poc as i32;
    let dpoc_L1 = refp[REFP_1].poc as i32 - poc;

    if dpoc_co == 0 {
        mvp[REFP_0][MV_X] = 0;
        mvp[REFP_0][MV_Y] = 0;
        mvp[REFP_1][MV_X] = 0;
        mvp[REFP_1][MV_Y] = 0;
    } else {
        mvp[REFP_0][MV_X] = (dpoc_L0 * mvc[MV_X] as i32 / dpoc_co) as i16;
        mvp[REFP_0][MV_Y] = (dpoc_L0 * mvc[MV_Y] as i32 / dpoc_co) as i16;
        mvp[REFP_1][MV_X] = (-dpoc_L1 * mvc[MV_X] as i32 / dpoc_co) as i16;
        mvp[REFP_1][MV_Y] = (-dpoc_L1 * mvc[MV_Y] as i32 / dpoc_co) as i16;
    }
}

pub(crate) fn evc_derived_chroma_qp_mapping_tables(
    structChromaQP: &EvcChromaTable,
) -> Vec<Vec<i8>> {
    let MAX_QP = MAX_QP_TABLE_SIZE as i8 - 1;
    let mut qpInVal = [0i8; MAX_QP_TABLE_SIZE_EXT];
    let mut qpOutVal = [0i8; MAX_QP_TABLE_SIZE_EXT];
    let mut p_evc_tbl_qp_chroma_dynamic = Vec::with_capacity(2);
    p_evc_tbl_qp_chroma_dynamic.push(vec![0; MAX_QP_TABLE_SIZE_EXT]);
    p_evc_tbl_qp_chroma_dynamic.push(vec![0; MAX_QP_TABLE_SIZE_EXT]);

    let startQp = if structChromaQP.global_offset_flag {
        16
    } else {
        -EVC_TBL_CHROMA_QP_OFFSET
    };

    for i in 0..if structChromaQP.same_qp_table_for_chroma {
        1
    } else {
        2
    } {
        qpInVal[0] = startQp + structChromaQP.delta_qp_in_val_minus1[i][0];
        qpOutVal[0] = startQp
            + structChromaQP.delta_qp_in_val_minus1[i][0]
            + structChromaQP.delta_qp_out_val[i][0];
        for j in 1..=structChromaQP.num_points_in_qp_table_minus1[i] {
            qpInVal[j] = qpInVal[j - 1] + structChromaQP.delta_qp_in_val_minus1[i][j] + 1;
            qpOutVal[j] = qpOutVal[j - 1]
                + (structChromaQP.delta_qp_in_val_minus1[i][j]
                    + 1
                    + structChromaQP.delta_qp_out_val[i][j]);
        }

        for j in 0..=structChromaQP.num_points_in_qp_table_minus1[i] {
            assert!(qpInVal[j] >= -EVC_TBL_CHROMA_QP_OFFSET && qpInVal[j] <= MAX_QP);
            assert!(qpOutVal[j] >= -EVC_TBL_CHROMA_QP_OFFSET && qpOutVal[j] <= MAX_QP);
        }

        p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + qpInVal[0]) as usize] =
            qpOutVal[0];
        let mut k = qpInVal[0] - 1;
        while k >= -EVC_TBL_CHROMA_QP_OFFSET {
            p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + k) as usize] = EVC_CLIP3(
                -EVC_TBL_CHROMA_QP_OFFSET,
                MAX_QP,
                p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + k + 1) as usize] - 1,
            );
            k -= 1;
        }
        for j in 0..structChromaQP.num_points_in_qp_table_minus1[i] {
            let sh = (structChromaQP.delta_qp_in_val_minus1[i][j + 1] + 1) >> 1;
            let mut m = 1;
            for k in qpInVal[j] + 1..=qpInVal[j + 1] {
                p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + k) as usize] =
                    p_evc_tbl_qp_chroma_dynamic[i]
                        [(EVC_TBL_CHROMA_QP_OFFSET + qpInVal[j]) as usize]
                        + ((qpOutVal[j + 1] - qpOutVal[j]) * m + sh)
                            / (structChromaQP.delta_qp_in_val_minus1[i][j + 1] + 1);
                m += 1;
            }
        }
        for k in qpInVal[structChromaQP.num_points_in_qp_table_minus1[i]] + 1..=MAX_QP {
            p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + k) as usize] = EVC_CLIP3(
                -EVC_TBL_CHROMA_QP_OFFSET,
                MAX_QP,
                p_evc_tbl_qp_chroma_dynamic[i][(EVC_TBL_CHROMA_QP_OFFSET + k - 1) as usize] + 1,
            );
        }
    }
    if structChromaQP.same_qp_table_for_chroma {
        let (p0, p1) = p_evc_tbl_qp_chroma_dynamic.split_at_mut(1);
        p1[0].copy_from_slice(&p0[0]);
    }

    p_evc_tbl_qp_chroma_dynamic
}

pub(crate) fn evc_get_split_mode(
    cud: u16,
    cup: u16,
    cuw: u16,
    cuh: u16,
    lcu_s: u16,
    split_mode_buf: &LcuSplitMode,
) -> SplitMode {
    if cuw < 8 && cuh < 8 {
        SplitMode::NO_SPLIT
    } else {
        let pos = cup
            + (((cuh >> 1) >> MIN_CU_LOG2) * (lcu_s >> MIN_CU_LOG2) + ((cuw >> 1) >> MIN_CU_LOG2));
        let shape = (BlockShape::SQUARE as i8 + CONV_LOG2(cuw as usize) as i8
            - CONV_LOG2(cuh as usize) as i8) as usize;
        split_mode_buf.data[cud as usize][shape][pos as usize]
    }
}

pub(crate) fn evc_check_split_mode(split_allow: &mut [bool]) {
    split_allow[SplitMode::NO_SPLIT as usize] = false;
    split_allow[SplitMode::SPLIT_QUAD as usize] = true;
}

pub(crate) fn evc_get_default_tree_cons() -> TREE_CONS {
    TREE_CONS {
        changed: false,
        tree_type: TREE_TYPE::TREE_LC,
        mode_cons: MODE_CONS::eAll,
    }
}

pub(crate) fn evc_get_avail_block(
    x_scu: u16,
    y_scu: u16,
    w_scu: u16,
    h_scu: u16,
    scup: u32,
    log2_cuw: u8,
    log2_cuh: u8,
    map_scu: &[MCU],
) -> u16 {
    let mut avail = 0;

    let log2_scuw = log2_cuw - MIN_CU_LOG2 as u8;
    let log2_scuh = log2_cuh - MIN_CU_LOG2 as u8;
    let scuw = 1 << log2_scuw;
    let scuh = 1 << log2_scuh;

    if x_scu > 0 && map_scu[(scup - 1) as usize].GET_COD() != 0 {
        SET_AVAIL(&mut avail, AVAIL_LE);
        if y_scu + scuh < h_scu
            && map_scu[(scup + (scuh * w_scu) as u32 - 1) as usize].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_LE);
        }
    }

    if y_scu > 0 {
        SET_AVAIL(&mut avail, AVAIL_UP);
        SET_AVAIL(&mut avail, AVAIL_RI_UP);

        if x_scu > 0 && map_scu[(scup - w_scu as u32 - 1) as usize].GET_COD() != 0 {
            SET_AVAIL(&mut avail, AVAIL_UP_LE);
        }
        if x_scu + scuw < w_scu
            && map_scu[(scup - w_scu as u32 + scuw as u32) as usize].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_UP_RI);
        }
    }

    if x_scu + scuw < w_scu && map_scu[(scup + scuw as u32) as usize].GET_COD() != 0 {
        SET_AVAIL(&mut avail, AVAIL_RI);

        if y_scu + scuh < h_scu
            && map_scu[(scup + (w_scu * scuh) as u32 + scuw as u32) as usize].GET_COD() != 0
        {
            SET_AVAIL(&mut avail, AVAIL_LO_RI);
        }
    }

    avail
}
