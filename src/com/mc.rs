use super::picman::*;
use super::*;
use crate::api::util::*;
use crate::dec::CUBuffer; //TODO: move CUBuffer to com

pub(crate) fn evc_mc(
    x: u16,
    y: u16,
    pic_w: u16,
    pic_h: u16,
    cuw: u8,
    cuh: u8,
    refi: &[i8],
    mv: &[[i16; MV_D]; REFP_NUM],
    refp: &Vec<Vec<EvcRefP>>,
    pred: &[CUBuffer<pel>; 2], //TODO: move CUBuffer to com
    poc_c: i32,
) {
    /*
        EVC_PIC    *ref_pic;
    //#if !OPT_SIMD_MC_L
        pel         *p2, *p3;
    //#endif
        int          qpel_gmv_x, qpel_gmv_y;
        int          bidx = 0;
        s16          mv_t[REFP_NUM][MV_D];
        s16          mv_before_clipping[REFP_NUM][MV_D]; //store it to pass it to interpolation function for deriving correct interpolation filter

        mv_before_clipping[REFP_0][MV_X] = mv[REFP_0][MV_X];
        mv_before_clipping[REFP_0][MV_Y] = mv[REFP_0][MV_Y];
        mv_before_clipping[REFP_1][MV_X] = mv[REFP_1][MV_X];
        mv_before_clipping[REFP_1][MV_Y] = mv[REFP_1][MV_Y];

        mv_clip(x, y, pic_w, pic_h, w, h, refi, mv, mv_t);

        s16          mv_refine[REFP_NUM][MV_D] = { {mv[REFP_0][MV_X], mv[REFP_0][MV_Y]},
                                                  {mv[REFP_1][MV_X], mv[REFP_1][MV_Y]} };

        s16          inital_mv[REFP_NUM][MV_D] = { { mv[REFP_0][MV_X], mv[REFP_0][MV_Y] },
                                                   { mv[REFP_1][MV_X], mv[REFP_1][MV_Y] } };

        s32          extend_width = (DMVR_NEW_VERSION_ITER_COUNT + 1) * REF_PRED_EXTENTION_PEL_COUNT;
        s32          extend_width_minus1 = DMVR_NEW_VERSION_ITER_COUNT * REF_PRED_EXTENTION_PEL_COUNT;
        int          stride = w + (extend_width << 1);
        s16          mv_offsets[REFP_NUM][MV_D] = { {0,}, };
        s32          center_point_avgs_l0_l1[2 * REFP_NUM] = { 0, 0, 0, 0 }; // center_point_avgs_l0_l1[2,3] for "A" and "B" current center point average
        int iterations_count = DMVR_ITER_COUNT;
        BOOL         dmvr_poc_condition;
        if (!REFI_IS_VALID(refi[REFP_0]) || !REFI_IS_VALID(refi[REFP_1]))
        {
            apply_DMVR = 0;
            dmvr_poc_condition = 0;
        }
        else
        {
            int          poc0 = refp[refi[REFP_0]][REFP_0].poc;
            int          poc1 = refp[refi[REFP_1]][REFP_1].poc;
            dmvr_poc_condition = ((BOOL)((poc_c - poc0)*(poc_c - poc1) < 0)) && (abs(poc_c - poc0) == abs(poc_c - poc1));
        }

        g_mc_ftr = MC_FILTER_BASE;

        if (REFI_IS_VALID(refi[REFP_0]))
        {
            /* forward */
            ref_pic = refp[refi[REFP_0]][REFP_0].pic;
            qpel_gmv_x = (x << 2) + mv_t[REFP_0][MV_X];
            qpel_gmv_y = (y << 2) + mv_t[REFP_0][MV_Y];

            if (!apply_DMVR)
            {
                evc_mc_l(mv_before_clipping[REFP_0][MV_X] << 2, mv_before_clipping[REFP_0][MV_Y] << 2, ref_pic->y, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_l, w, pred[0][Y_C], w, h);
            }

            if (!REFI_IS_VALID(refi[REFP_1]) || !apply_DMVR || !dmvr_poc_condition)
            {
                evc_mc_c(mv_before_clipping[REFP_0][MV_X] << 2, mv_before_clipping[REFP_0][MV_Y] << 2, ref_pic->u, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_c, w >> 1, pred[0][U_C], w >> 1, h >> 1);
                evc_mc_c(mv_before_clipping[REFP_0][MV_X] << 2, mv_before_clipping[REFP_0][MV_Y] << 2, ref_pic->v, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_c, w >> 1, pred[0][V_C], w >> 1, h >> 1);
            }

            bidx++;
        }

        /* check identical motion */
        if (REFI_IS_VALID(refi[REFP_0]) && REFI_IS_VALID(refi[REFP_1]))
        {
            if (refp[refi[REFP_0]][REFP_0].pic->poc == refp[refi[REFP_1]][REFP_1].pic->poc &&  mv_t[REFP_0][MV_X] == mv_t[REFP_1][MV_X] && mv_t[REFP_0][MV_Y] == mv_t[REFP_1][MV_Y])
            {
                return;
            }
        }

        if (REFI_IS_VALID(refi[REFP_1]))
        {
            /* backward */
            ref_pic = refp[refi[REFP_1]][REFP_1].pic;
            qpel_gmv_x = (x << 2) + mv_t[REFP_1][MV_X];
            qpel_gmv_y = (y << 2) + mv_t[REFP_1][MV_Y];

            if (!apply_DMVR)
            {
                evc_mc_l(mv_before_clipping[REFP_1][MV_X] << 2, mv_before_clipping[REFP_1][MV_Y] << 2, ref_pic->y, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_l, w, pred[bidx][Y_C], w, h);
            }

            if (!REFI_IS_VALID(refi[REFP_0]) || !apply_DMVR || !dmvr_poc_condition)
            {
                evc_mc_c(mv_before_clipping[REFP_1][MV_X] << 2, mv_before_clipping[REFP_1][MV_Y] << 2, ref_pic->u, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_c, w >> 1, pred[bidx][U_C], w >> 1, h >> 1);
                evc_mc_c(mv_before_clipping[REFP_1][MV_X] << 2, mv_before_clipping[REFP_1][MV_Y] << 2, ref_pic->v, (qpel_gmv_x << 2), (qpel_gmv_y << 2), ref_pic->s_c, w >> 1, pred[bidx][V_C], w >> 1, h >> 1);
            }

            bidx++;
        }

        if (bidx == 2)
        {
            BOOL template_needs_update = FALSE;
            s32 center_cost[2] = { 1 << 30, 1 << 30 };

            //only if the references are located on opposite sides of the current frame
            if (apply_DMVR && dmvr_poc_condition)
            {
                if (apply_DMVR)
                {
                    processDMVR(x, y, pic_w, pic_h, w, h, refi, mv, refp, pred, poc_c, dmvr_current_template, dmvr_ref_pred_interpolated
                        , dmvr_half_pred_interpolated
                        , iterations_count
                    );
                }

                mv[REFP_0][MV_X] = inital_mv[REFP_0][MV_X];
                mv[REFP_0][MV_Y] = inital_mv[REFP_0][MV_Y];

                mv[REFP_1][MV_X] = inital_mv[REFP_1][MV_X];
                mv[REFP_1][MV_Y] = inital_mv[REFP_1][MV_Y];
            } //if (apply_DMVR && ((poc_c - poc0)*(poc_c - poc1) < 0))

    //#if OPT_SIMD_MC_L
    //        average_16b_no_clip_sse(pred[0][Y_C], pred[1][Y_C], pred[0][Y_C], w, w, w, w, h);
    //#else
            pel* p0 = pred[0][Y_C];
            pel* p1 = pred[1][Y_C];
            for (int j = 0; j < h; j++)
            {
                for (int i = 0; i < w; i++)
                {
                    p0[i] = (p0[i] + p1[i] + 1) >> 1;
                }
                p0 += w;
                p1 += w;
            }
    //#endif

    /*#if OPT_SIMD_MC_L
            w >>= 1;
            h >>= 1;
            average_16b_no_clip_sse(pred[0][U_C], pred[1][U_C], pred[0][U_C], w, w, w, w, h);
            average_16b_no_clip_sse(pred[0][V_C], pred[1][V_C], pred[0][V_C], w, w, w, w, h);
    #else*/
            {
                pel *p0, *p1;
                int i, j;
                p0 = pred[0][U_C];
                p1 = pred[1][U_C];
                p2 = pred[0][V_C];
                p3 = pred[1][V_C];
                w >>= 1;
                h >>= 1;
                for (j = 0; j < h; j++)
                {
                    for (i = 0; i < w; i++)
                    {
                        p0[i] = (p0[i] + p1[i] + 1) >> 1;
                        p2[i] = (p2[i] + p3[i] + 1) >> 1;
                    }
                    p0 += w;
                    p1 += w;
                    p2 += w;
                    p3 += w;
                }
            }
    //#endif
        }

         */
}
