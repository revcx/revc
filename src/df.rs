use super::def::*;
use super::picman::*;
use super::region::PlaneRegionMut;
use super::tbl::*;
use super::tracer::*;
use super::util::*;

use std::cell::RefCell;
use std::cmp::*;
use std::rc::Rc;

pub(crate) fn evc_deblock(
    sh_qp_u_offset: i8,
    sh_qp_v_offset: i8,
    w_lcu: u16,
    h_lcu: u16,
    w_scu: u16,
    h_scu: u16,
    w: u16,
    h: u16,
    tracer: &mut Option<Tracer>,
    pic: &Option<Rc<RefCell<EvcPic>>>,
    map_scu: &mut [MCU],
    map_split: &[LcuSplitMode],
    map_mv: &Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    map_refi: &Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    evc_tbl_qp_chroma_dynamic_ext: &Vec<Vec<i8>>,
) {
    if let Some(pic) = pic {
        let mut p = pic.borrow_mut();
        p.pic_qp_u_offset = sh_qp_u_offset;
        p.pic_qp_v_offset = sh_qp_v_offset;
    }

    let scu_in_lcu_wh = 1 << (MAX_CU_LOG2 - MIN_CU_LOG2);

    let x_l = 0; //entry point lcu's x location
    let y_l = 0; // entry point lcu's y location
    let x_r = x_l + w_lcu;
    let y_r = y_l + h_lcu;
    let l_scu = x_l * scu_in_lcu_wh;
    let r_scu = EVC_CLIP3(0, w_scu, x_r * scu_in_lcu_wh);
    let t_scu = y_l * scu_in_lcu_wh;
    let b_scu = EVC_CLIP3(0, h_scu, y_r * scu_in_lcu_wh);

    for j in t_scu..b_scu {
        for i in l_scu..r_scu {
            map_scu[(i + j * w_scu) as usize].CLR_COD();
        }
    }

    /* horizontal filtering */
    for j in y_l..y_r {
        for i in x_l..x_r {
            evc_deblock_tree(
                (i << MAX_CU_LOG2),
                (j << MAX_CU_LOG2),
                MAX_CU_SIZE as u16,
                MAX_CU_SIZE as u16,
                0,
                0,
                false, /*horizontal filtering of vertical edge*/
                w_lcu,
                w_scu,
                w,
                h,
                tracer,
                pic,
                map_scu,
                map_split,
                map_mv,
                map_refi,
                evc_tbl_qp_chroma_dynamic_ext,
            );
        }
    }

    for j in t_scu..b_scu {
        for i in l_scu..r_scu {
            map_scu[(i + j * w_scu) as usize].CLR_COD();
        }
    }

    /* vertical filtering */
    for j in y_l..y_r {
        for i in x_l..x_r {
            evc_deblock_tree(
                (i << MAX_CU_LOG2),
                (j << MAX_CU_LOG2),
                MAX_CU_SIZE as u16,
                MAX_CU_SIZE as u16,
                0,
                0,
                true, /*vertical filtering of horizontal edge*/
                w_lcu,
                w_scu,
                w,
                h,
                tracer,
                pic,
                map_scu,
                map_split,
                map_mv,
                map_refi,
                evc_tbl_qp_chroma_dynamic_ext,
            );
        }
    }
}

