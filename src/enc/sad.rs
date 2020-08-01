use crate::def::*;
use crate::region::*;

pub(crate) fn evce_satd_16b(w: usize, h: usize, org: &PlaneRegion<'_, pel>, cur: &[pel]) -> u32 {
    let mut sum = 0u32;
    let mut step = 1;
    /*
        if w > h && (h & 7) == 0 && (w & 15) == 0 {
            int  offset_org = s_org << 3;
            int  offset_cur = s_cur << 3;

            for(y = 0; y < h; y += 8)
            {
                for(x = 0; x < w; x += 16)
                {
                    sum += evc_had_16x8(&org[x], &cur[x], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if(w < h && (w & 7) == 0 && (h & 15) == 0)
        {
            int  offset_org = s_org << 4;
            int  offset_cur = s_cur << 4;

            for(y = 0; y < h; y += 16)
            {
                for(x = 0; x < w; x += 8)
                {
                    sum += evc_had_8x16(&org[x], &cur[x], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if(w > h && (h & 3) == 0 && (w & 7) == 0)
        {
            int  offset_org = s_org << 2;
            int  offset_cur = s_cur << 2;

            for(y = 0; y < h; y += 4)
            {
                for(x = 0; x < w; x += 8)
                {
                    sum += evc_had_8x4(&org[x], &cur[x], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if(w < h && (w & 3) == 0 && (h & 7) == 0)
        {
            int  offset_org = s_org << 3;
            int  offset_cur = s_cur << 3;

            for(y = 0; y < h; y += 8)
            {
                for(x = 0; x < w; x += 4)
                {
                    sum += evc_had_4x8(&org[x], &cur[x], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if((w % 8 == 0) && (h % 8 == 0))
        {
            int  offset_org = s_org << 3;
            int  offset_cur = s_cur << 3;

            for(y = 0; y < h; y += 8)
            {
                for(x = 0; x < w; x += 8)
                {
                    sum += evc_had_8x8(&org[x], &cur[x*step], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if((w % 4 == 0) && (h % 4 == 0))
        {
            int  offset_org = s_org << 2;
            int  offset_cur = s_cur << 2;

            for(y = 0; y < h; y += 4)
            {
                for(x = 0; x < w; x += 4)
                {
                    sum += evc_had_4x4(&org[x], &cur[x*step], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else if((w % 2 == 0) && (h % 2 == 0) )
        {
            int  offset_org = s_org << 1;
            int  offset_cur = s_cur << 1;

            for(y = 0; y < h; y +=2)
            {
                for(x = 0; x < w; x += 2)
                {
                    sum += evc_had_2x2(&org[x], &cur[x*step], s_org, s_cur, step);
                }
                org += offset_org;
                cur += offset_cur;
            }
        }
        else
        {
            evc_assert(0);
        }
    */

    sum >> (BIT_DEPTH - 8)
}

/* DIFF **********************************************************************/
pub(crate) fn evce_diff_16b(
    w: usize,
    h: usize,
    src1: &PlaneRegion<'_, pel>,
    src2: &[pel],
    diff: &mut [i16],
) {
    for j in 0..h {
        for i in 0..w {
            diff[j * w + h] = src1[j][i] as i16 - src2[j * w + i] as i16;
        }
    }
}
