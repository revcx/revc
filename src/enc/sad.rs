use crate::def::*;
use crate::region::*;

fn evc_had_8x8(
    x: usize,
    y: usize,
    u: usize,
    v: usize,
    w: usize,
    h: usize,
    org: &PlaneRegion<'_, pel>,
    cur: &[pel],
) -> u32 {
    let mut satd = 0u32;
    let mut diff = [[0i32; 8]; 8];
    let mut m1 = [[0; 8]; 8];
    let mut m2 = [[0; 8]; 8];
    let mut m3 = [[0; 8]; 8];

    for j in 0..8 {
        for i in 0..8 {
            diff[j][i] = org[y + v + j][x + u + i] as i32 - cur[(v + j) * w + (u + i)] as i32;
        }
    }

    /* horizontal */
    for j in 0..8 {
        m2[j][0] = diff[j][0] + diff[j][4];
        m2[j][1] = diff[j][1] + diff[j][5];
        m2[j][2] = diff[j][2] + diff[j][6];
        m2[j][3] = diff[j][3] + diff[j][7];
        m2[j][4] = diff[j][0] - diff[j][4];
        m2[j][5] = diff[j][1] - diff[j][5];
        m2[j][6] = diff[j][2] - diff[j][6];
        m2[j][7] = diff[j][3] - diff[j][7];

        m1[j][0] = m2[j][0] + m2[j][2];
        m1[j][1] = m2[j][1] + m2[j][3];
        m1[j][2] = m2[j][0] - m2[j][2];
        m1[j][3] = m2[j][1] - m2[j][3];
        m1[j][4] = m2[j][4] + m2[j][6];
        m1[j][5] = m2[j][5] + m2[j][7];
        m1[j][6] = m2[j][4] - m2[j][6];
        m1[j][7] = m2[j][5] - m2[j][7];

        m2[j][0] = m1[j][0] + m1[j][1];
        m2[j][1] = m1[j][0] - m1[j][1];
        m2[j][2] = m1[j][2] + m1[j][3];
        m2[j][3] = m1[j][2] - m1[j][3];
        m2[j][4] = m1[j][4] + m1[j][5];
        m2[j][5] = m1[j][4] - m1[j][5];
        m2[j][6] = m1[j][6] + m1[j][7];
        m2[j][7] = m1[j][6] - m1[j][7];
    }

    /* vertical */
    for i in 0..8 {
        m3[0][i] = m2[0][i] + m2[4][i];
        m3[1][i] = m2[1][i] + m2[5][i];
        m3[2][i] = m2[2][i] + m2[6][i];
        m3[3][i] = m2[3][i] + m2[7][i];
        m3[4][i] = m2[0][i] - m2[4][i];
        m3[5][i] = m2[1][i] - m2[5][i];
        m3[6][i] = m2[2][i] - m2[6][i];
        m3[7][i] = m2[3][i] - m2[7][i];

        m1[0][i] = m3[0][i] + m3[2][i];
        m1[1][i] = m3[1][i] + m3[3][i];
        m1[2][i] = m3[0][i] - m3[2][i];
        m1[3][i] = m3[1][i] - m3[3][i];
        m1[4][i] = m3[4][i] + m3[6][i];
        m1[5][i] = m3[5][i] + m3[7][i];
        m1[6][i] = m3[4][i] - m3[6][i];
        m1[7][i] = m3[5][i] - m3[7][i];

        m2[0][i] = m1[0][i] + m1[1][i];
        m2[1][i] = m1[0][i] - m1[1][i];
        m2[2][i] = m1[2][i] + m1[3][i];
        m2[3][i] = m1[2][i] - m1[3][i];
        m2[4][i] = m1[4][i] + m1[5][i];
        m2[5][i] = m1[4][i] - m1[5][i];
        m2[6][i] = m1[6][i] + m1[7][i];
        m2[7][i] = m1[6][i] - m1[7][i];
    }

    satd += m2[0][0].abs() as u32 >> 2;
    for j in 1..8 {
        satd += m2[0][j].abs() as u32;
    }
    for i in 1..8 {
        for j in 0..8 {
            satd += m2[i][j].abs() as u32;
        }
    }

    satd = ((satd + 2) >> 2);

    satd
}