fn evc_deblock_tree(
    x: u16,
    y: u16,
    cuw: u16,
    cuh: u16,
    cud: u16,
    cup: u16,
    is_hor_edge: bool,
    w_lcu: u16,
    w_scu: u16,
    w: u16,
    h: u16,
    tracer: &mut Option<Tracer>,
    pic: &Option<Rc<RefCell<EvcPic>>>,
    map_scu: &mut [MCU],
    map_split: &[LcuSplitMode],
    map_mv: &Option<Rc<RefCell<Vec<[[i16; MV_D]; REFP_NUM]>>>>,
    map_refi: &Option<Rc<RefCell<Vec<[i8; REFP_NUM]>>>>,
    evc_tbl_qp_chroma_dynamic_ext: &Vec<Vec<i8>>,
) {
    let lcu_num = (x >> MAX_CU_LOG2) + (y >> MAX_CU_LOG2) * w_lcu;
    let split_mode = evc_get_split_mode(
        cud,
        cup,
        cuw,
        cuh,
        MAX_CU_SIZE as u16,
        &map_split[lcu_num as usize],
    );

    /*EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "split_mod ");
    EVC_TRACE(
        tracer,
        if split_mode == SplitMode::NO_SPLIT {
            0
        } else {
            5
        },
    );
    EVC_TRACE(tracer, " \n");*/

    if split_mode != SplitMode::NO_SPLIT {
        let split_struct = evc_split_get_part_structure(
            split_mode,
            x,
            y,
            cuw,
            cuh,
            cup,
            cud,
            (MAX_CU_LOG2 - MIN_CU_LOG2) as u8,
        );

        // In base profile we have small chroma blocks
        for part_num in 0..split_struct.part_count {
            let cur_part_num = part_num;
            let sub_cuw = split_struct.width[cur_part_num];
            let sub_cuh = split_struct.height[cur_part_num];
            let x_pos = split_struct.x_pos[cur_part_num];
            let y_pos = split_struct.y_pos[cur_part_num];

            if x_pos < w && y_pos < h {
                evc_deblock_tree(
                    x_pos,
                    y_pos,
                    sub_cuw,
                    sub_cuh,
                    split_struct.cud[cur_part_num],
                    split_struct.cup[cur_part_num],
                    is_hor_edge,
                    w_lcu,
                    w_scu,
                    w,
                    h,
                    tracer,
                    pic,
                    map_scu,
                    map_split,
                    map_mv,
                    map_refi,
                    evc_tbl_qp_chroma_dynamic_ext,
                );
            }
        }
    } else if let (Some(pic), Some(map_refi), Some(map_mv)) = (pic, map_refi, map_mv) {
        // deblock
        if is_hor_edge {
            if cuh > MAX_TR_SIZE as u16 {
                evc_deblock_cu_hor(
                    tracer,
                    &*pic.borrow(),
                    x as usize,
                    y as usize,
                    cuw as usize,
                    cuh as usize >> 1,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                );

                evc_deblock_cu_hor(
                    tracer,
                    &*pic.borrow(),
                    x as usize,
                    y as usize + MAX_TR_SIZE,
                    cuw as usize,
                    cuh as usize >> 1,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                );
            } else {
                evc_deblock_cu_hor(
                    tracer,
                    &*pic.borrow(),
                    x as usize,
                    y as usize,
                    cuw as usize,
                    cuh as usize,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                );
            }
        } else {
            if cuw > MAX_TR_SIZE as u16 {
                evc_deblock_cu_ver(
                    tracer,
                    &*pic.borrow(),
                    x as usize,
                    y as usize,
                    cuw as usize >> 1,
                    cuh as usize,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                    w as usize,
                );
                evc_deblock_cu_ver(
                    tracer,
                    &*pic.borrow(),
                    x as usize + MAX_TR_SIZE,
                    y as usize,
                    cuw as usize >> 1,
                    cuh as usize,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                    w as usize,
                );
            } else {
                evc_deblock_cu_ver(
                    tracer,
                    &*pic.borrow(),
                    x as usize,
                    y as usize,
                    cuw as usize,
                    cuh as usize,
                    map_scu,
                    &*map_refi.borrow(),
                    &*map_mv.borrow(),
                    w_scu as usize,
                    evc_tbl_qp_chroma_dynamic_ext,
                    w as usize,
                );
            }
        }
    }
}

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
    evc_tbl_qp_chroma_dynamic_ext: &Vec<Vec<i8>>,
) {
    let w = cuw >> MIN_CU_LOG2;
    let h = cuh >> MIN_CU_LOG2;
    let offset = (x_pel >> MIN_CU_LOG2) + (y_pel >> MIN_CU_LOG2) * w_scu;

    /* horizontal filtering */
    if y_pel > 0 {
        let planes = &mut pic.frame.borrow_mut().planes;

        for i in 0..w {
            let tbl_qp_to_st = evc_get_tbl_qp_to_st(
                map_scu[offset + i],
                map_scu[offset + i - w_scu],
                &map_refi[offset + i],
                &map_refi[offset + i - w_scu],
                &map_mv[offset + i],
                &map_mv[offset + i - w_scu],
            );

            let qp = map_scu[offset + i].GET_QP();
            let t = (i << MIN_CU_LOG2);

            deblock_scu_hor(
                tracer,
                &mut planes[Y_C].as_region_mut(),
                qp as usize,
                Y_C,
                tbl_qp_to_st,
                x_pel + t,
                y_pel,
            );

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
    evc_tbl_qp_chroma_dynamic_ext: &Vec<Vec<i8>>,
    pic_w: usize,
) {
    let w = cuw >> MIN_CU_LOG2;
    let h = cuh >> MIN_CU_LOG2;
    let offset = (x_pel >> MIN_CU_LOG2) + (y_pel >> MIN_CU_LOG2) * w_scu;
    let planes = &mut pic.frame.borrow_mut().planes;

    /* vertical filtering */
    if x_pel > 0 && map_scu[offset - 1].GET_COD() != 0 {
        for j in 0..h {
            let tbl_qp_to_st = evc_get_tbl_qp_to_st(
                map_scu[offset + j * w_scu + 0],
                map_scu[offset + j * w_scu - 1],
                &map_refi[offset + j * w_scu + 0],
                &map_refi[offset + j * w_scu - 1],
                &map_mv[offset + j * w_scu + 0],
                &map_mv[offset + j * w_scu - 1],
            );
            let qp = map_scu[offset + j * w_scu + 0].GET_QP();
            let t = (j << MIN_CU_LOG2);

            deblock_scu_ver(
                tracer,
                &mut planes[Y_C].as_region_mut(),
                qp as usize,
                Y_C,
                tbl_qp_to_st,
                x_pel,
                y_pel + t,
            );

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

    if x_pel + cuw < pic_w && map_scu[offset + w].GET_COD() != 0 {
        for j in 0..h {
            let tbl_qp_to_st = evc_get_tbl_qp_to_st(
                map_scu[offset + j * w_scu + w],
                map_scu[offset + j * w_scu + w - 1],
                &map_refi[offset + j * w_scu + w],
                &map_refi[offset + j * w_scu + w - 1],
                &map_mv[offset + j * w_scu + w],
                &map_mv[offset + j * w_scu + w - 1],
            );
            let qp = map_scu[offset + j * w_scu + w].GET_QP();
            let t = (j << MIN_CU_LOG2);

            deblock_scu_ver(
                tracer,
                &mut planes[Y_C].as_region_mut(),
                qp as usize,
                Y_C,
                tbl_qp_to_st,
                x_pel + cuw,
                y_pel + t,
            );

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

    for j in 0..h {
        for i in 0..w {
            map_scu[offset + j * w_scu + i].SET_COD();
        }
    }
}

fn evc_get_tbl_qp_to_st(
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

//TODO: evc_mc_l should be private, but in order to be visible for benchmark,
// change it to pub. Need to figure out a way to hide visible for API caller
pub fn deblock_scu_hor(
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

//TODO: evc_mc_l should be private, but in order to be visible for benchmark,
// change it to pub. Need to figure out a way to hide visible for API caller
pub fn deblock_scu_hor_chroma(
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

//TODO: evc_mc_l should be private, but in order to be visible for benchmark,
// change it to pub. Need to figure out a way to hide visible for API caller
pub fn deblock_scu_ver(
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

            let abs = d.abs();
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

//TODO: evc_mc_l should be private, but in order to be visible for benchmark,
// change it to pub. Need to figure out a way to hide visible for API caller
pub fn deblock_scu_ver_chroma(
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

            let abs = d.abs();
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
