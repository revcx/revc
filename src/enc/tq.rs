use super::util::*;
use super::*;
use crate::api::*;
use crate::def::*;
use crate::tbl::*;
use crate::util::*;

const TX_SHIFT1: usize = BIT_DEPTH - 8 - 1;
const TX_SHIFT2: usize = 6;
const quant_scale: [u16; 6] = [26214, 23302, 20560, 18396, 16384, 14564];
const GET_IEP_RATE: i32 = (32768);

const FAST_RDOQ_INTRA_RND_OFST: i64 = 201; //171
const FAST_RDOQ_INTER_RND_OFST: i64 = 153; //85

lazy_static! {
    static ref err_scale_tbl: [Box<[i64]>; 6] = {
        [
            evce_init_err_scale(0),
            evce_init_err_scale(1),
            evce_init_err_scale(2),
            evce_init_err_scale(3),
            evce_init_err_scale(4),
            evce_init_err_scale(5),
        ]
    };
}

fn evce_init_err_scale(qp: usize) -> Box<[i64]> {
    let mut tbl = vec![0; MAX_CU_DEPTH].into_boxed_slice();
    let q_value = quant_scale[qp];

    for i in 0..MAX_CU_DEPTH {
        let tr_shift = MAX_TX_DYNAMIC_RANGE as f64 - BIT_DEPTH as f64 - (i as f64 + 1.0);

        let mut err_scale = (1 << SCALE_BITS) as f64 * (2.0f64).powf(-tr_shift);
        err_scale = err_scale / q_value as f64 / (1 << (BIT_DEPTH - 8)) as f64;
        tbl[i] = (err_scale * (1 << ERR_SCALE_PRECISION_BITS) as f64) as i64;
    }

    tbl
}

fn evc_get_transform_shift(log2_size: usize, typ: u8) -> usize {
    if typ == 0 {
        TX_SHIFT1 + log2_size
    } else {
        TX_SHIFT2 + log2_size
    }
}

fn tx_pb2b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O */
        let E = src[j * 2 + 0] as i64 + src[j * 2 + 1] as i64;
        let O = src[j * 2 + 0] as i64 - src[j * 2 + 1] as i64;

        dst[0 * line + j] = ((evc_tbl_tm2[0][0] as i64 * E + add) >> shift) as i32;
        dst[1 * line + j] = ((evc_tbl_tm2[1][0] as i64 * O + add) >> shift) as i32;
    }
}
fn tx_pb2b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O */
        let E = src[j * 2 + 0] as i64 + src[j * 2 + 1] as i64;
        let O = src[j * 2 + 0] as i64 - src[j * 2 + 1] as i64;

        dst[0 * line + j] = ((evc_tbl_tm2[0][0] as i64 * E + add) >> shift) as i16;
        dst[1 * line + j] = ((evc_tbl_tm2[1][0] as i64 * O + add) >> shift) as i16;
    }
}

fn tx_pb4b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let mut E = [0i64; 2];
    let mut O = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O */
        E[0] = src[j * 4 + 0] as i64 + src[j * 4 + 3] as i64;
        O[0] = src[j * 4 + 0] as i64 - src[j * 4 + 3] as i64;
        E[1] = src[j * 4 + 1] as i64 + src[j * 4 + 2] as i64;
        O[1] = src[j * 4 + 1] as i64 - src[j * 4 + 2] as i64;

        dst[0 * line + j] =
            ((evc_tbl_tm4[0][0] as i64 * E[0] + evc_tbl_tm4[0][1] as i64 * E[1] + add) >> shift)
                as i32;
        dst[2 * line + j] =
            ((evc_tbl_tm4[2][0] as i64 * E[0] + evc_tbl_tm4[2][1] as i64 * E[1] + add) >> shift)
                as i32;
        dst[1 * line + j] =
            ((evc_tbl_tm4[1][0] as i64 * O[0] + evc_tbl_tm4[1][1] as i64 * O[1] + add) >> shift)
                as i32;
        dst[3 * line + j] =
            ((evc_tbl_tm4[3][0] as i64 * O[0] + evc_tbl_tm4[3][1] as i64 * O[1] + add) >> shift)
                as i32;
    }
}
fn tx_pb4b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let mut E = [0i64; 2];
    let mut O = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O */
        E[0] = src[j * 4 + 0] as i64 + src[j * 4 + 3] as i64;
        O[0] = src[j * 4 + 0] as i64 - src[j * 4 + 3] as i64;
        E[1] = src[j * 4 + 1] as i64 + src[j * 4 + 2] as i64;
        O[1] = src[j * 4 + 1] as i64 - src[j * 4 + 2] as i64;

        dst[0 * line + j] =
            ((evc_tbl_tm4[0][0] as i64 * E[0] + evc_tbl_tm4[0][1] as i64 * E[1] + add) >> shift)
                as i16;
        dst[2 * line + j] =
            ((evc_tbl_tm4[2][0] as i64 * E[0] + evc_tbl_tm4[2][1] as i64 * E[1] + add) >> shift)
                as i16;
        dst[1 * line + j] =
            ((evc_tbl_tm4[1][0] as i64 * O[0] + evc_tbl_tm4[1][1] as i64 * O[1] + add) >> shift)
                as i16;
        dst[3 * line + j] =
            ((evc_tbl_tm4[3][0] as i64 * O[0] + evc_tbl_tm4[3][1] as i64 * O[1] + add) >> shift)
                as i16;
    }
}

