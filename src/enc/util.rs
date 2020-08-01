use super::*;
use crate::def::*;

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
