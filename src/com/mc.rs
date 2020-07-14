use super::picman::*;
use super::region::*;
use super::util::*;
use super::*;
use crate::api::util::*;
use crate::dec::CUBuffer; //TODO: move CUBuffer to com

use std::cmp::max;

/* padding for store intermediate values, which should be larger than
1+ half of filter tap */
const MC_IBUF_PAD_C: usize = 4;
const MC_IBUF_PAD_L: usize = 8;
const MC_IBUF_PAD_BL: usize = 2;

const MC_PRECISION_ADD: usize = 2;
const MAC_SFT_N0: i32 = (6);
const MAC_ADD_N0: i32 = (1 << 5);
const MAC_SFT_0N: i32 = MAC_SFT_N0;
const MAC_ADD_0N: i32 = MAC_ADD_N0;
const MAC_SFT_NN_S1: i32 = (2);
const MAC_ADD_NN_S1: i32 = (0);
const MAC_SFT_NN_S2: i32 = (10);
const MAC_ADD_NN_S2: i32 = (1 << 9);

#[rustfmt::skip]
static tbl_mc_l_coeff:[[i16;8];4 << MC_PRECISION_ADD] = [
    [  0, 0,   0, 64,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 1,  -5, 52, 20,  -5,  1,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 2, -10, 40, 40, -10,  2,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 1,  -5, 20, 52,  -5,  1,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
    [  0, 0,   0,  0,  0,   0,  0,  0 ],
];

#[rustfmt::skip]
static tbl_mc_c_coeff: [[i16;4];8 << MC_PRECISION_ADD] = [
    [  0, 64,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -2, 58, 10, -2 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -4, 52, 20, -4 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -6, 46, 30, -6 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -8, 40, 40, -8 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -6, 30, 46, -6 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -4, 20, 52, -4 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [ -2, 10, 58, -2 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
    [  0,  0,  0,  0 ],
];

#[inline]
fn MAC_8TAP(
    c: &[i16],
    r0: i16,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    r5: i16,
    r6: i16,
    r7: i16,
) -> i32 {
    c[0] as i32 * r0 as i32
        + c[1] as i32 * r1 as i32
        + c[2] as i32 * r2 as i32
        + c[3] as i32 * r3 as i32
        + c[4] as i32 * r4 as i32
        + c[5] as i32 * r5 as i32
        + c[6] as i32 * r6 as i32
        + c[7] as i32 * r7 as i32
}
#[inline]
fn MAC_8TAP_N0(
    c: &[i16],
    r0: i16,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    r5: i16,
    r6: i16,
    r7: i16,
) -> i32 {
    (MAC_8TAP(c, r0, r1, r2, r3, r4, r5, r6, r7) + MAC_ADD_N0) >> MAC_SFT_N0
}

#[inline]
fn MAC_8TAP_0N(
    c: &[i16],
    r0: i16,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    r5: i16,
    r6: i16,
    r7: i16,
) -> i32 {
    (MAC_8TAP(c, r0, r1, r2, r3, r4, r5, r6, r7) + MAC_ADD_0N) >> MAC_SFT_0N
}

#[inline]
fn MAC_8TAP_NN_S1(
    c: &[i16],
    r0: i16,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    r5: i16,
    r6: i16,
    r7: i16,
) -> i32 {
    (MAC_8TAP(c, r0, r1, r2, r3, r4, r5, r6, r7) + MAC_ADD_NN_S1) >> MAC_SFT_NN_S1
}

#[inline]
fn MAC_8TAP_NN_S2(
    c: &[i16],
    r0: i16,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    r5: i16,
    r6: i16,
    r7: i16,
) -> i32 {
    (MAC_8TAP(c, r0, r1, r2, r3, r4, r5, r6, r7) + MAC_ADD_NN_S2) >> MAC_SFT_NN_S2
}

#[inline]
fn MAC_4TAP(c: &[i16], r0: i16, r1: i16, r2: i16, r3: i16) -> i32 {
    c[0] as i32 * r0 as i32
        + c[1] as i32 * r1 as i32
        + c[2] as i32 * r2 as i32
        + c[3] as i32 * r3 as i32
}

#[inline]
fn MAC_4TAP_N0(c: &[i16], r0: i16, r1: i16, r2: i16, r3: i16) -> i32 {
    (MAC_4TAP(c, r0, r1, r2, r3) + MAC_ADD_N0) >> MAC_SFT_N0
}
#[inline]
fn MAC_4TAP_0N(c: &[i16], r0: i16, r1: i16, r2: i16, r3: i16) -> i32 {
    (MAC_4TAP(c, r0, r1, r2, r3) + MAC_ADD_0N) >> MAC_SFT_0N
}

#[inline]
fn MAC_4TAP_NN_S1(c: &[i16], r0: i16, r1: i16, r2: i16, r3: i16) -> i32 {
    (MAC_4TAP(c, r0, r1, r2, r3) + MAC_ADD_NN_S1) >> MAC_SFT_NN_S1
}

#[inline]
fn MAC_4TAP_NN_S2(c: &[i16], r0: i16, r1: i16, r2: i16, r3: i16) -> i32 {
    (MAC_4TAP(c, r0, r1, r2, r3) + MAC_ADD_NN_S2) >> MAC_SFT_NN_S2
}

fn mv_clip(
    mut x: i16,
    mut y: i16,
    pic_w: i16,
    pic_h: i16,
    mut cuw: i16,
    mut cuh: i16,
    refi: &[i8],
    mv: &[[i16; MV_D]; REFP_NUM],
    mv_t: &mut [[i16; MV_D]; REFP_NUM],
) {
    let mut min_clip = [0i16; MV_D];
    let mut max_clip = [0i16; MV_D];

    x <<= 2;
    y <<= 2;
    cuw <<= 2;
    cuh <<= 2;
    min_clip[MV_X] = -(MAX_CU_SIZE as i16) << 2;
    min_clip[MV_Y] = -(MAX_CU_SIZE as i16) << 2;
    max_clip[MV_X] = (pic_w - 1 + MAX_CU_SIZE as i16) << 2;
    max_clip[MV_Y] = (pic_h - 1 + MAX_CU_SIZE as i16) << 2;

    mv_t[REFP_0][MV_X] = mv[REFP_0][MV_X];
    mv_t[REFP_0][MV_Y] = mv[REFP_0][MV_Y];
    mv_t[REFP_1][MV_X] = mv[REFP_1][MV_X];
    mv_t[REFP_1][MV_Y] = mv[REFP_1][MV_Y];

    if REFI_IS_VALID(refi[REFP_0]) {
        if x + mv[REFP_0][MV_X] < min_clip[MV_X] {
            mv_t[REFP_0][MV_X] = min_clip[MV_X] - x;
        }
        if y + mv[REFP_0][MV_Y] < min_clip[MV_Y] {
            mv_t[REFP_0][MV_Y] = min_clip[MV_Y] - y;
        }
        if x + mv[REFP_0][MV_X] + cuw - 4 > max_clip[MV_X] {
            mv_t[REFP_0][MV_X] = max_clip[MV_X] - x - cuw + 4;
        }
        if y + mv[REFP_0][MV_Y] + cuh - 4 > max_clip[MV_Y] {
            mv_t[REFP_0][MV_Y] = max_clip[MV_Y] - y - cuh + 4;
        }
    }
    if REFI_IS_VALID(refi[REFP_1]) {
        if x + mv[REFP_1][MV_X] < min_clip[MV_X] {
            mv_t[REFP_1][MV_X] = min_clip[MV_X] - x;
        }
        if y + mv[REFP_1][MV_Y] < min_clip[MV_Y] {
            mv_t[REFP_1][MV_Y] = min_clip[MV_Y] - y;
        }
        if x + mv[REFP_1][MV_X] + cuw - 4 > max_clip[MV_X] {
            mv_t[REFP_1][MV_X] = max_clip[MV_X] - x - cuw + 4;
        }
        if y + mv[REFP_1][MV_Y] + cuh - 4 > max_clip[MV_Y] {
            mv_t[REFP_1][MV_Y] = max_clip[MV_Y] - y - cuh + 4;
        }
    }
}

type EVC_MC_FN =
    fn(r: &PlaneRegion<'_, pel>, gmv_x: i16, gmv_y: i16, pred: &mut [pel], cuw: i16, cuh: i16);

static evc_tbl_mc_l: [[EVC_MC_FN; 2]; 2] = [
    [
        evc_mc_l_00, /* dx == 0 && dy == 0 */
        evc_mc_l_0n, /* dx == 0 && dy != 0 */
    ],
    [
        evc_mc_l_n0, /* dx != 0 && dy == 0 */
        evc_mc_l_nn, /* dx != 0 && dy != 0 */
    ],
];

static evc_tbl_mc_c: [[EVC_MC_FN; 2]; 2] = [
    [
        evc_mc_c_00, /* dx == 0 && dy == 0 */
        evc_mc_c_0n, /* dx == 0 && dy != 0 */
    ],
    [
        evc_mc_c_n0, /* dx != 0 && dy == 0 */
        evc_mc_c_nn, /* dx != 0 && dy != 0 */
    ],
];

fn evc_mc_l_00(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let gmv_x = gmv_x >> 4;
    let gmv_y = gmv_y >> 4;

    for y in 0..cuh {
        for x in 0..cuw {
            pred[(y * cuw + x) as usize] = r[(y + gmv_y) as usize][(x + gmv_x) as usize];
        }
    }
}
fn evc_mc_l_n0(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let dx = gmv_x & 15;
    let gmv_x = (gmv_x >> 4) - 3;
    let gmv_y = gmv_y >> 4;

    for y in 0..cuh {
        for x in 0..cuw {
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let rx1 = max(0, x + gmv_x + 1) as usize;
            let rx2 = max(0, x + gmv_x + 2) as usize;
            let rx3 = max(0, x + gmv_x + 3) as usize;
            let rx4 = max(0, x + gmv_x + 4) as usize;
            let rx5 = max(0, x + gmv_x + 5) as usize;
            let rx6 = max(0, x + gmv_x + 6) as usize;
            let rx7 = max(0, x + gmv_x + 7) as usize;

            let pt = MAC_8TAP_N0(
                &tbl_mc_l_coeff[dx as usize],
                r[ry0][rx0] as i16,
                r[ry0][rx1] as i16,
                r[ry0][rx2] as i16,
                r[ry0][rx3] as i16,
                r[ry0][rx4] as i16,
                r[ry0][rx5] as i16,
                r[ry0][rx6] as i16,
                r[ry0][rx7] as i16,
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_l_0n(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let dy = gmv_y & 15;
    let gmv_x = gmv_x >> 4;
    let gmv_y = (gmv_y >> 4) - 3;

    for y in 0..cuh {
        for x in 0..cuw {
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let ry1 = max(0, y + gmv_y + 1) as usize;
            let ry2 = max(0, y + gmv_y + 2) as usize;
            let ry3 = max(0, y + gmv_y + 3) as usize;
            let ry4 = max(0, y + gmv_y + 4) as usize;
            let ry5 = max(0, y + gmv_y + 5) as usize;
            let ry6 = max(0, y + gmv_y + 6) as usize;
            let ry7 = max(0, y + gmv_y + 7) as usize;

            let pt = MAC_8TAP_0N(
                &tbl_mc_l_coeff[dy as usize],
                r[ry0][rx0] as i16,
                r[ry1][rx0] as i16,
                r[ry2][rx0] as i16,
                r[ry3][rx0] as i16,
                r[ry4][rx0] as i16,
                r[ry5][rx0] as i16,
                r[ry6][rx0] as i16,
                r[ry7][rx0] as i16,
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_l_nn(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let mut buf = [0i16; (MAX_CU_SIZE + MC_IBUF_PAD_L) * MAX_CU_SIZE];

    let dx = gmv_x & 15;
    let dy = gmv_y & 15;
    let gmv_x = (gmv_x >> 4) - 3;
    let gmv_y = (gmv_y >> 4) - 3;

    for y in 0..(cuh + 7) {
        for x in 0..cuw {
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let rx1 = max(0, x + gmv_x + 1) as usize;
            let rx2 = max(0, x + gmv_x + 2) as usize;
            let rx3 = max(0, x + gmv_x + 3) as usize;
            let rx4 = max(0, x + gmv_x + 4) as usize;
            let rx5 = max(0, x + gmv_x + 5) as usize;
            let rx6 = max(0, x + gmv_x + 6) as usize;
            let rx7 = max(0, x + gmv_x + 7) as usize;

            buf[(y * cuw + x) as usize] = MAC_8TAP_NN_S1(
                &tbl_mc_l_coeff[dx as usize],
                r[ry0][rx0] as i16,
                r[ry0][rx1] as i16,
                r[ry0][rx2] as i16,
                r[ry0][rx3] as i16,
                r[ry0][rx4] as i16,
                r[ry0][rx5] as i16,
                r[ry0][rx6] as i16,
                r[ry0][rx7] as i16,
            ) as i16;
        }
    }

    for y in 0..cuh {
        for x in 0..cuw {
            let pt = MAC_8TAP_NN_S2(
                &tbl_mc_l_coeff[dy as usize],
                buf[((y + 0) * cuw + x) as usize],
                buf[((y + 1) * cuw + x) as usize],
                buf[((y + 2) * cuw + x) as usize],
                buf[((y + 3) * cuw + x) as usize],
                buf[((y + 4) * cuw + x) as usize],
                buf[((y + 5) * cuw + x) as usize],
                buf[((y + 6) * cuw + x) as usize],
                buf[((y + 7) * cuw + x) as usize],
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_c_00(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let gmv_x = gmv_x >> 5;
    let gmv_y = gmv_y >> 5;

    for y in 0..cuh {
        for x in 0..cuw {
            pred[(y * cuw + x) as usize] = r[(y + gmv_y) as usize][(x + gmv_x) as usize];
        }
    }
}
fn evc_mc_c_n0(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let dx = gmv_x & 31;
    let gmv_x = (gmv_x >> 5) - 1;
    let gmv_y = gmv_y >> 5;

    for y in 0..cuh {
        for x in 0..cuw {
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let rx1 = max(0, x + gmv_x + 1) as usize;
            let rx2 = max(0, x + gmv_x + 2) as usize;
            let rx3 = max(0, x + gmv_x + 3) as usize;

            let pt = MAC_4TAP_N0(
                &tbl_mc_c_coeff[dx as usize],
                r[ry0][rx0] as i16,
                r[ry0][rx1] as i16,
                r[ry0][rx2] as i16,
                r[ry0][rx3] as i16,
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_c_0n(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let dy = gmv_y & 31;
    let gmv_x = gmv_x >> 5;
    let gmv_y = (gmv_y >> 5) - 1;

    for y in 0..cuh {
        for x in 0..cuw {
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let ry1 = max(0, y + gmv_y + 1) as usize;
            let ry2 = max(0, y + gmv_y + 2) as usize;
            let ry3 = max(0, y + gmv_y + 3) as usize;

            let pt = MAC_4TAP_0N(
                &tbl_mc_c_coeff[dy as usize],
                r[ry0][rx0] as i16,
                r[ry1][rx0] as i16,
                r[ry2][rx0] as i16,
                r[ry3][rx0] as i16,
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_c_nn(
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let mut buf = [0i16; ((MAX_CU_SIZE >> 1) + MC_IBUF_PAD_C) * (MAX_CU_SIZE >> 1)];

    let dx = gmv_x & 31;
    let dy = gmv_y & 31;
    let gmv_x = (gmv_x >> 5) - 1;
    let gmv_y = (gmv_y >> 5) - 1;

    for y in 0..(cuh + 3) {
        for x in 0..cuw {
            let ry0 = max(0, y + gmv_y + 0) as usize;
            let rx0 = max(0, x + gmv_x + 0) as usize;
            let rx1 = max(0, x + gmv_x + 1) as usize;
            let rx2 = max(0, x + gmv_x + 2) as usize;
            let rx3 = max(0, x + gmv_x + 3) as usize;

            buf[(y * cuw + x) as usize] = MAC_4TAP_NN_S1(
                &tbl_mc_c_coeff[dx as usize],
                r[ry0][rx0] as i16,
                r[ry0][rx1] as i16,
                r[ry0][rx2] as i16,
                r[ry0][rx3] as i16,
            ) as i16;
        }
    }

    for y in 0..cuh {
        for x in 0..cuw {
            let pt = MAC_4TAP_NN_S2(
                &tbl_mc_c_coeff[dy as usize],
                buf[((y + 0) * cuw + x) as usize],
                buf[((y + 1) * cuw + x) as usize],
                buf[((y + 2) * cuw + x) as usize],
                buf[((y + 3) * cuw + x) as usize],
            );
            pred[(y * cuw + x) as usize] =
                EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, pt) as pel;
        }
    }
}

fn evc_mc_l(
    ori_mv_x: i16,
    ori_mv_y: i16,
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let x =
        (((ori_mv_x) | ((ori_mv_x) >> 1) | ((ori_mv_x) >> 2) | ((ori_mv_x) >> 3)) & 0x1) as usize;
    let y =
        (((ori_mv_y) | ((ori_mv_y) >> 1) | ((ori_mv_y) >> 2) | ((ori_mv_y) >> 3)) & 0x1) as usize;
    evc_tbl_mc_l[x][y](r, gmv_x, gmv_y, pred, cuw, cuh)
}

fn evc_mc_c(
    ori_mv_x: i16,
    ori_mv_y: i16,
    r: &PlaneRegion<'_, pel>,
    gmv_x: i16,
    gmv_y: i16,
    pred: &mut [pel],
    cuw: i16,
    cuh: i16,
) {
    let x = (((ori_mv_x)
        | ((ori_mv_x) >> 1)
        | ((ori_mv_x) >> 2)
        | ((ori_mv_x) >> 3)
        | ((ori_mv_x) >> 4))
        & 0x1) as usize;
    let y = (((ori_mv_y)
        | ((ori_mv_y) >> 1)
        | ((ori_mv_y) >> 2)
        | ((ori_mv_y) >> 3)
        | ((ori_mv_y) >> 4))
        & 0x1) as usize;
    evc_tbl_mc_c[x][y](r, gmv_x, gmv_y, pred, cuw, cuh)
}

pub(crate) fn evc_mc(
    x: i16,
    y: i16,
    pic_w: i16,
    pic_h: i16,
    cuw: i16,
    cuh: i16,
    refi: &[i8],
    mv: &[[i16; MV_D]; REFP_NUM],
    refp: &Vec<Vec<EvcRefP>>,
    pred: &mut [CUBuffer<pel>; 2], //TODO: move CUBuffer to com
    poc_c: i32,
) {
    let mut bidx = 0;
    let mut mv_t = [[0i16; MV_D]; REFP_NUM];

    //store it to pass it to interpolation function for deriving correct interpolation filter
    let mv_before_clipping = [
        [mv[REFP_0][MV_X], mv[REFP_0][MV_Y]],
        [mv[REFP_1][MV_X], mv[REFP_1][MV_Y]],
    ];

    mv_clip(x, y, pic_w, pic_h, cuw, cuh, refi, mv, &mut mv_t);

    if REFI_IS_VALID(refi[REFP_0]) {
        /* forward */
        if let Some(ref_pic) = &refp[refi[REFP_0] as usize][REFP_0].pic {
            let qpel_gmv_x = (x << 2) + mv_t[REFP_0][MV_X];
            let qpel_gmv_y = (y << 2) + mv_t[REFP_0][MV_Y];
            let pic = ref_pic.borrow();
            let planes = &pic.frame.borrow().planes;

            evc_mc_l(
                mv_before_clipping[REFP_0][MV_X] << 2,
                mv_before_clipping[REFP_0][MV_Y] << 2,
                &planes[Y_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[0].data[Y_C],
                cuw,
                cuh,
            );
            evc_mc_c(
                mv_before_clipping[REFP_0][MV_X] << 2,
                mv_before_clipping[REFP_0][MV_Y] << 2,
                &planes[U_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[0].data[U_C],
                cuw >> 1,
                cuh >> 1,
            );
            evc_mc_c(
                mv_before_clipping[REFP_0][MV_X] << 2,
                mv_before_clipping[REFP_0][MV_Y] << 2,
                &planes[V_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[0].data[V_C],
                cuw >> 1,
                cuh >> 1,
            );

            bidx += 1;
        }
    }

    /* check identical motion */
    if REFI_IS_VALID(refi[REFP_0]) && REFI_IS_VALID(refi[REFP_1]) {
        if let (Some(pic0), Some(pic1)) = (
            &refp[refi[REFP_0] as usize][REFP_0].pic,
            &refp[refi[REFP_1] as usize][REFP_1].pic,
        ) {
            if pic0.borrow().poc == pic1.borrow().poc
                && mv_t[REFP_0][MV_X] == mv_t[REFP_1][MV_X]
                && mv_t[REFP_0][MV_Y] == mv_t[REFP_1][MV_Y]
            {
                return;
            }
        }
    }

    if REFI_IS_VALID(refi[REFP_1]) {
        /* backward */
        if let Some(ref_pic) = &refp[refi[REFP_1] as usize][REFP_1].pic {
            let qpel_gmv_x = (x << 2) + mv_t[REFP_1][MV_X];
            let qpel_gmv_y = (y << 2) + mv_t[REFP_1][MV_Y];
            let pic = ref_pic.borrow();
            let planes = &pic.frame.borrow().planes;

            evc_mc_l(
                mv_before_clipping[REFP_1][MV_X] << 2,
                mv_before_clipping[REFP_1][MV_Y] << 2,
                &planes[Y_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[bidx].data[Y_C],
                cuw,
                cuh,
            );
            evc_mc_c(
                mv_before_clipping[REFP_1][MV_X] << 2,
                mv_before_clipping[REFP_1][MV_Y] << 2,
                &planes[U_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[bidx].data[U_C],
                cuw >> 1,
                cuh >> 1,
            );
            evc_mc_c(
                mv_before_clipping[REFP_1][MV_X] << 2,
                mv_before_clipping[REFP_1][MV_Y] << 2,
                &planes[V_C].as_region(),
                (qpel_gmv_x << 2),
                (qpel_gmv_y << 2),
                &mut pred[bidx].data[V_C],
                cuw >> 1,
                cuh >> 1,
            );

            bidx += 1;
        }
    }

    if bidx == 2 {
        let (pred0, pred1) = pred.split_at_mut(1);
        let mut p0 = &mut pred0[0].data[Y_C];
        let mut p1 = &mut pred1[0].data[Y_C];
        for y in 0..cuh {
            for x in 0..cuw {
                let pos = (y * cuw + x) as usize;
                p0[pos] = (p0[pos] + p1[pos] + 1) >> 1;
            }
        }

        let mut p0 = &mut pred0[0].data[U_C];
        let mut p1 = &mut pred1[0].data[U_C];
        for y in 0..cuh >> 1 {
            for x in 0..cuw >> 1 {
                let pos = (y * (cuw >> 1) + x) as usize;
                p0[pos] = (p0[pos] + p1[pos] + 1) >> 1;
            }
        }

        let mut p0 = &mut pred0[0].data[V_C];
        let mut p1 = &mut pred1[0].data[V_C];
        for y in 0..cuh >> 1 {
            for x in 0..cuw >> 1 {
                let pos = (y * (cuw >> 1) + x) as usize;
                p0[pos] = (p0[pos] + p1[pos] + 1) >> 1;
            }
        }
    }
}