fn tx_pb8b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let mut E = [0i64; 4];
    let mut O = [0i64; 4];
    let mut EE = [0i64; 2];
    let mut EO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..4 {
            E[k] = src[j * 8 + k] as i64 + src[j * 8 + 7 - k] as i64;
            O[k] = src[j * 8 + k] as i64 - src[j * 8 + 7 - k] as i64;
        }
        /* EE and EO */
        EE[0] = E[0] + E[3];
        EO[0] = E[0] - E[3];
        EE[1] = E[1] + E[2];
        EO[1] = E[1] - E[2];

        dst[0 * line + j] =
            ((evc_tbl_tm8[0][0] as i64 * EE[0] + evc_tbl_tm8[0][1] as i64 * EE[1] + add) >> shift)
                as i32;
        dst[4 * line + j] =
            ((evc_tbl_tm8[4][0] as i64 * EE[0] + evc_tbl_tm8[4][1] as i64 * EE[1] + add) >> shift)
                as i32;
        dst[2 * line + j] =
            ((evc_tbl_tm8[2][0] as i64 * EO[0] + evc_tbl_tm8[2][1] as i64 * EO[1] + add) >> shift)
                as i32;
        dst[6 * line + j] =
            ((evc_tbl_tm8[6][0] as i64 * EO[0] + evc_tbl_tm8[6][1] as i64 * EO[1] + add) >> shift)
                as i32;

        dst[1 * line + j] = ((evc_tbl_tm8[1][0] as i64 * O[0]
            + evc_tbl_tm8[1][1] as i64 * O[1]
            + evc_tbl_tm8[1][2] as i64 * O[2]
            + evc_tbl_tm8[1][3] as i64 * O[3]
            + add)
            >> shift) as i32;
        dst[3 * line + j] = ((evc_tbl_tm8[3][0] as i64 * O[0]
            + evc_tbl_tm8[3][1] as i64 * O[1]
            + evc_tbl_tm8[3][2] as i64 * O[2]
            + evc_tbl_tm8[3][3] as i64 * O[3]
            + add)
            >> shift) as i32;
        dst[5 * line + j] = ((evc_tbl_tm8[5][0] as i64 * O[0]
            + evc_tbl_tm8[5][1] as i64 * O[1]
            + evc_tbl_tm8[5][2] as i64 * O[2]
            + evc_tbl_tm8[5][3] as i64 * O[3]
            + add)
            >> shift) as i32;
        dst[7 * line + j] = ((evc_tbl_tm8[7][0] as i64 * O[0]
            + evc_tbl_tm8[7][1] as i64 * O[1]
            + evc_tbl_tm8[7][2] as i64 * O[2]
            + evc_tbl_tm8[7][3] as i64 * O[3]
            + add)
            >> shift) as i32;
    }
}
fn tx_pb8b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let mut E = [0i64; 4];
    let mut O = [0i64; 4];
    let mut EE = [0i64; 2];
    let mut EO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..4 {
            E[k] = src[j * 8 + k] as i64 + src[j * 8 + 7 - k] as i64;
            O[k] = src[j * 8 + k] as i64 - src[j * 8 + 7 - k] as i64;
        }
        /* EE and EO */
        EE[0] = E[0] + E[3];
        EO[0] = E[0] - E[3];
        EE[1] = E[1] + E[2];
        EO[1] = E[1] - E[2];

        dst[0 * line + j] =
            ((evc_tbl_tm8[0][0] as i64 * EE[0] + evc_tbl_tm8[0][1] as i64 * EE[1] + add) >> shift)
                as i16;
        dst[4 * line + j] =
            ((evc_tbl_tm8[4][0] as i64 * EE[0] + evc_tbl_tm8[4][1] as i64 * EE[1] + add) >> shift)
                as i16;
        dst[2 * line + j] =
            ((evc_tbl_tm8[2][0] as i64 * EO[0] + evc_tbl_tm8[2][1] as i64 * EO[1] + add) >> shift)
                as i16;
        dst[6 * line + j] =
            ((evc_tbl_tm8[6][0] as i64 * EO[0] + evc_tbl_tm8[6][1] as i64 * EO[1] + add) >> shift)
                as i16;

        dst[1 * line + j] = ((evc_tbl_tm8[1][0] as i64 * O[0]
            + evc_tbl_tm8[1][1] as i64 * O[1]
            + evc_tbl_tm8[1][2] as i64 * O[2]
            + evc_tbl_tm8[1][3] as i64 * O[3]
            + add)
            >> shift) as i16;
        dst[3 * line + j] = ((evc_tbl_tm8[3][0] as i64 * O[0]
            + evc_tbl_tm8[3][1] as i64 * O[1]
            + evc_tbl_tm8[3][2] as i64 * O[2]
            + evc_tbl_tm8[3][3] as i64 * O[3]
            + add)
            >> shift) as i16;
        dst[5 * line + j] = ((evc_tbl_tm8[5][0] as i64 * O[0]
            + evc_tbl_tm8[5][1] as i64 * O[1]
            + evc_tbl_tm8[5][2] as i64 * O[2]
            + evc_tbl_tm8[5][3] as i64 * O[3]
            + add)
            >> shift) as i16;
        dst[7 * line + j] = ((evc_tbl_tm8[7][0] as i64 * O[0]
            + evc_tbl_tm8[7][1] as i64 * O[1]
            + evc_tbl_tm8[7][2] as i64 * O[2]
            + evc_tbl_tm8[7][3] as i64 * O[3]
            + add)
            >> shift) as i16;
    }
}

