use super::plane_region::*;
use super::tbl::*;
use super::*;
use crate::api::util::*;

pub(crate) fn evc_get_nbr_b(
    x: u16,
    y: u16,
    cuw: u8,
    cuh: u8,
    src: &PlaneRegion<'_, pel>,
    avail_cu: u16,
    nb: &mut [[pel; MAX_CU_SIZE * 3]; N_REF],
    scup: u32,
    map_scu: &[MCU],
    w_scu: u16,
    h_scu: u16,
    ch_type: usize,
    constrained_intra_pred: bool,
) {
    /*
    int  i, j;
    int  scuw = (ch_type == Y_C) ? (cuw >> MIN_CU_LOG2) : (cuw >> (MIN_CU_LOG2 - 1));
    int  scuh = (ch_type == Y_C) ? (cuh >> MIN_CU_LOG2) : (cuh >> (MIN_CU_LOG2 - 1));
    int  unit_size = (ch_type == Y_C) ? MIN_CU_SIZE : (MIN_CU_SIZE >> 1);
    int  x_scu = PEL2SCU(ch_type == Y_C ? x : x << 1);
    int  y_scu = PEL2SCU(ch_type == Y_C ? y : y << 1);
    pel *tmp = src;
    pel *left = nb[ch_type][0] + 2;
    pel *up = nb[ch_type][1] + cuh;

    if (IS_AVAIL(avail_cu, AVAIL_UP_LE) && (!constrained_intra_pred || MCU_GET_IF(map_scu[scup - w_scu - 1])) &&
        (map_tidx[scup] == map_tidx[scup - w_scu - 1]))
    {
        evc_mcpy(up - 1, src - s_src - 1, cuw * sizeof(pel));
    }
    else
    {
        up[-1] = 1 << (BIT_DEPTH - 1);
    }

    for (i = 0; i < (scuw + scuh); i++)
    {
        int is_avail = (y_scu > 0) && (x_scu + i < w_scu);
        if (is_avail && MCU_GET_COD(map_scu[scup - w_scu + i]) && (!constrained_intra_pred || MCU_GET_IF(map_scu[scup - w_scu + i])) &&
            (map_tidx[scup] == map_tidx[scup - w_scu + i]))
        {
            evc_mcpy(up + i * unit_size, src - s_src + i * unit_size, unit_size * sizeof(pel));
        }
        else
        {
            evc_mset_16b(up + i * unit_size, 1 << (BIT_DEPTH - 1), unit_size);
        }
    }

    src--;
    for (i = 0; i < (scuh + scuw); ++i)
    {
        int is_avail = (x_scu > 0) && (y_scu + i < h_scu);
        if (is_avail && MCU_GET_COD(map_scu[scup - 1 + i * w_scu]) && (!constrained_intra_pred || MCU_GET_IF(map_scu[scup - 1 + i * w_scu])) &&
            (map_tidx[scup] == map_tidx[scup - 1 + i * w_scu]))
        {
            for (j = 0; j < unit_size; ++j)
            {
                left[i * unit_size + j] = *src;
                src += s_src;
            }
        }
        else
        {
            evc_mset_16b(left + i * unit_size, 1 << (BIT_DEPTH - 1), unit_size);
            src += (s_src * unit_size);
        }
    }
    left[-1] = up[-1];

     */
}

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
