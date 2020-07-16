use super::picman::*;
use super::tbl::*;
use super::tracer::*;
use super::util::*;
use super::*;
use crate::com::region::PlaneRegionMut;

use std::cmp::*;

pub(crate) fn evc_deblock_cu_hor(
    tracer: &mut Option<Tracer>,
    pic: &EvcPic,
    x_pel: usize,
    y_pel: usize,
    cuw: usize,
    cuh: usize,
    map_scu: &mut [MCU],
    map_refi: &Vec<[i8; REFP_NUM]>,
    map_mv: &Vec<[[i16; MV_D]; REFP_NUM]>,
    w_scu: usize,
    tree_cons: &TREE_CONS,
    evc_tbl_qp_chroma_dynamic_ext: &[Vec<i8>; 2],
) {
    let w = cuw >> MIN_CU_LOG2;
    let h = cuh >> MIN_CU_LOG2;
    let offset = (x_pel >> MIN_CU_LOG2) + (y_pel >> MIN_CU_LOG2) * w_scu;

    /* horizontal filtering */
    if y_pel > 0 {
        let planes = &mut pic.frame.borrow_mut().planes;

        for i in 0..w {
            let tbl_qp_to_st = get_tbl_qp_to_st(
                map_scu[offset + i],
                map_scu[offset + i - w_scu],
                &map_refi[offset + i],
                &map_refi[offset + i - w_scu],
                &map_mv[offset + i],
                &map_mv[offset + i - w_scu],
            );

            let qp = map_scu[offset + i].GET_QP();
            let t = (i << MIN_CU_LOG2);

            if evc_check_luma(tree_cons) {
                deblock_scu_hor(
                    tracer,
                    &mut planes[Y_C].as_region_mut(),
                    qp as usize,
                    Y_C,
                    tbl_qp_to_st,
                    x_pel + t,
                    y_pel,
                );
            }

            if evc_check_chroma(tree_cons) {
                let qp_u = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_u_offset,
                );
                let qp_v = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_v_offset,
                );

                deblock_scu_hor_chroma(
                    tracer,
                    &mut planes[U_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[0][(EVC_TBL_CHROMA_QP_OFFSET + qp_u) as usize]
                        as usize,
                    U_C,
                    tbl_qp_to_st,
                    (x_pel + t) >> 1,
                    y_pel >> 1,
                );
                deblock_scu_hor_chroma(
                    tracer,
                    &mut planes[V_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[1][(EVC_TBL_CHROMA_QP_OFFSET + qp_v) as usize]
                        as usize,
                    V_C,
                    tbl_qp_to_st,
                    (x_pel + t) >> 1,
                    y_pel >> 1,
                );
            }
        }
    }

    for j in 0..h {
        for i in 0..w {
            map_scu[offset + j * w_scu + i].SET_COD();
        }
    }
}