fn tx_pb16b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let mut E = [0i64; 8];
    let mut O = [0i64; 8];
    let mut EE = [0i64; 4];
    let mut EO = [0i64; 4];
    let mut EEE = [0i64; 2];
    let mut EEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..8 {
            E[k] = src[j * 16 + k] as i64 + src[j * 16 + 15 - k] as i64;
            O[k] = src[j * 16 + k] as i64 - src[j * 16 + 15 - k] as i64;
        }
        /* EE and EO */
        for k in 0..4 {
            EE[k] = E[k] + E[7 - k];
            EO[k] = E[k] - E[7 - k];
        }
        /* EEE and EEO */
        EEE[0] = EE[0] + EE[3];
        EEO[0] = EE[0] - EE[3];
        EEE[1] = EE[1] + EE[2];
        EEO[1] = EE[1] - EE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm16[0][0] as i64 * EEE[0] + evc_tbl_tm16[0][1] as i64 * EEE[1] + add)
                >> shift) as i32;
        dst[8 * line + j] =
            ((evc_tbl_tm16[8][0] as i64 * EEE[0] + evc_tbl_tm16[8][1] as i64 * EEE[1] + add)
                >> shift) as i32;
        dst[4 * line + j] =
            ((evc_tbl_tm16[4][0] as i64 * EEO[0] + evc_tbl_tm16[4][1] as i64 * EEO[1] + add)
                >> shift) as i32;
        dst[12 * line + j] =
            ((evc_tbl_tm16[12][0] as i64 * EEO[0] + evc_tbl_tm16[12][1] as i64 * EEO[1] + add)
                >> shift) as i32;

        for k in (2..16).step_by(4) {
            dst[k * line + j] = ((evc_tbl_tm16[k][0] as i64 * EO[0]
                + evc_tbl_tm16[k][1] as i64 * EO[1]
                + evc_tbl_tm16[k][2] as i64 * EO[2]
                + evc_tbl_tm16[k][3] as i64 * EO[3]
                + add)
                >> shift) as i32;
        }

        for k in (1..16).step_by(2) {
            dst[k * line + j] = ((evc_tbl_tm16[k][0] as i64 * O[0]
                + evc_tbl_tm16[k][1] as i64 * O[1]
                + evc_tbl_tm16[k][2] as i64 * O[2]
                + evc_tbl_tm16[k][3] as i64 * O[3]
                + evc_tbl_tm16[k][4] as i64 * O[4]
                + evc_tbl_tm16[k][5] as i64 * O[5]
                + evc_tbl_tm16[k][6] as i64 * O[6]
                + evc_tbl_tm16[k][7] as i64 * O[7]
                + add)
                >> shift) as i32;
        }
    }
}
fn tx_pb16b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let mut E = [0i64; 8];
    let mut O = [0i64; 8];
    let mut EE = [0i64; 4];
    let mut EO = [0i64; 4];
    let mut EEE = [0i64; 2];
    let mut EEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..8 {
            E[k] = src[j * 16 + k] as i64 + src[j * 16 + 15 - k] as i64;
            O[k] = src[j * 16 + k] as i64 - src[j * 16 + 15 - k] as i64;
        }
        /* EE and EO */
        for k in 0..4 {
            EE[k] = E[k] + E[7 - k];
            EO[k] = E[k] - E[7 - k];
        }
        /* EEE and EEO */
        EEE[0] = EE[0] + EE[3];
        EEO[0] = EE[0] - EE[3];
        EEE[1] = EE[1] + EE[2];
        EEO[1] = EE[1] - EE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm16[0][0] as i64 * EEE[0] + evc_tbl_tm16[0][1] as i64 * EEE[1] + add)
                >> shift) as i16;
        dst[8 * line + j] =
            ((evc_tbl_tm16[8][0] as i64 * EEE[0] + evc_tbl_tm16[8][1] as i64 * EEE[1] + add)
                >> shift) as i16;
        dst[4 * line + j] =
            ((evc_tbl_tm16[4][0] as i64 * EEO[0] + evc_tbl_tm16[4][1] as i64 * EEO[1] + add)
                >> shift) as i16;
        dst[12 * line + j] =
            ((evc_tbl_tm16[12][0] as i64 * EEO[0] + evc_tbl_tm16[12][1] as i64 * EEO[1] + add)
                >> shift) as i16;

        for k in (2..16).step_by(4) {
            dst[k * line + j] = ((evc_tbl_tm16[k][0] as i64 * EO[0]
                + evc_tbl_tm16[k][1] as i64 * EO[1]
                + evc_tbl_tm16[k][2] as i64 * EO[2]
                + evc_tbl_tm16[k][3] as i64 * EO[3]
                + add)
                >> shift) as i16;
        }

        for k in (1..16).step_by(2) {
            dst[k * line + j] = ((evc_tbl_tm16[k][0] as i64 * O[0]
                + evc_tbl_tm16[k][1] as i64 * O[1]
                + evc_tbl_tm16[k][2] as i64 * O[2]
                + evc_tbl_tm16[k][3] as i64 * O[3]
                + evc_tbl_tm16[k][4] as i64 * O[4]
                + evc_tbl_tm16[k][5] as i64 * O[5]
                + evc_tbl_tm16[k][6] as i64 * O[6]
                + evc_tbl_tm16[k][7] as i64 * O[7]
                + add)
                >> shift) as i16;
        }
    }
}

