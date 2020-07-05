use super::tbl::*;
use super::util::*;
use super::*;

pub(crate) fn evc_sub_block_itdq(
    coef: &mut [[i16; MAX_CU_DIM]],
    log2_cuw: u8,
    log2_cuh: u8,
    qp_y: u8,
    qp_u: u8,
    qp_v: u8,
    flag: &[bool],
    nnz_sub: &[[bool; MAX_SUB_TB_NUM]],
) {
    let mut coef_temp_buf = vec![[0i16; MAX_TR_DIM]; N_C];
    let log2_w_sub = if log2_cuw as usize > MAX_TR_LOG2 {
        MAX_TR_LOG2
    } else {
        log2_cuw as usize
    };
    let log2_h_sub = if log2_cuh as usize > MAX_TR_LOG2 {
        MAX_TR_LOG2
    } else {
        log2_cuh as usize
    };
    let loop_w = if log2_cuw as usize > MAX_TR_LOG2 {
        (1 << (log2_cuw as usize - MAX_TR_LOG2))
    } else {
        1
    };
    let loop_h = if log2_cuh as usize > MAX_TR_LOG2 {
        (1 << (log2_cuh as usize - MAX_TR_LOG2))
    } else {
        1
    };
    let stride = 1 << log2_cuw as usize;
    let sub_stride = 1 << log2_w_sub as usize;
    let qp: [u8; N_C] = [qp_y, qp_u, qp_v];
    let mut scale = 0;

    for j in 0..loop_h {
        for i in 0..loop_w {
            for c in 0..N_C {
                let chroma = if c > 1 { 1 } else { 0 };
                if nnz_sub[c][(j << 1) | i] {
                    let pos_sub_x = i * (1 << (log2_w_sub - chroma));
                    let pos_sub_y = j * (1 << (log2_h_sub - chroma)) * (stride >> chroma);

                    let mut coef_temp = if loop_h + loop_w > 2 {
                        evc_block_copy(
                            &coef[c][pos_sub_x + pos_sub_y..],
                            stride >> chroma,
                            &mut coef_temp_buf[c][..],
                            sub_stride >> chroma,
                            (log2_w_sub - chroma) as u8,
                            (log2_h_sub - chroma) as u8,
                        );
                        &coef_temp_buf[c][..]
                    } else {
                        &coef[c][..]
                    };

                    scale = evc_tbl_dq_scale_b[qp[c] as usize % 6] << (qp[c] / 6) as i16;

                    /*
                    evc_itdq(
                        coef_temp,
                        log2_w_sub - chroma,
                        log2_h_sub - chroma,
                        scale,
                        iqt_flag,
                        ats_intra_cu_on,
                        ats_mode_idx,
                    );*/

                    if loop_h + loop_w > 2 {
                        evc_block_copy(
                            &coef_temp_buf[c][..],
                            sub_stride >> chroma,
                            &mut coef[c][pos_sub_x + pos_sub_y..],
                            stride >> chroma,
                            (log2_w_sub - chroma) as u8,
                            (log2_h_sub - chroma) as u8,
                        );
                    }
                }
            }
        }
    }
}

fn evc_itdq(coef: &[i16], log2_w: usize, log2_h: usize, scale: i16) {
    let log2_size = (log2_w + log2_h) >> 1;
    let ns_shift = if (log2_w + log2_h) & 1 != 0 { 8 } else { 0 };

    let tr_shift = MAX_TX_DYNAMIC_RANGE - BIT_DEPTH - log2_size;
    let shift = QUANT_IQUANT_SHIFT - QUANT_SHIFT - tr_shift + ns_shift;
    let offset = if shift == 0 { 0 } else { 1 << (shift - 1) };

    // evc_dquant(coef, log2_w, log2_h, scale, offset, shift);
    // evc_itrans(coef, log2_w, log2_h, iqt_flag);
}