fn evc_had_4x4(
    x: usize,
    y: usize,
    u: usize,
    v: usize,
    w: usize,
    h: usize,
    org: &PlaneRegion<'_, pel>,
    cur: &[pel],
) -> u32 {
    let mut satd = 0;
    let mut diff = [0i32; 16];
    let mut m = [0; 16];
    let mut d = [0; 16];

    for j in 0..4 {
        for i in 0..4 {
            diff[j * 4 + i] = org[y + v + j][x + u + i] as i32 - cur[(v + j) * w + (u + i)] as i32;
        }
    }

    m[0] = diff[0] + diff[12];
    m[1] = diff[1] + diff[13];
    m[2] = diff[2] + diff[14];
    m[3] = diff[3] + diff[15];
    m[4] = diff[4] + diff[8];
    m[5] = diff[5] + diff[9];
    m[6] = diff[6] + diff[10];
    m[7] = diff[7] + diff[11];
    m[8] = diff[4] - diff[8];
    m[9] = diff[5] - diff[9];
    m[10] = diff[6] - diff[10];
    m[11] = diff[7] - diff[11];
    m[12] = diff[0] - diff[12];
    m[13] = diff[1] - diff[13];
    m[14] = diff[2] - diff[14];
    m[15] = diff[3] - diff[15];

    d[0] = m[0] + m[4];
    d[1] = m[1] + m[5];
    d[2] = m[2] + m[6];
    d[3] = m[3] + m[7];
    d[4] = m[8] + m[12];
    d[5] = m[9] + m[13];
    d[6] = m[10] + m[14];
    d[7] = m[11] + m[15];
    d[8] = m[0] - m[4];
    d[9] = m[1] - m[5];
    d[10] = m[2] - m[6];
    d[11] = m[3] - m[7];
    d[12] = m[12] - m[8];
    d[13] = m[13] - m[9];
    d[14] = m[14] - m[10];
    d[15] = m[15] - m[11];

    m[0] = d[0] + d[3];
    m[1] = d[1] + d[2];
    m[2] = d[1] - d[2];
    m[3] = d[0] - d[3];
    m[4] = d[4] + d[7];
    m[5] = d[5] + d[6];
    m[6] = d[5] - d[6];
    m[7] = d[4] - d[7];
    m[8] = d[8] + d[11];
    m[9] = d[9] + d[10];
    m[10] = d[9] - d[10];
    m[11] = d[8] - d[11];
    m[12] = d[12] + d[15];
    m[13] = d[13] + d[14];
    m[14] = d[13] - d[14];
    m[15] = d[12] - d[15];

    d[0] = m[0] + m[1];
    d[1] = m[0] - m[1];
    d[2] = m[2] + m[3];
    d[3] = m[3] - m[2];
    d[4] = m[4] + m[5];
    d[5] = m[4] - m[5];
    d[6] = m[6] + m[7];
    d[7] = m[7] - m[6];
    d[8] = m[8] + m[9];
    d[9] = m[8] - m[9];
    d[10] = m[10] + m[11];
    d[11] = m[11] - m[10];
    d[12] = m[12] + m[13];
    d[13] = m[12] - m[13];
    d[14] = m[14] + m[15];
    d[15] = m[15] - m[14];

    satd += d[0].abs() as u32 >> 2;
    for k in 1..16 {
        satd += d[k].abs() as u32;
    }
    satd = ((satd + 1) >> 1);

    satd
}

fn evc_had_2x2(
    x: usize,
    y: usize,
    u: usize,
    v: usize,
    w: usize,
    h: usize,
    org: &PlaneRegion<'_, pel>,
    cur: &[pel],
) -> u32 {
    let mut satd = 0;
    let mut diff = [0i32; 4];
    let mut m = [0; 4];

    for j in 0..2 {
        for i in 0..2 {
            diff[j * 2 + i] = org[y + v + j][x + u + i] as i32 - cur[(v + j) * w + (u + i)] as i32;
        }
    }
    m[0] = diff[0] + diff[2];
    m[1] = diff[1] + diff[3];
    m[2] = diff[0] - diff[2];
    m[3] = diff[1] - diff[3];
    satd += ((m[0] + m[1]).abs() as u32 >> 2);
    satd += (m[0] - m[1]).abs() as u32;
    satd += (m[2] + m[3]).abs() as u32;
    satd += (m[2] - m[3]).abs() as u32;

    satd
}

pub(crate) fn evce_satd_16b(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    org: &PlaneRegion<'_, pel>,
    cur: &[pel],
) -> u32 {
    let mut sum = 0u32;
    let mut step = 1;

    if (w % 8 == 0) && (h % 8 == 0) {
        for v in (0..h).step_by(8) {
            for u in (0..w).step_by(8) {
                sum += evc_had_8x8(x, y, u, v, w, h, org, cur);
            }
        }
    } else if (w % 4 == 0) && (h % 4 == 0) {
        for v in (0..h).step_by(4) {
            for u in (0..w).step_by(4) {
                sum += evc_had_4x4(x, y, u, v, w, h, org, cur);
            }
        }
    } else if (w % 2 == 0) && (h % 2 == 0) {
        for v in (0..h).step_by(2) {
            for u in (0..w).step_by(2) {
                sum += evc_had_2x2(x, y, u, v, w, h, org, cur);
            }
        }
    } else {
        assert!(false);
    }

    sum >> (BIT_DEPTH - 8)
}

/* DIFF **********************************************************************/
pub(crate) fn evce_diff_16b(
    x: usize,
    y: usize,
    log_cuw: usize,
    log_cuh: usize,
    src1: &PlaneRegion<'_, pel>,
    src2: &[pel],
    diff: &mut [i16],
) {
    let cuw = 1 << log_cuw;
    let cuh = 1 << log_cuh;
    for j in 0..cuh {
        for i in 0..cuw {
            diff[j * cuw + i] = src1[y + j][x + i] as i16 - src2[j * cuw + i] as i16;
        }
    }
}

/* SSD ***********************************************************************/
pub(crate) fn evce_ssd_16b(
    x: usize,
    y: usize,
    log_cuw: usize,
    log_cuh: usize,
    src1: &PlaneRegion<'_, pel>,
    src2: &[pel],
) -> i64 {
    let shift = (BIT_DEPTH - 8) << 1;
    let mut ssd = 0;
    let cuw = 1 << log_cuw;
    let cuh = 1 << log_cuh;

    for j in 0..cuh {
        for i in 0..cuw {
            let diff = src2[j * cuw + i] as i64 - src1[y + j][x + i] as i64;
            ssd += (diff * diff) >> shift;
        }
    }

    ssd
}