fn tx_pb32b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let mut E = [0i64; 16];
    let mut O = [0i64; 16];
    let mut EE = [0i64; 8];
    let mut EO = [0i64; 8];
    let mut EEE = [0i64; 4];
    let mut EEO = [0i64; 4];
    let mut EEEE = [0i64; 2];
    let mut EEEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..16 {
            E[k] = src[j * 32 + k] as i64 + src[j * 32 + 31 - k] as i64;
            O[k] = src[j * 32 + k] as i64 - src[j * 32 + 31 - k] as i64;
        }
        /* EE and EO */
        for k in 0..8 {
            EE[k] = E[k] + E[15 - k];
            EO[k] = E[k] - E[15 - k];
        }
        /* EEE and EEO */
        for k in 0..4 {
            EEE[k] = EE[k] + EE[7 - k];
            EEO[k] = EE[k] - EE[7 - k];
        }
        /* EEEE and EEEO */
        EEEE[0] = EEE[0] + EEE[3];
        EEEO[0] = EEE[0] - EEE[3];
        EEEE[1] = EEE[1] + EEE[2];
        EEEO[1] = EEE[1] - EEE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm32[0][0] as i64 * EEEE[0] + evc_tbl_tm32[0][1] as i64 * EEEE[1] + add)
                >> shift) as i32;
        dst[16 * line + j] =
            ((evc_tbl_tm32[16][0] as i64 * EEEE[0] + evc_tbl_tm32[16][1] as i64 * EEEE[1] + add)
                >> shift) as i32;
        dst[8 * line + j] =
            ((evc_tbl_tm32[8][0] as i64 * EEEO[0] + evc_tbl_tm32[8][1] as i64 * EEEO[1] + add)
                >> shift) as i32;
        dst[24 * line + j] =
            ((evc_tbl_tm32[24][0] as i64 * EEEO[0] + evc_tbl_tm32[24][1] as i64 * EEEO[1] + add)
                >> shift) as i32;
        for k in (4..32).step_by(8) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * EEO[0]
                + evc_tbl_tm32[k][1] as i64 * EEO[1]
                + evc_tbl_tm32[k][2] as i64 * EEO[2]
                + evc_tbl_tm32[k][3] as i64 * EEO[3]
                + add)
                >> shift) as i32;
        }
        for k in (2..32).step_by(4) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * EO[0]
                + evc_tbl_tm32[k][1] as i64 * EO[1]
                + evc_tbl_tm32[k][2] as i64 * EO[2]
                + evc_tbl_tm32[k][3] as i64 * EO[3]
                + evc_tbl_tm32[k][4] as i64 * EO[4]
                + evc_tbl_tm32[k][5] as i64 * EO[5]
                + evc_tbl_tm32[k][6] as i64 * EO[6]
                + evc_tbl_tm32[k][7] as i64 * EO[7]
                + add)
                >> shift) as i32;
        }
        for k in (1..32).step_by(2) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * O[0]
                + evc_tbl_tm32[k][1] as i64 * O[1]
                + evc_tbl_tm32[k][2] as i64 * O[2]
                + evc_tbl_tm32[k][3] as i64 * O[3]
                + evc_tbl_tm32[k][4] as i64 * O[4]
                + evc_tbl_tm32[k][5] as i64 * O[5]
                + evc_tbl_tm32[k][6] as i64 * O[6]
                + evc_tbl_tm32[k][7] as i64 * O[7]
                + evc_tbl_tm32[k][8] as i64 * O[8]
                + evc_tbl_tm32[k][9] as i64 * O[9]
                + evc_tbl_tm32[k][10] as i64 * O[10]
                + evc_tbl_tm32[k][11] as i64 * O[11]
                + evc_tbl_tm32[k][12] as i64 * O[12]
                + evc_tbl_tm32[k][13] as i64 * O[13]
                + evc_tbl_tm32[k][14] as i64 * O[14]
                + evc_tbl_tm32[k][15] as i64 * O[15]
                + add)
                >> shift) as i32;
        }
    }
}
fn tx_pb32b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let mut E = [0i64; 16];
    let mut O = [0i64; 16];
    let mut EE = [0i64; 8];
    let mut EO = [0i64; 8];
    let mut EEE = [0i64; 4];
    let mut EEO = [0i64; 4];
    let mut EEEE = [0i64; 2];
    let mut EEEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        /* E and O*/
        for k in 0..16 {
            E[k] = src[j * 32 + k] as i64 + src[j * 32 + 31 - k] as i64;
            O[k] = src[j * 32 + k] as i64 - src[j * 32 + 31 - k] as i64;
        }
        /* EE and EO */
        for k in 0..8 {
            EE[k] = E[k] + E[15 - k];
            EO[k] = E[k] - E[15 - k];
        }
        /* EEE and EEO */
        for k in 0..4 {
            EEE[k] = EE[k] + EE[7 - k];
            EEO[k] = EE[k] - EE[7 - k];
        }
        /* EEEE and EEEO */
        EEEE[0] = EEE[0] + EEE[3];
        EEEO[0] = EEE[0] - EEE[3];
        EEEE[1] = EEE[1] + EEE[2];
        EEEO[1] = EEE[1] - EEE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm32[0][0] as i64 * EEEE[0] + evc_tbl_tm32[0][1] as i64 * EEEE[1] + add)
                >> shift) as i16;
        dst[16 * line + j] =
            ((evc_tbl_tm32[16][0] as i64 * EEEE[0] + evc_tbl_tm32[16][1] as i64 * EEEE[1] + add)
                >> shift) as i16;
        dst[8 * line + j] =
            ((evc_tbl_tm32[8][0] as i64 * EEEO[0] + evc_tbl_tm32[8][1] as i64 * EEEO[1] + add)
                >> shift) as i16;
        dst[24 * line + j] =
            ((evc_tbl_tm32[24][0] as i64 * EEEO[0] + evc_tbl_tm32[24][1] as i64 * EEEO[1] + add)
                >> shift) as i16;
        for k in (4..32).step_by(8) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * EEO[0]
                + evc_tbl_tm32[k][1] as i64 * EEO[1]
                + evc_tbl_tm32[k][2] as i64 * EEO[2]
                + evc_tbl_tm32[k][3] as i64 * EEO[3]
                + add)
                >> shift) as i16;
        }
        for k in (2..32).step_by(4) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * EO[0]
                + evc_tbl_tm32[k][1] as i64 * EO[1]
                + evc_tbl_tm32[k][2] as i64 * EO[2]
                + evc_tbl_tm32[k][3] as i64 * EO[3]
                + evc_tbl_tm32[k][4] as i64 * EO[4]
                + evc_tbl_tm32[k][5] as i64 * EO[5]
                + evc_tbl_tm32[k][6] as i64 * EO[6]
                + evc_tbl_tm32[k][7] as i64 * EO[7]
                + add)
                >> shift) as i16;
        }
        for k in (1..32).step_by(2) {
            dst[k * line + j] = ((evc_tbl_tm32[k][0] as i64 * O[0]
                + evc_tbl_tm32[k][1] as i64 * O[1]
                + evc_tbl_tm32[k][2] as i64 * O[2]
                + evc_tbl_tm32[k][3] as i64 * O[3]
                + evc_tbl_tm32[k][4] as i64 * O[4]
                + evc_tbl_tm32[k][5] as i64 * O[5]
                + evc_tbl_tm32[k][6] as i64 * O[6]
                + evc_tbl_tm32[k][7] as i64 * O[7]
                + evc_tbl_tm32[k][8] as i64 * O[8]
                + evc_tbl_tm32[k][9] as i64 * O[9]
                + evc_tbl_tm32[k][10] as i64 * O[10]
                + evc_tbl_tm32[k][11] as i64 * O[11]
                + evc_tbl_tm32[k][12] as i64 * O[12]
                + evc_tbl_tm32[k][13] as i64 * O[13]
                + evc_tbl_tm32[k][14] as i64 * O[14]
                + evc_tbl_tm32[k][15] as i64 * O[15]
                + add)
                >> shift) as i16;
        }
    }
}

