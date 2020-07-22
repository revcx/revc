use super::def::*;
use super::region::*;
use super::tbl::*;

pub(crate) fn evc_get_mpm_b(
    x_scu: u16,
    y_scu: u16,
    map_scu: &[MCU],
    map_ipm: &[IntraPredDir],
    scup: u32,
    w_scu: u16,
) -> &'static [u8] {
    let mut ipm_l = 0;
    let mut ipm_u = 0;

    if x_scu > 0
        && map_scu[(scup - 1) as usize].GET_IF() != 0
        && map_scu[(scup - 1) as usize].GET_COD() != 0
    {
        ipm_l = (map_ipm[(scup - 1) as usize] as i8 + 1) as usize;
    }
    if y_scu > 0
        && map_scu[(scup - w_scu as u32) as usize].GET_IF() != 0
        && map_scu[(scup - w_scu as u32) as usize].GET_COD() != 0
    {
        ipm_u = (map_ipm[(scup - w_scu as u32) as usize] as i8 + 1) as usize;
    }

    &evey_tbl_mpm[ipm_l as usize][ipm_u as usize]
}

pub(crate) fn evc_get_nbr_b(
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
    src: &PlaneRegion<'_, pel>,
    avail_cu: u16,
    nb: &mut Vec<Vec<pel>>, //[[pel; MAX_CU_SIZE * 3]; N_REF],
    scup: usize,
    map_scu: &[MCU],
    w_scu: usize,
    h_scu: usize,
    ch_type: usize,
    constrained_intra_pred: bool,
) {
    let scuw = if ch_type == Y_C {
        cuw >> MIN_CU_LOG2
    } else {
        cuw >> (MIN_CU_LOG2 - 1)
    };
    let scuh = if ch_type == Y_C {
        cuh >> MIN_CU_LOG2
    } else {
        cuh >> (MIN_CU_LOG2 - 1)
    };
    let unit_size = if ch_type == Y_C {
        MIN_CU_SIZE
    } else {
        MIN_CU_SIZE >> 1
    };
    let x_scu = PEL2SCU(if ch_type == Y_C { x } else { x << 1 });
    let y_scu = PEL2SCU(if ch_type == Y_C { y } else { y << 1 });

    {
        let up_left = &mut nb[1][cuh - 1..];
        if IS_AVAIL(avail_cu, AVAIL_UP_LE)
            && (!constrained_intra_pred || map_scu[scup - w_scu - 1].GET_IF() != 0)
        {
            //evc_mcpy(up - 1, src - s_src - 1, cuw * sizeof(pel));
            up_left[0..cuw].copy_from_slice(&src[y - 1][x - 1..x - 1 + cuw]);
        } else {
            up_left[0] = (1 << (BIT_DEPTH - 1)) as pel;
        }
    }

    {
        let up = &mut nb[1][cuh..];
        for i in 0..(scuw + scuh) {
            let is_avail = (y_scu > 0) && (x_scu + i < w_scu);
            if is_avail
                && map_scu[scup - w_scu + i].GET_COD() != 0
                && (!constrained_intra_pred || map_scu[scup - w_scu + i].GET_IF() != 0)
            {
                up[i * unit_size..(i + 1) * unit_size]
                    .copy_from_slice(&src[y - 1][x + i * unit_size..x + (i + 1) * unit_size]);
            } else {
                for v in up[i * unit_size..(i + 1) * unit_size].iter_mut() {
                    *v = 1 << (BIT_DEPTH - 1) as pel;
                }
            }
        }
    }

    {
        let left = &mut nb[0][2..];
        for i in 0..(scuh + scuw) {
            let is_avail = (x_scu > 0) && (y_scu + i < h_scu);
            if is_avail
                && map_scu[scup - 1 + i * w_scu].GET_COD() != 0
                && (!constrained_intra_pred || map_scu[scup - 1 + i * w_scu].GET_IF() != 0)
            {
                for j in 0..unit_size {
                    left[i * unit_size + j] = src[y + i * unit_size + j][x - 1];
                }
            } else {
                for v in left[i * unit_size..(i + 1) * unit_size].iter_mut() {
                    *v = 1 << (BIT_DEPTH - 1) as pel;
                }
            }
        }
    }

    {
        //left[-1] = up[-1];
        nb[0][1] = nb[1][cuh - 1];
    }
}

/* intra prediction for baseline profile */
pub(crate) fn evc_ipred_b(
    src_le: &[pel],
    src_up: &[pel],
    src_tl: pel,
    dst: &mut [pel],
    ipm: IntraPredDir,
    cuw: usize,
    cuh: usize,
) {
    match ipm {
        IntraPredDir::IPD_VER_B => ipred_vert(src_up, dst, cuw, cuh),
        IntraPredDir::IPD_HOR_B => ipred_hor_b(src_le, dst, cuw, cuh),
        IntraPredDir::IPD_DC_B => ipred_dc_b(src_le, src_up, dst, cuw, cuh),
        IntraPredDir::IPD_UL_B => ipred_ul(src_le, src_up, src_tl, dst, cuw, cuh),
        IntraPredDir::IPD_UR_B => ipred_ur(src_le, src_up, dst, cuw, cuh),
        _ => print!("\n illegal intra prediction mode\n"),
    }
}

fn ipred_vert(src_up: &[pel], dst: &mut [pel], w: usize, h: usize) {
    for i in 0..h {
        dst[i * w..(i + 1) * w].copy_from_slice(&src_up[0..w]);
    }
}

fn ipred_hor_b(src_le: &[pel], dst: &mut [pel], w: usize, h: usize) {
    for i in 0..h {
        for v in dst[i * w..(i + 1) * w].iter_mut() {
            *v = src_le[i];
        }
    }
}

fn ipred_dc_b(src_le: &[pel], src_up: &[pel], dst: &mut [pel], w: usize, h: usize) {
    let mut dc = 0;
    for i in 0..h {
        dc += src_le[i];
    }
    for j in 0..w {
        dc += src_up[j];
    }
    dc = (dc + w as pel) >> (evc_tbl_log2[w] + 1) as pel;

    for v in dst[..w * h].iter_mut() {
        *v = dc;
    }
}

fn ipred_ul(src_le: &[pel], src_up: &[pel], src_tl: pel, dst: &mut [pel], w: usize, h: usize) {
    for i in 0..h {
        for j in 0..w {
            let pos = i * w + j;
            let diag = i as isize - j as isize;
            if diag > 0 {
                dst[pos] = src_le[diag as usize - 1];
            } else if diag == 0 {
                dst[pos] = src_tl;
            } else {
                dst[pos] = src_up[(-diag - 1) as usize];
            }
        }
    }
}

fn ipred_ur(src_le: &[pel], src_up: &[pel], dst: &mut [pel], w: usize, h: usize) {
    for i in 0..h {
        for j in 0..w {
            let pos = i * w + j;
            dst[pos] = (src_up[i + j + 1] + src_le[i + j + 1]) >> 1;
        }
    }
}
