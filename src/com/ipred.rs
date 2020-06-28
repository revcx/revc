use super::tbl::*;
use super::*;

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