fn tx_pb64b0(src: &[i16], dst: &mut [i32], shift: usize, line: usize) {
    let mut E = [0i64; 32];
    let mut O = [0i64; 32];
    let mut EE = [0i64; 16];
    let mut EO = [0i64; 16];
    let mut EEE = [0i64; 8];
    let mut EEO = [0i64; 8];
    let mut EEEE = [0i64; 4];
    let mut EEEO = [0i64; 4];
    let mut EEEEE = [0i64; 2];
    let mut EEEEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        for k in 0..32 {
            E[k] = src[j * 64 + k] as i64 + src[j * 64 + 63 - k] as i64;
            O[k] = src[j * 64 + k] as i64 - src[j * 64 + 63 - k] as i64;
        }
        for k in 0..16 {
            EE[k] = E[k] + E[31 - k];
            EO[k] = E[k] - E[31 - k];
        }
        for k in 0..8 {
            EEE[k] = EE[k] + EE[15 - k];
            EEO[k] = EE[k] - EE[15 - k];
        }
        for k in 0..4 {
            EEEE[k] = EEE[k] + EEE[7 - k];
            EEEO[k] = EEE[k] - EEE[7 - k];
        }
        EEEEE[0] = EEEE[0] + EEEE[3];
        EEEEO[0] = EEEE[0] - EEEE[3];
        EEEEE[1] = EEEE[1] + EEEE[2];
        EEEEO[1] = EEEE[1] - EEEE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm64[0][0] as i64 * EEEEE[0] + evc_tbl_tm64[0][1] as i64 * EEEEE[1] + add)
                >> shift) as i32;
        dst[16 * line + j] =
            ((evc_tbl_tm64[16][0] as i64 * EEEEO[0] + evc_tbl_tm64[16][1] as i64 * EEEEO[1] + add)
                >> shift) as i32;
        dst[32 * line + j] = 0;
        dst[48 * line + j] = 0;

        for k in (8..64).step_by(16) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EEEO[0]
                    + evc_tbl_tm64[k][1] as i64 * EEEO[1]
                    + evc_tbl_tm64[k][2] as i64 * EEEO[2]
                    + evc_tbl_tm64[k][3] as i64 * EEEO[3]
                    + add)
                    >> shift) as i32;
            }
        }
        for k in (4..64).step_by(8) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EEO[0]
                    + evc_tbl_tm64[k][1] as i64 * EEO[1]
                    + evc_tbl_tm64[k][2] as i64 * EEO[2]
                    + evc_tbl_tm64[k][3] as i64 * EEO[3]
                    + evc_tbl_tm64[k][4] as i64 * EEO[4]
                    + evc_tbl_tm64[k][5] as i64 * EEO[5]
                    + evc_tbl_tm64[k][6] as i64 * EEO[6]
                    + evc_tbl_tm64[k][7] as i64 * EEO[7]
                    + add)
                    >> shift) as i32;
            }
        }
        for k in (2..64).step_by(4) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EO[0]
                    + evc_tbl_tm64[k][1] as i64 * EO[1]
                    + evc_tbl_tm64[k][2] as i64 * EO[2]
                    + evc_tbl_tm64[k][3] as i64 * EO[3]
                    + evc_tbl_tm64[k][4] as i64 * EO[4]
                    + evc_tbl_tm64[k][5] as i64 * EO[5]
                    + evc_tbl_tm64[k][6] as i64 * EO[6]
                    + evc_tbl_tm64[k][7] as i64 * EO[7]
                    + evc_tbl_tm64[k][8] as i64 * EO[8]
                    + evc_tbl_tm64[k][9] as i64 * EO[9]
                    + evc_tbl_tm64[k][10] as i64 * EO[10]
                    + evc_tbl_tm64[k][11] as i64 * EO[11]
                    + evc_tbl_tm64[k][12] as i64 * EO[12]
                    + evc_tbl_tm64[k][13] as i64 * EO[13]
                    + evc_tbl_tm64[k][14] as i64 * EO[14]
                    + evc_tbl_tm64[k][15] as i64 * EO[15]
                    + add)
                    >> shift) as i32;
            }
        }
        for k in (1..64).step_by(2) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * O[0]
                    + evc_tbl_tm64[k][1] as i64 * O[1]
                    + evc_tbl_tm64[k][2] as i64 * O[2]
                    + evc_tbl_tm64[k][3] as i64 * O[3]
                    + evc_tbl_tm64[k][4] as i64 * O[4]
                    + evc_tbl_tm64[k][5] as i64 * O[5]
                    + evc_tbl_tm64[k][6] as i64 * O[6]
                    + evc_tbl_tm64[k][7] as i64 * O[7]
                    + evc_tbl_tm64[k][8] as i64 * O[8]
                    + evc_tbl_tm64[k][9] as i64 * O[9]
                    + evc_tbl_tm64[k][10] as i64 * O[10]
                    + evc_tbl_tm64[k][11] as i64 * O[11]
                    + evc_tbl_tm64[k][12] as i64 * O[12]
                    + evc_tbl_tm64[k][13] as i64 * O[13]
                    + evc_tbl_tm64[k][14] as i64 * O[14]
                    + evc_tbl_tm64[k][15] as i64 * O[15]
                    + evc_tbl_tm64[k][16] as i64 * O[16]
                    + evc_tbl_tm64[k][17] as i64 * O[17]
                    + evc_tbl_tm64[k][18] as i64 * O[18]
                    + evc_tbl_tm64[k][19] as i64 * O[19]
                    + evc_tbl_tm64[k][20] as i64 * O[20]
                    + evc_tbl_tm64[k][21] as i64 * O[21]
                    + evc_tbl_tm64[k][22] as i64 * O[22]
                    + evc_tbl_tm64[k][23] as i64 * O[23]
                    + evc_tbl_tm64[k][24] as i64 * O[24]
                    + evc_tbl_tm64[k][25] as i64 * O[25]
                    + evc_tbl_tm64[k][26] as i64 * O[26]
                    + evc_tbl_tm64[k][27] as i64 * O[27]
                    + evc_tbl_tm64[k][28] as i64 * O[28]
                    + evc_tbl_tm64[k][29] as i64 * O[29]
                    + evc_tbl_tm64[k][30] as i64 * O[30]
                    + evc_tbl_tm64[k][31] as i64 * O[31]
                    + add)
                    >> shift) as i32;
            }
        }
    }
}
fn tx_pb64b1(src: &[i32], dst: &mut [i16], shift: usize, line: usize) {
    let mut E = [0i64; 32];
    let mut O = [0i64; 32];
    let mut EE = [0i64; 16];
    let mut EO = [0i64; 16];
    let mut EEE = [0i64; 8];
    let mut EEO = [0i64; 8];
    let mut EEEE = [0i64; 4];
    let mut EEEO = [0i64; 4];
    let mut EEEEE = [0i64; 2];
    let mut EEEEO = [0i64; 2];
    let add = if shift == 0 { 0 } else { 1 << (shift - 1) };

    for j in 0..line {
        for k in 0..32 {
            E[k] = src[j * 64 + k] as i64 + src[j * 64 + 63 - k] as i64;
            O[k] = src[j * 64 + k] as i64 - src[j * 64 + 63 - k] as i64;
        }
        for k in 0..16 {
            EE[k] = E[k] + E[31 - k];
            EO[k] = E[k] - E[31 - k];
        }
        for k in 0..8 {
            EEE[k] = EE[k] + EE[15 - k];
            EEO[k] = EE[k] - EE[15 - k];
        }
        for k in 0..4 {
            EEEE[k] = EEE[k] + EEE[7 - k];
            EEEO[k] = EEE[k] - EEE[7 - k];
        }
        EEEEE[0] = EEEE[0] + EEEE[3];
        EEEEO[0] = EEEE[0] - EEEE[3];
        EEEEE[1] = EEEE[1] + EEEE[2];
        EEEEO[1] = EEEE[1] - EEEE[2];

        dst[0 * line + j] =
            ((evc_tbl_tm64[0][0] as i64 * EEEEE[0] + evc_tbl_tm64[0][1] as i64 * EEEEE[1] + add)
                >> shift) as i16;
        dst[16 * line + j] =
            ((evc_tbl_tm64[16][0] as i64 * EEEEO[0] + evc_tbl_tm64[16][1] as i64 * EEEEO[1] + add)
                >> shift) as i16;
        dst[32 * line + j] = 0;
        dst[48 * line + j] = 0;

        for k in (8..64).step_by(16) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EEEO[0]
                    + evc_tbl_tm64[k][1] as i64 * EEEO[1]
                    + evc_tbl_tm64[k][2] as i64 * EEEO[2]
                    + evc_tbl_tm64[k][3] as i64 * EEEO[3]
                    + add)
                    >> shift) as i16;
            }
        }
        for k in (4..64).step_by(8) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EEO[0]
                    + evc_tbl_tm64[k][1] as i64 * EEO[1]
                    + evc_tbl_tm64[k][2] as i64 * EEO[2]
                    + evc_tbl_tm64[k][3] as i64 * EEO[3]
                    + evc_tbl_tm64[k][4] as i64 * EEO[4]
                    + evc_tbl_tm64[k][5] as i64 * EEO[5]
                    + evc_tbl_tm64[k][6] as i64 * EEO[6]
                    + evc_tbl_tm64[k][7] as i64 * EEO[7]
                    + add)
                    >> shift) as i16;
            }
        }
        for k in (2..64).step_by(4) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * EO[0]
                    + evc_tbl_tm64[k][1] as i64 * EO[1]
                    + evc_tbl_tm64[k][2] as i64 * EO[2]
                    + evc_tbl_tm64[k][3] as i64 * EO[3]
                    + evc_tbl_tm64[k][4] as i64 * EO[4]
                    + evc_tbl_tm64[k][5] as i64 * EO[5]
                    + evc_tbl_tm64[k][6] as i64 * EO[6]
                    + evc_tbl_tm64[k][7] as i64 * EO[7]
                    + evc_tbl_tm64[k][8] as i64 * EO[8]
                    + evc_tbl_tm64[k][9] as i64 * EO[9]
                    + evc_tbl_tm64[k][10] as i64 * EO[10]
                    + evc_tbl_tm64[k][11] as i64 * EO[11]
                    + evc_tbl_tm64[k][12] as i64 * EO[12]
                    + evc_tbl_tm64[k][13] as i64 * EO[13]
                    + evc_tbl_tm64[k][14] as i64 * EO[14]
                    + evc_tbl_tm64[k][15] as i64 * EO[15]
                    + add)
                    >> shift) as i16;
            }
        }
        for k in (1..64).step_by(2) {
            if k > 31 {
                dst[k * line + j] = 0;
            } else {
                dst[k * line + j] = ((evc_tbl_tm64[k][0] as i64 * O[0]
                    + evc_tbl_tm64[k][1] as i64 * O[1]
                    + evc_tbl_tm64[k][2] as i64 * O[2]
                    + evc_tbl_tm64[k][3] as i64 * O[3]
                    + evc_tbl_tm64[k][4] as i64 * O[4]
                    + evc_tbl_tm64[k][5] as i64 * O[5]
                    + evc_tbl_tm64[k][6] as i64 * O[6]
                    + evc_tbl_tm64[k][7] as i64 * O[7]
                    + evc_tbl_tm64[k][8] as i64 * O[8]
                    + evc_tbl_tm64[k][9] as i64 * O[9]
                    + evc_tbl_tm64[k][10] as i64 * O[10]
                    + evc_tbl_tm64[k][11] as i64 * O[11]
                    + evc_tbl_tm64[k][12] as i64 * O[12]
                    + evc_tbl_tm64[k][13] as i64 * O[13]
                    + evc_tbl_tm64[k][14] as i64 * O[14]
                    + evc_tbl_tm64[k][15] as i64 * O[15]
                    + evc_tbl_tm64[k][16] as i64 * O[16]
                    + evc_tbl_tm64[k][17] as i64 * O[17]
                    + evc_tbl_tm64[k][18] as i64 * O[18]
                    + evc_tbl_tm64[k][19] as i64 * O[19]
                    + evc_tbl_tm64[k][20] as i64 * O[20]
                    + evc_tbl_tm64[k][21] as i64 * O[21]
                    + evc_tbl_tm64[k][22] as i64 * O[22]
                    + evc_tbl_tm64[k][23] as i64 * O[23]
                    + evc_tbl_tm64[k][24] as i64 * O[24]
                    + evc_tbl_tm64[k][25] as i64 * O[25]
                    + evc_tbl_tm64[k][26] as i64 * O[26]
                    + evc_tbl_tm64[k][27] as i64 * O[27]
                    + evc_tbl_tm64[k][28] as i64 * O[28]
                    + evc_tbl_tm64[k][29] as i64 * O[29]
                    + evc_tbl_tm64[k][30] as i64 * O[30]
                    + evc_tbl_tm64[k][31] as i64 * O[31]
                    + add)
                    >> shift) as i16;
            }
        }
    }
}

