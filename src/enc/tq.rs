use super::util::*;
use crate::api::*;
use crate::def::*;
use crate::util::*;

const TX_SHIFT1: usize = BIT_DEPTH - 8 - 1;
const TX_SHIFT2: usize = 6;
const quant_scale: [u16; 6] = [26214, 23302, 20560, 18396, 16384, 14564];

fn evc_get_transform_shift(log2_size: usize, typ: u8) -> usize {
    if typ == 0 {
        TX_SHIFT1 + log2_size
    } else {
        TX_SHIFT2 + log2_size
    }
}

fn evce_trans(coef: &[i16], log2_cuw: usize, log2_cuh: usize) {
    let shift1 = evc_get_transform_shift(log2_cuw, 0);
    let shift2 = evc_get_transform_shift(log2_cuh, 1);

    let mut tb = [0; MAX_TR_DIM]; /* temp buffer */
    //evce_tbl_txb[log2_cuw - 1](coef, tb, 0, 1 << log2_cuh, 0);
    //evce_tbl_txb[log2_cuh - 1](tb, coef, (shift1 + shift2), 1 << log2_cuw, 1);
}

fn evce_tq_nnz(
    qp: u8,
    lambda: f64,
    coef: &[i16],
    log2_cuw: usize,
    log2_cuh: usize,
    scale: u16,
    slice_type: SliceType,
    ch_type: usize,
    is_intra: bool,
) -> u16 {
    evce_trans(coef, log2_cuw, log2_cuh);

    return 0; /* evce_quant_nnz(
                  qp, lambda, is_intra, coef, log2_cuw, log2_cuh, scale, ch_type, slice_type,
              );*/
}

pub(crate) fn evce_sub_block_tq(
    coef: &CUBuffer<i16>,
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
                &coef.data[c],
                log2_cuw - chroma,
                log2_cuh - chroma,
                scale,
                slice_type,
                c,
                is_intra,
            );
        } else {
            nnz[c] = 0;
        }
    }

    nnz[Y_C] + nnz[U_C] + nnz[V_C]
}