pub(crate) fn evc_deblock_cu_ver(
    tracer: &mut Option<Tracer>,
    pic: &EvcPic,
    x_pel: usize,
    y_pel: usize,
    cuw: usize,
    cuh: usize,
    map_scu: &mut [MCU],
    map_refi: &Vec<[i8; REFP_NUM]>,
    map_mv: &Vec<[[i16; MV_D]; REFP_NUM]>,
    w_scu: usize,
    tree_cons: &TREE_CONS,
    evc_tbl_qp_chroma_dynamic_ext: &[Vec<i8>; 2],
    pic_w: usize,
) {
    let w = cuw >> MIN_CU_LOG2;
    let h = cuh >> MIN_CU_LOG2;
    let offset = (x_pel >> MIN_CU_LOG2) + (y_pel >> MIN_CU_LOG2) * w_scu;
    let planes = &mut pic.frame.borrow_mut().planes;

    /* vertical filtering */
    if x_pel > 0 && map_scu[offset - 1].GET_COD() != 0 {
        for j in 0..h {
            let tbl_qp_to_st = get_tbl_qp_to_st(
                map_scu[offset + 0],
                map_scu[offset - 1],
                &map_refi[offset + 0],
                &map_refi[offset - 1],
                &map_mv[offset + 0],
                &map_mv[offset - 1],
            );
            let qp = map_scu[offset + 0].GET_QP();
            let t = (j << MIN_CU_LOG2);

            if evc_check_luma(tree_cons) {
                deblock_scu_ver(
                    tracer,
                    &mut planes[Y_C].as_region_mut(),
                    qp as usize,
                    Y_C,
                    tbl_qp_to_st,
                    x_pel,
                    y_pel + t,
                );
            }

            if evc_check_chroma(tree_cons) {
                let qp_u = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_u_offset,
                );
                let qp_v = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_v_offset,
                );
                deblock_scu_ver_chroma(
                    tracer,
                    &mut planes[U_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[0][(EVC_TBL_CHROMA_QP_OFFSET + qp_u) as usize]
                        as usize,
                    U_C,
                    tbl_qp_to_st,
                    x_pel >> 1,
                    (y_pel + t) >> 1,
                );
                deblock_scu_ver_chroma(
                    tracer,
                    &mut planes[V_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[0][(EVC_TBL_CHROMA_QP_OFFSET + qp_v) as usize]
                        as usize,
                    V_C,
                    tbl_qp_to_st,
                    x_pel >> 1,
                    (y_pel + t) >> 1,
                );
            }
        }
    }

    if x_pel + cuw < pic_w && map_scu[offset + w].GET_COD() != 0 {
        for j in 0..h {
            let tbl_qp_to_st = get_tbl_qp_to_st(
                map_scu[offset + w],
                map_scu[offset + w - 1],
                &map_refi[offset + w],
                &map_refi[offset + w - 1],
                &map_mv[offset + w],
                &map_mv[offset + w - 1],
            );
            let qp = map_scu[offset + w].GET_QP();
            let t = (j << MIN_CU_LOG2);

            if evc_check_luma(tree_cons) {
                deblock_scu_ver(
                    tracer,
                    &mut planes[Y_C].as_region_mut(),
                    qp as usize,
                    Y_C,
                    tbl_qp_to_st,
                    x_pel + cuw,
                    y_pel + t,
                );
            }
            if evc_check_chroma(tree_cons) {
                let qp_u = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_u_offset,
                );
                let qp_v = EVC_CLIP3(
                    -6 * (BIT_DEPTH as i8 - 8),
                    57,
                    qp as i8 + pic.pic_qp_v_offset,
                );
                deblock_scu_ver_chroma(
                    tracer,
                    &mut planes[U_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[0][(EVC_TBL_CHROMA_QP_OFFSET + qp_u) as usize]
                        as usize,
                    U_C,
                    tbl_qp_to_st,
                    (x_pel + cuw) >> 1,
                    (y_pel + t) >> 1,
                );
                deblock_scu_ver_chroma(
                    tracer,
                    &mut planes[V_C].as_region_mut(),
                    evc_tbl_qp_chroma_dynamic_ext[0][(EVC_TBL_CHROMA_QP_OFFSET + qp_v) as usize]
                        as usize,
                    V_C,
                    tbl_qp_to_st,
                    (x_pel + cuw) >> 1,
                    (y_pel + t) >> 1,
                );
            }
        }
    }

    for j in 0..h {
        for i in 0..w {
            map_scu[offset + j * w_scu + i].SET_COD();
        }
    }
}