type EVC_TXB0 = fn(src: &[i16], dst: &mut [i32], shift: usize, line: usize);
type EVC_TXB1 = fn(src: &[i32], dst: &mut [i16], shift: usize, line: usize);

static tbl_txb0: [EVC_TXB0; MAX_TR_LOG2] = [
    tx_pb2b0, tx_pb4b0, tx_pb8b0, tx_pb16b0, tx_pb32b0, tx_pb64b0,
];
static tbl_txb1: [EVC_TXB1; MAX_TR_LOG2] = [
    tx_pb2b1, tx_pb4b1, tx_pb8b1, tx_pb16b1, tx_pb32b1, tx_pb64b1,
];

fn evce_trans(coef: &mut [i16], log2_cuw: usize, log2_cuh: usize) {
    let shift1 = evc_get_transform_shift(log2_cuw, 0);
    let shift2 = evc_get_transform_shift(log2_cuh, 1);

    let mut tb = [0i32; MAX_TR_DIM]; /* temp buffer */
    tbl_txb0[log2_cuw - 1](coef, &mut tb, 0, 1 << log2_cuh);
    tbl_txb1[log2_cuh - 1](&tb, coef, (shift1 + shift2), 1 << log2_cuw);
}

fn get_ic_rate_cost_rl(
    abs_level: u32,
    run: u32,
    ctx_run: usize,
    ctx_level: usize,
    lambda: i64,
    rdoq_est: &EvceRdoqEst,
) -> i64 {
    let mut rate = 0i32;
    if abs_level == 0 {
        rate = 0;
        if run == 0 {
            rate += rdoq_est.run[ctx_run][1];
        } else {
            rate += rdoq_est.run[ctx_run + 1][1];
        }
    } else {
        rate = GET_IEP_RATE;
        if run == 0 {
            rate += rdoq_est.run[ctx_run][0];
        } else {
            rate += rdoq_est.run[ctx_run + 1][0];
        }

        if abs_level == 1 {
            rate += rdoq_est.level[ctx_level][0];
        } else {
            rate += rdoq_est.level[ctx_level][1];
            rate += rdoq_est.level[ctx_level + 1][1] * (abs_level as i32 - 2);
            rate += rdoq_est.level[ctx_level + 1][0];
        }
    }

    rate as i64 * lambda
}

