use super::pinter::*;
use super::*;
use crate::def::*;

const MAX_FIRST_SEARCH_STEP: i16 = 3;
const MAX_REFINE_SEARCH_STEP: i16 = 2;
const RASTER_SEARCH_STEP: i16 = 5;
const RASTER_SEARCH_THD: i16 = 5;
const REFINE_SEARCH_THD: i16 = 0;
const BI_STEP: i16 = 5;

impl EvcePInter {
    pub(crate) fn pinter_me_epzs(
        &mut self,
        x: i16,
        y: i16,
        log2_cuw: usize,
        log2_cuh: usize,
        refi: i8,
        lidx: usize,
        mvp: &[i16],
        mv: &mut [i16],
        bi: u8,
    ) -> u32 {
        let mut mvc = [0i16; MV_D]; /* MV center for search */
        let mut gmvp = [0i16; MV_D]; /* MVP in frame cordinate */
        let mut range = [[0i16; MV_D]; MV_RANGE_DIM]; /* search range after clipping */
        let mut mvi = [0i16; MV_D];
        let mut mvt = [0i16; MV_D];
        let mut cost = std::u32::MAX;
        let mut cost_best = std::u32::MAX;
        let ri = refi; /* reference buffer index */
        let mut tmpstep = 0;
        let mut beststep = 0;

        gmvp[MV_X] = mvp[MV_X] + (x << 2);
        gmvp[MV_Y] = mvp[MV_Y] + (y << 2);

        if bi == BI_NORMAL {
            mvi[MV_X] = mv[MV_X] + (x << 2);
            mvi[MV_Y] = mv[MV_Y] + (y << 2);
            mvc[MV_X] = x + (mv[MV_X] >> 2);
            mvc[MV_Y] = y + (mv[MV_Y] >> 2);
        } else {
            mvi[MV_X] = mvp[MV_X] + (x << 2);
            mvi[MV_Y] = mvp[MV_Y] + (y << 2);
            mvc[MV_X] = x + (mvp[MV_X] >> 2);
            mvc[MV_Y] = y + (mvp[MV_Y] >> 2);
        }

        mvc[MV_X] = EVC_CLIP3(self.min_clip[MV_X], self.max_clip[MV_X], mvc[MV_X]);
        mvc[MV_Y] = EVC_CLIP3(self.min_clip[MV_Y], self.max_clip[MV_Y], mvc[MV_Y]);
        //TODO:
        /*get_range_ipel(
            pi,
            mvc,
            range,
            if bi != BI_NORMAL { 0 } else { 1 },
            ri,
            lidx,
        );*/

        //TODO:      cost = me_ipel_diamond(pi, x, y, log2_cuw, log2_cuh, ri, lidx, range, gmvp, mvi, mvt, bi, &tmpstep, MAX_FIRST_SEARCH_STEP);

        if cost < cost_best {
            cost_best = cost;
            mv[MV_X] = mvt[MV_X];
            mv[MV_Y] = mvt[MV_Y];
            if (mvp[MV_X] - mv[MV_X]).abs() < 2 && (mvp[MV_Y] - mv[MV_Y]).abs() < 2 {
                beststep = 0;
            } else {
                beststep = tmpstep;
            }
        }

        if bi == BI_NON && beststep > RASTER_SEARCH_THD {
            //TODO: cost = me_raster(pi, x, y, log2_cuw, log2_cuh, ri, lidx, range, gmvp, mvt);
            if cost < cost_best {
                beststep = RASTER_SEARCH_THD;

                cost_best = cost;

                mv[MV_X] = mvt[MV_X];
                mv[MV_Y] = mvt[MV_Y];
            }
        }

        while bi != BI_NORMAL && beststep > REFINE_SEARCH_THD {
            mvc[MV_X] = x + (mv[MV_X] >> 2);
            mvc[MV_Y] = y + (mv[MV_Y] >> 2);

            //TODO: get_range_ipel(pi, mvc, range, (bi != BI_NORMAL)? 0: 1, ri, lidx);

            mvi[MV_X] = mv[MV_X] + (x << 2);
            mvi[MV_Y] = mv[MV_Y] + (y << 2);

            beststep = 0;
            //TODO: cost = me_ipel_diamond(pi, x, y, log2_cuw, log2_cuh, ri, lidx, range, gmvp, mvi, mvt, bi, &tmpstep, MAX_REFINE_SEARCH_STEP);
            if cost < cost_best {
                cost_best = cost;

                mv[MV_X] = mvt[MV_X];
                mv[MV_Y] = mvt[MV_Y];

                if (mvp[MV_X] - mv[MV_X]).abs() < 2 && (mvp[MV_Y] - mv[MV_Y]).abs() < 2 {
                    beststep = 0;
                } else {
                    beststep = tmpstep;
                }
            }
        }

        if self.me_level > ME_LEV_IPEL {
            /* sub-pel ME */
            //TODO:  cost = me_spel_pattern(pi, x, y, log2_cuw, log2_cuh, ri, lidx, gmvp, mv, mvt, bi);

            if cost < cost_best {
                cost_best = cost;

                mv[MV_X] = mvt[MV_X];
                mv[MV_Y] = mvt[MV_Y];
            }
        } else {
            mvc[MV_X] = x + (mv[MV_X] >> 2);
            mvc[MV_Y] = y + (mv[MV_Y] >> 2);

            //TODO: get_range_ipel(pi, mvc, range, (bi != BI_NORMAL) ? 0: 1, ri, lidx);

            mvi[MV_X] = mv[MV_X] + (x << 2);
            mvi[MV_Y] = mv[MV_Y] + (y << 2);
            //TODO:  cost = me_ipel_refinement(pi, x, y, log2_cuw, log2_cuh, ri, lidx, range, gmvp, mvi, mvt, bi, &tmpstep, MAX_REFINE_SEARCH_STEP);
            if cost < cost_best {
                cost_best = cost;

                mv[MV_X] = mvt[MV_X];
                mv[MV_Y] = mvt[MV_Y];
            }
        }

        cost_best
    }
}