fn get_tbl_qp_to_st(
    mcu0: MCU,
    mcu1: MCU,
    refi0: &[i8],
    refi1: &[i8],
    mv0: &[[i16; MV_D]; REFP_NUM],
    mv1: &[[i16; MV_D]; REFP_NUM],
) -> &'static [u8] {
    let mut idx = 3;

    if mcu0.GET_IF() != 0 || mcu1.GET_IF() != 0 {
        idx = 0;
    } else if mcu0.GET_CBFL() == 1 || mcu1.GET_CBFL() == 1 {
        idx = 1;
    //} //else if mcu0.GET_IBC() || mcu1.GET_IBC() {
    //    idx = 2;
    } else {
        let mut mv0_l0 = [mv0[REFP_0][MV_X], mv0[REFP_0][MV_Y]];
        let mut mv0_l1 = [mv0[REFP_1][MV_X], mv0[REFP_1][MV_Y]];
        let mut mv1_l0 = [mv1[REFP_0][MV_X], mv1[REFP_0][MV_Y]];
        let mut mv1_l1 = [mv1[REFP_1][MV_X], mv1[REFP_1][MV_Y]];

        if !REFI_IS_VALID(refi0[REFP_0]) {
            mv0_l0[0] = 0;
            mv0_l0[1] = 0;
        }

        if !REFI_IS_VALID(refi0[REFP_1]) {
            mv0_l1[0] = 0;
            mv0_l1[1] = 0;
        }

        if !REFI_IS_VALID(refi1[REFP_0]) {
            mv1_l0[0] = 0;
            mv1_l0[1] = 0;
        }

        if !REFI_IS_VALID(refi1[REFP_1]) {
            mv1_l1[0] = 0;
            mv1_l1[1] = 0;
        }

        if (refi0[REFP_0] == refi1[REFP_0]) && (refi0[REFP_1] == refi1[REFP_1]) {
            idx = if (mv0_l0[MV_X] - mv1_l0[MV_X]).abs() >= 4
                || (mv0_l0[MV_Y] - mv1_l0[MV_Y]).abs() >= 4
                || (mv0_l1[MV_X] - mv1_l1[MV_X]).abs() >= 4
                || (mv0_l1[MV_Y] - mv1_l1[MV_Y]).abs() >= 4
            {
                2
            } else {
                3
            };
        } else if (refi0[REFP_0] == refi1[REFP_1]) && (refi0[REFP_1] == refi1[REFP_0]) {
            idx = if (mv0_l0[MV_X] - mv1_l1[MV_X]).abs() >= 4
                || (mv0_l0[MV_Y] - mv1_l1[MV_Y]).abs() >= 4
                || (mv0_l1[MV_X] - mv1_l0[MV_X]).abs() >= 4
                || (mv0_l1[MV_Y] - mv1_l0[MV_Y]).abs() >= 4
            {
                2
            } else {
                3
            };
        } else {
            idx = 2;
        }
    }

    return &evc_tbl_df_st[idx];
}

fn deblock_scu_hor(
    tracer: &mut Option<Tracer>,
    buf: &mut PlaneRegionMut<'_, pel>,
    qp: usize,
    ch_type: usize,
    tbl_qp_to_st: &[u8],
    x: usize,
    y: usize,
) {
    let st = (tbl_qp_to_st[qp] as i16) << (BIT_DEPTH as i16 - 8);
    let size = if ch_type == Y_C {
        MIN_CU_SIZE
    } else {
        MIN_CU_SIZE >> 1
    };

    if st != 0 {
        for i in 0..size {
            let mut A = buf[y - 2][x + i] as i16;
            let mut B = buf[y - 1][x + i] as i16;
            let mut C = buf[y + 0][x + i] as i16;
            let mut D = buf[y + 1][x + i] as i16;

            let d = (A - (B << 2) + (C << 2) - D) / 8;

            let abs: i16 = d.abs();
            let sign = d < 0;

            let t16 = max(0, ((abs - st) << 1));
            let mut clip = max(0, (abs - t16));
            let d1 = if sign { -clip } else { clip };
            clip >>= 1;
            let d2 = EVC_CLIP3(-clip, clip, ((A - D) / 4));

            A -= d2;
            B += d1;
            C -= d1;
            D += d2;

            buf[y - 2][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, A) as pel;
            buf[y - 1][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, B) as pel;
            buf[y + 0][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, C) as pel;
            buf[y + 1][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, D) as pel;
        }
        TRACE_DBF(tracer, ch_type, x, y, size, true, buf);
    }
}