fn get_coded_level_rl(
    rd64_uncoded_cost: &mut i64,
    rd64_coded_cost: &mut i64,
    level_double: i64,
    max_abs_level: u32,
    run: u32,
    ctx_run: usize,
    ctx_level: usize,
    q_bits: isize,
    err_scale: i64,
    lambda: i64,
    rdoq_est: &EvceRdoqEst,
) -> u32 {
    let mut best_abs_level = 0;
    let err1 = (level_double * err_scale) >> ERR_SCALE_PRECISION_BITS;

    *rd64_uncoded_cost = err1 * err1;
    *rd64_coded_cost =
        *rd64_uncoded_cost + get_ic_rate_cost_rl(0, run, ctx_run, ctx_level, lambda, rdoq_est);

    let min_abs_level = if max_abs_level > 1 {
        max_abs_level - 1
    } else {
        1
    };
    for abs_level in (min_abs_level..=max_abs_level).rev() {
        let i64Delta = level_double - ((abs_level as i64) << q_bits);
        let err = (i64Delta * err_scale) >> ERR_SCALE_PRECISION_BITS;
        let dCurrCost =
            err * err + get_ic_rate_cost_rl(abs_level, run, ctx_run, ctx_level, lambda, rdoq_est);

        if dCurrCost < *rd64_coded_cost {
            best_abs_level = abs_level;
            *rd64_coded_cost = dCurrCost;
        }
    }
    best_abs_level
}

fn evce_rdoq_run_length_cc(
    qp: u8,
    d_lambda: f64,
    is_intra: bool,
    coef: &mut [i16],
    log2_cuw: usize,
    log2_cuh: usize,
    ch_type: usize,
    rdoq_est: &EvceRdoqEst,
) -> u16 {
    let qp_rem = qp as usize % 6;
    let ns_shift = if ((log2_cuw + log2_cuh) & 1) != 0 {
        7
    } else {
        0
    };
    let ns_scale = if ((log2_cuw + log2_cuh) & 1) != 0 {
        181
    } else {
        1
    };
    let ns_offset = if ((log2_cuw + log2_cuh) & 1) != 0 {
        (1 << (ns_shift - 1))
    } else {
        0
    };
    let q_value = (quant_scale[qp_rem] * ns_scale + ns_offset) >> ns_shift;
    let log2_size = (log2_cuw + log2_cuh) >> 1;
    let tr_shift = MAX_TX_DYNAMIC_RANGE as isize - BIT_DEPTH as isize - log2_size as isize;
    let max_num_coef = 1 << (log2_cuw + log2_cuh);
    let scan = &evc_scan_tbl[log2_cuw - 1];
    let ctx_last = if ch_type == Y_C { 0 } else { 1 };
    let q_bits = QUANT_SHIFT as isize + tr_shift + (qp as isize / 6);
    let mut nnz = 0;
    let mut sum_all = 0;

    let mut best_last_idx_p1 = 0;
    let mut tmp_coef = [0i16; MAX_TR_DIM];
    let mut tmp_level_double = [0i64; MAX_TR_DIM];
    let mut tmp_dst_coef = [0i16; MAX_TR_DIM];
    let lambda = (d_lambda * (1 << SCALE_BITS) as f64 + 0.5) as i64;
    let err_scale = err_scale_tbl[qp_rem][log2_size - 1];
    let mut d64_best_cost = 0;
    let mut d64_base_cost = 0;
    let mut d64_coded_cost = 0;
    let mut d64_uncoded_cost = 0;
    let mut d64_block_uncoded_cost = 0;

    /* ===== quantization ===== */
    for scan_pos in 0..max_num_coef {
        let blk_pos = scan[scan_pos] as usize;
        let temp_level = coef[blk_pos].abs() as i64 * q_value as i64;
        let level_double = std::cmp::min(temp_level, i32::MAX as i64 - (1i64 << (q_bits - 1)));
        tmp_level_double[blk_pos] = level_double;
        let mut max_abs_level = (level_double >> q_bits) as u32;
        let lower_int =
            (level_double - ((max_abs_level as i64) << q_bits)) < (1i64 << (q_bits - 1));

        if !lower_int {
            max_abs_level += 1;
        }

        let err_val = (level_double * err_scale) >> ERR_SCALE_PRECISION_BITS;
        d64_block_uncoded_cost += err_val * err_val;
        tmp_coef[blk_pos] = if coef[blk_pos] > 0 {
            max_abs_level as i16
        } else {
            -(max_abs_level as i16)
        };
        sum_all += max_abs_level;
    }

    for v in &mut coef[0..max_num_coef] {
        *v = 0;
    }

    if sum_all == 0 {
        return nnz;
    }

    if !is_intra && ch_type == Y_C {
        d64_best_cost = d64_block_uncoded_cost + (rdoq_est.cbf_all[0] * lambda);
        d64_base_cost = d64_block_uncoded_cost + (rdoq_est.cbf_all[1] * lambda);
    } else {
        if ch_type == Y_C {
            d64_best_cost = d64_block_uncoded_cost + (rdoq_est.cbf_luma[0] * lambda);
            d64_base_cost = d64_block_uncoded_cost + (rdoq_est.cbf_luma[1] * lambda);
        } else if ch_type == U_C {
            d64_best_cost = d64_block_uncoded_cost + (rdoq_est.cbf_cb[0] * lambda);
            d64_base_cost = d64_block_uncoded_cost + (rdoq_est.cbf_cb[1] * lambda);
        } else {
            d64_best_cost = d64_block_uncoded_cost + (rdoq_est.cbf_cr[0] * lambda);
            d64_base_cost = d64_block_uncoded_cost + (rdoq_est.cbf_cr[1] * lambda);
        }
    }

    let mut run = 0;
    let mut prev_level = 6;

    for scan_pos in 0..max_num_coef {
        let blk_pos = scan[scan_pos] as usize;
        let ctx_run = if ch_type == Y_C { 0 } else { 2 };
        let ctx_level = if ch_type == Y_C { 0 } else { 2 };

        let level = get_coded_level_rl(
            &mut d64_uncoded_cost,
            &mut d64_coded_cost,
            tmp_level_double[blk_pos],
            tmp_coef[blk_pos].abs() as u32,
            run,
            ctx_run,
            ctx_level,
            q_bits,
            err_scale,
            lambda,
            rdoq_est,
        );
        tmp_dst_coef[blk_pos] = if tmp_coef[blk_pos] < 0 {
            -(level as i16)
        } else {
            level as i16
        };
        d64_base_cost -= d64_uncoded_cost;
        d64_base_cost += d64_coded_cost;

        if level != 0 {
            /* ----- check for last flag ----- */
            let d64_cost_last_zero = (rdoq_est.last[ctx_last][0] as i64 * lambda);
            let d64_cost_last_one = (rdoq_est.last[ctx_last][1] as i64 * lambda);
            let d64_cur_is_last_cost = d64_base_cost + d64_cost_last_one;

            d64_base_cost += d64_cost_last_zero;

            if d64_cur_is_last_cost < d64_best_cost {
                d64_best_cost = d64_cur_is_last_cost;
                best_last_idx_p1 = scan_pos + 1;
            }
            run = 0;
            prev_level = level;
        } else {
            run += 1;
        }
    }

    /* ===== clean uncoded coeficients ===== */
    for scan_pos in 0..max_num_coef {
        let blk_pos = scan[scan_pos] as usize;

        if scan_pos < best_last_idx_p1 {
            if tmp_dst_coef[blk_pos] != 0 {
                nnz += 1;
            }
        } else {
            tmp_dst_coef[blk_pos] = 0;
        }

        coef[blk_pos] = tmp_dst_coef[blk_pos];
    }

    nnz
}

