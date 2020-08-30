use super::def::*;
use super::plane::*;
use super::region::*;
use super::tracer::*;
use super::util::*;

fn evc_recon_plane_region(
    tracer: &mut Option<Tracer>,
    coef: &[i16],
    pred: &[pel],
    is_coef: bool,
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
    rec: &mut PlaneRegionMut<'_, pel>,
    ch_type: usize,
) {
    if !is_coef {
        /* just copy pred to rec */
        for j in 0..cuh {
            for i in 0..cuw {
                rec[y + j][x + i] = EVC_CLIP3(0, (1 << BIT_DEPTH) - 1, pred[j * cuw + i]);
            }
        }
    //#if SIMD_CLIP
    //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
    //#endif
    } else
    /* add b/w pred and coef and copy it into rec */
    {
        for j in 0..cuh {
            for i in 0..cuw {
                let t0 = coef[j * cuw + i] as i32 + pred[j * cuw + i] as i32;
                rec[y + j][x + i] = EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, t0) as u16;
            }
        }
        //#if SIMD_CLIP
        //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
        //#endif
    }

    TRACE_RECO_PLANE_REGION(tracer, ch_type, x, y, cuw, cuh, rec);
}

pub(crate) fn evc_recon(
    tracer: &mut Option<Tracer>,
    coef: &[i16],
    pred: &[pel],
    is_coef: bool,
    cuw: usize,
    cuh: usize,
    rec: &mut [pel],
    ch_type: usize,
) {
    if !is_coef {
        /* just copy pred to rec */
        for j in 0..cuh {
            for i in 0..cuw {
                rec[j * cuw + i] = EVC_CLIP3(0, (1 << BIT_DEPTH) - 1, pred[j * cuw + i]);
            }
        }
    //#if SIMD_CLIP
    //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
    //#endif
    } else
    /* add b/w pred and coef and copy it into rec */
    {
        for j in 0..cuh {
            for i in 0..cuw {
                let t0 = coef[j * cuw + i] as i32 + pred[j * cuw + i] as i32;
                rec[j * cuw + i] = EVC_CLIP3(0i32, ((1 << BIT_DEPTH) - 1) as i32, t0) as u16;
            }
        }
        //#if SIMD_CLIP
        //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
        //#endif
    }

    TRACE_RECO(tracer, ch_type, cuw, cuh, rec);
}

pub(crate) fn evc_recon_yuv(
    tracer: &mut Option<Tracer>,
    mut x: usize,
    mut y: usize,
    mut cuw: usize,
    mut cuh: usize,
    coef: &Vec<Vec<i16>>, //[[i16; MAX_CU_DIM]; N_C],
    pred: &Vec<Vec<pel>>, //[[pel; MAX_CU_DIM]; N_C],
    nnz: &[bool; N_C],
    planes: &mut [Plane<pel>; N_C],
) {
    /* Y */
    let rec = &mut planes[Y_C].as_region_mut();
    evc_recon_plane_region(
        tracer, &coef[Y_C], &pred[Y_C], nnz[Y_C], x, y, cuw, cuh, rec, Y_C,
    );

    /* chroma */
    x >>= 1;
    y >>= 1;
    cuw >>= 1;
    cuh >>= 1;

    {
        let rec = &mut planes[U_C].as_region_mut();
        evc_recon_plane_region(
            tracer, &coef[U_C], &pred[U_C], nnz[U_C], x, y, cuw, cuh, rec, U_C,
        );
    }
    {
        let rec = &mut planes[V_C].as_region_mut();
        evc_recon_plane_region(
            tracer, &coef[V_C], &pred[V_C], nnz[V_C], x, y, cuw, cuh, rec, V_C,
        );
    }
}