fn deblock_scu_hor_chroma(
    tracer: &mut Option<Tracer>,
    buf: &mut PlaneRegionMut<'_, pel>,
    qp: usize,
    ch_type: usize,
    tbl_qp_to_st: &[u8],
    x: usize,
    y: usize,
) {
    let st = (tbl_qp_to_st[qp] as i16) << (BIT_DEPTH as i16 - 8);
    let size = if ch_type == Y_C {
        MIN_CU_SIZE
    } else {
        MIN_CU_SIZE >> 1
    };

    if st != 0 {
        for i in 0..size {
            let mut A = buf[y - 2][x + i] as i16;
            let mut B = buf[y - 1][x + i] as i16;
            let mut C = buf[y + 0][x + i] as i16;
            let mut D = buf[y + 1][x + i] as i16;

            let d = (A - (B << 2) + (C << 2) - D) / 8;

            let abs = d.abs();
            let sign = d < 0;

            let t16 = max(0, ((abs - st) << 1));
            let clip = max(0, (abs - t16));
            let d1 = if sign { -clip } else { clip };

            B += d1;
            C -= d1;

            buf[y - 1][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, B) as pel;
            buf[y + 0][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, C) as pel;
        }
        TRACE_DBF(tracer, ch_type, x, y, size, true, buf);
    }
}

fn deblock_scu_ver(
    tracer: &mut Option<Tracer>,
    buf: &mut PlaneRegionMut<'_, pel>,
    qp: usize,
    ch_type: usize,
    tbl_qp_to_st: &[u8],
    x: usize,
    y: usize,
) {
    let st = (tbl_qp_to_st[qp] as i16) << (BIT_DEPTH as i16 - 8);
    let size = if ch_type == Y_C {
        MIN_CU_SIZE
    } else {
        MIN_CU_SIZE >> 1
    };

    if st != 0 {
        for j in 0..size {
            let mut A = buf[y + j][x - 2] as i16;
            let mut B = buf[y + j][x - 1] as i16;
            let mut C = buf[y + j][x + 0] as i16;
            let mut D = buf[y + j][x + 1] as i16;

            let d = (A - (B << 2) + (C << 2) - D) / 8;

            let abs: i16 = d.abs();
            let sign = d < 0;

            let t16 = max(0, ((abs - st) << 1));
            let mut clip = max(0, (abs - t16));
            let d1 = if sign { -clip } else { clip };
            clip >>= 1;
            let d2 = EVC_CLIP3(-clip, clip, ((A - D) / 4));

            A -= d2;
            B += d1;
            C -= d1;
            D += d2;

            buf[y + j][x - 2] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, A) as pel;
            buf[y + j][x - 1] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, B) as pel;
            buf[y + j][x + 0] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, C) as pel;
            buf[y + j][x + 1] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, D) as pel;
        }
        TRACE_DBF(tracer, ch_type, x, y, size, false, buf);
    }
}

fn deblock_scu_ver_chroma(
    tracer: &mut Option<Tracer>,
    buf: &mut PlaneRegionMut<'_, pel>,
    qp: usize,
    ch_type: usize,
    tbl_qp_to_st: &[u8],
    x: usize,
    y: usize,
) {
    let st = (tbl_qp_to_st[qp] as i16) << (BIT_DEPTH as i16 - 8);
    let size = if ch_type == Y_C {
        MIN_CU_SIZE
    } else {
        MIN_CU_SIZE >> 1
    };

    if st != 0 {
        for j in 0..size {
            let mut A = buf[y + j][x - 2] as i16;
            let mut B = buf[y + j][x - 1] as i16;
            let mut C = buf[y + j][x + 0] as i16;
            let mut D = buf[y + j][x + 1] as i16;

            let d = (A - (B << 2) + (C << 2) - D) / 8;

            let abs: i16 = d.abs();
            let sign = d < 0;

            let t16 = max(0, ((abs - st) << 1));
            let clip = max(0, (abs - t16));
            let d1 = if sign { -clip } else { clip };

            B += d1;
            C -= d1;

            buf[y + j][x - 1] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, B) as pel;
            buf[y + j][x + 0] = EVC_CLIP3(0, (1 << BIT_DEPTH as i16) - 1, C) as pel;
        }
        TRACE_DBF(tracer, ch_type, x, y, size, false, buf);
    }
}