fn evce_quant_nnz(
    qp: u8,
    lambda: f64,
    is_intra: bool,
    coef: &mut [i16],
    log2_cuw: usize,
    log2_cuh: usize,
    scale: u16,
    ch_type: usize,
    slice_type: SliceType,
    rdqo_est: &EvceRdoqEst,
) -> u16 {
    let mut nnz = 0;
    let log2_size = (log2_cuw + log2_cuh) >> 1;
    let ns_shift = if ((log2_cuw + log2_cuh) & 1) != 0 {
        7
    } else {
        0
    };
    let ns_scale = if ((log2_cuw + log2_cuh) & 1) != 0 {
        181
    } else {
        1
    };
    let tr_shift =
        MAX_TX_DYNAMIC_RANGE as isize - BIT_DEPTH as isize - log2_size as isize + ns_shift;
    let shift = QUANT_SHIFT as isize + tr_shift + (qp as isize / 6);
    let cuwxh = (1usize << (log2_cuw + log2_cuh));

    if USE_RDOQ {
        let mut is_coded = false;
        let offset = if slice_type == SliceType::EVC_ST_I {
            FAST_RDOQ_INTRA_RND_OFST
        } else {
            FAST_RDOQ_INTER_RND_OFST
        } << (shift as i64 - 9);

        let zero_coeff_threshold = (1i64 << shift) - offset;

        for i in 0..cuwxh {
            let lev = coef[i].abs() as i64 * scale as i64 * ns_scale;
            if lev >= zero_coeff_threshold {
                is_coded = true;
                break;
            }
        }

        if !is_coded {
            for v in &mut coef[0..cuwxh] {
                *v = 0;
            }
            return nnz;
        }
    }

    if USE_RDOQ {
        nnz = evce_rdoq_run_length_cc(
            qp, lambda, is_intra, coef, log2_cuw, log2_cuh, ch_type, rdqo_est,
        );
    } else {
        let offset = if slice_type == SliceType::EVC_ST_I {
            171
        } else {
            85
        } << (shift as i64 - 9);

        for i in 0..cuwxh {
            let sign = coef[i] < 0;
            let lev = coef[i].abs() as i64 * scale as i64;
            let lev = ((lev * ns_scale + offset) >> shift) as i16;
            coef[i] = if sign { -lev } else { lev };
            nnz += if coef[i] != 0 { 1 } else { 0 };
        }
    }

    return nnz;
}

fn evce_tq_nnz(
    qp: u8,
    lambda: f64,
    coef: &mut [i16],
    log2_cuw: usize,
    log2_cuh: usize,
    scale: u16,
    slice_type: SliceType,
    ch_type: usize,
    is_intra: bool,
    rdqo_est: &EvceRdoqEst,
) -> u16 {
    evce_trans(coef, log2_cuw, log2_cuh);

    return evce_quant_nnz(
        qp, lambda, is_intra, coef, log2_cuw, log2_cuh, scale, ch_type, slice_type, rdqo_est,
    );
}

pub(crate) fn evce_sub_block_tq(
    coef: &mut CUBuffer<i16>,
    log2_cuw: usize,
    log2_cuh: usize,
    qp_y: u8,
    qp_u: u8,
    qp_v: u8,
    slice_type: SliceType,
    nnz: &mut [u16],
    is_intra: bool,
    lambda_y: f64,
    lambda_u: f64,
    lambda_v: f64,
    mut run_stats: u8,
    tree_cons: &TREE_CONS,
    rdqo_est: &EvceRdoqEst,
) -> u16 {
    run_stats = evc_get_run(run_stats, tree_cons);
    let run = [run_stats & 1, (run_stats >> 1) & 1, (run_stats >> 2) & 1];

    let qp = [qp_y, qp_u, qp_v];
    let lambda = [lambda_y, lambda_u, lambda_v];

    for c in 0..N_C {
        if run[c] != 0 {
            let chroma = if c > 0 { 1 } else { 0 };
            let pos_sub_x = 0;
            let pos_sub_y = 0;

            let scale = quant_scale[qp[c as usize] as usize % 6];
            nnz[c] = evce_tq_nnz(
                qp[c],
                lambda[c],
                &mut coef.data[c],
                log2_cuw - chroma,
                log2_cuh - chroma,
                scale,
                slice_type,
                c,
                is_intra,
                rdqo_est,
            );
        } else {
            nnz[c] = 0;
        }
    }

    nnz[Y_C] + nnz[U_C] + nnz[V_C]
}
