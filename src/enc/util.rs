use super::sad::*;
use super::*;
use crate::def::*;
use crate::plane::*;
use crate::region::*;

lazy_static! {
    pub(crate) static ref entropy_bits: Box<[i32]> = {
        let mut bits = vec![0; 1024].into_boxed_slice();
        for i in 0..1024 {
            let p = (512.0 * (i as f64 + 0.5)) / 1024.0;
            bits[i] = (-32768.0 * (p.log10() / (2.0f64).log10() - 9.0)) as i32;
        }
        bits
    };
}

pub(crate) fn biari_no_bits(symbol: usize, cm: SBAC_CTX_MODEL) -> i32 {
    let mps = cm & 1;
    let mut state = cm >> 1;
    let sym = if symbol != 0 { 1 } else { 0 };
    state = if sym != mps { state } else { 512 - state };

    entropy_bits[(state as usize) << 1]
}

impl EvceCtx {
    pub(crate) fn evce_set_qp(&mut self, qp: u8) {
        self.core.qp = qp;
        self.core.qp_y = GET_LUMA_QP(self.core.qp as i8) as u8;
        let qp_i_cb = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_u_offset,
        );
        let qp_i_cr = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_v_offset,
        );
        self.core.qp_u = (self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;
        self.core.qp_v = (self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;
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

pub(crate) fn evc_get_luma_cup(
    x_scu: u16,
    y_scu: u16,
    cu_w_scu: u16,
    cu_h_scu: u16,
    w_scu: u16,
) -> u16 {
    (y_scu + (cu_h_scu >> 1)) * w_scu + x_scu + (cu_w_scu >> 1)
}

pub(crate) enum TQC_RUN {
    RUN_L = 1,
    RUN_CB = 2,
    RUN_CR = 4,
}

pub(crate) fn evc_get_run(run_list: u8, tree_cons: &TREE_CONS) -> u8 {
    let mut ans = 0;
    if evc_check_luma(tree_cons) {
        ans |= run_list & TQC_RUN::RUN_L as u8;
    }

    if evc_check_chroma(tree_cons) {
        ans |= run_list & TQC_RUN::RUN_CB as u8;
        ans |= run_list & TQC_RUN::RUN_CR as u8;
    }
    return ans;
}

pub(crate) fn evce_diff_pred(
    mut x: usize,
    mut y: usize,
    mut log2_cuw: usize,
    mut log2_cuh: usize,
    planes: &[Plane<pel>],
    pred: &CUBuffer<pel>,
    diff: &mut CUBuffer<i16>,
) {
    let mut cuw = 1 << log2_cuw;
    let mut cuh = 1 << log2_cuh;

    /* Y */
    evce_diff_16b(
        x,
        y,
        log2_cuw,
        log2_cuh,
        &planes[Y_C].as_region(),
        &pred.data[Y_C],
        &mut diff.data[Y_C],
    );

    cuw >>= 1;
    cuh >>= 1;
    x >>= 1;
    y >>= 1;
    log2_cuw -= 1;
    log2_cuh -= 1;

    /* U */
    let buf = &planes[U_C].as_region();
    evce_diff_16b(
        x,
        y,
        log2_cuw,
        log2_cuh,
        &planes[U_C].as_region(),
        &pred.data[U_C],
        &mut diff.data[U_C],
    );

    /* V */
    let buf = &planes[V_C].as_region();
    evce_diff_16b(
        x,
        y,
        log2_cuw,
        log2_cuh,
        &planes[V_C].as_region(),
        &pred.data[V_C],
        &mut diff.data[V_C],
    );
}

pub(crate) fn copy_tu_from_cu(
    tu_resi: &mut CUBuffer<i16>,
    cu_resi: &CUBuffer<i16>,
    log2_cuw: usize,
    log2_cuh: usize,
) {
    let cuwh = (1 << log2_cuw) * (1 << log2_cuh);

    //Y
    tu_resi.data[Y_C][0..cuwh].copy_from_slice(&cu_resi.data[Y_C][0..cuwh]);

    //UV
    let cuwh = cuwh >> 2;
    tu_resi.data[U_C][0..cuwh].copy_from_slice(&cu_resi.data[U_C][0..cuwh]);
    tu_resi.data[V_C][0..cuwh].copy_from_slice(&cu_resi.data[V_C][0..cuwh]);
}

/* Get original dummy buffer for bi prediction */
pub(crate) fn get_org_bi(
    org_bi: &mut [i16],
    org: &PlaneRegion<'_, pel>,
    pred: &[pel],
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
) {
    for j in 0..cuh {
        for i in 0..cuw {
            org_bi[j * cuw + i] = (org[y + j][x + i] << 1) as i16 - pred[j * cuw + i] as i16;
        }
    }
}

#[inline]
pub(crate) fn get_exp_golomb_bits(abs_mvd: u32) -> u32 {
    let mut bits = 0;

    /* abs(mvd) */
    let mut nn = ((abs_mvd + 1) >> 1);
    let mut len_i = 0;
    while len_i < 16 && nn != 0 {
        nn >>= 1;
        len_i += 1;
    }
    let len_c = (len_i << 1) + 1;

    bits += len_c;

    /* sign */
    if abs_mvd != 0 {
        bits += 1;
    }

    return bits;
}

pub(crate) fn get_mv_bits(mvd_x: i16, mvd_y: i16, num_refp: u8, refi: i8) -> u32 {
    let mut bits = if mvd_x > 2048 || mvd_x <= -2048 {
        get_exp_golomb_bits(mvd_x.abs() as u32)
    } else {
        evce_tbl_mv_bits_data[(MV_BITS_BASE as i16 + mvd_x) as usize] as u32
    };
    bits += if mvd_y > 2048 || mvd_y <= -2048 {
        get_exp_golomb_bits(mvd_y.abs() as u32)
    } else {
        evce_tbl_mv_bits_data[(MV_BITS_BASE as i16 + mvd_y) as usize] as u32
    };
    bits += evce_tbl_refi_bits[num_refp as usize][refi as usize] as u32;

    bits
}

#[inline]
pub(crate) fn MV_COST(lambda_mv: u32, mv_bits: u32) -> u32 {
    (lambda_mv * mv_bits + (1 << 15)) >> 16
}
