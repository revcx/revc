use super::plane::*;
use super::plane_region::*;
use super::util::*;
use super::*;
use crate::api::util::*;

fn evc_recon(
    coef: &[i16],
    pred: &[pel],
    is_coef: bool,
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
    rec: &mut PlaneRegionMut<'_, pel>,
) {
    if !is_coef {
        /* just copy pred to rec */
        for i in 0..cuh {
            for j in 0..cuw {
                rec[y + i][x + j] = EVC_CLIP3(0, (1 << BIT_DEPTH) - 1, pred[i * cuw + j]);
            }
        }
    //#if SIMD_CLIP
    //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
    //#endif
    } else
    /* add b/w pred and coef and copy it into rec */
    {
        for i in 0..cuh {
            for j in 0..cuw {
                let t0 = coef[i * cuw + j] + pred[i * cuw + j] as i16;
                rec[y + i][x + j] = EVC_CLIP3(0i16, ((1 << BIT_DEPTH) - 1) as i16, t0) as u16;
            }
        }
        //#if SIMD_CLIP
        //        clip_simd(rec, s_rec, rec, s_rec, cuw, cuh, adapt_clip_min[adapt_clip_comp], adapt_clip_max[adapt_clip_comp]);
        //#endif
    }
}

pub(crate) fn evc_recon_yuv(
    mut x: usize,
    mut y: usize,
    mut cuw: usize,
    mut cuh: usize,
    coef: &[[i16; MAX_CU_DIM]; N_C],
    pred: &[[pel; MAX_CU_DIM]; N_C],
    nnz: &[bool; N_C],
    planes: &mut [Plane<pel>; N_C],
    tree_cons: &TREE_CONS,
) {
    if evc_check_luma(tree_cons) {
        /* Y */
        let rec = &mut planes[Y_C].as_region_mut();
        evc_recon(&coef[Y_C], &pred[Y_C], nnz[Y_C], x, y, cuw, cuh, rec);
    }
    if evc_check_chroma(tree_cons) {
        /* chroma */
        x >>= 1;
        y >>= 1;
        cuw >>= 1;
        cuh >>= 1;

        {
            let rec = &mut planes[U_C].as_region_mut();
            evc_recon(&coef[U_C], &pred[U_C], nnz[U_C], x, y, cuw, cuh, rec);
        }
        {
            let rec = &mut planes[V_C].as_region_mut();
            evc_recon(&coef[V_C], &pred[V_C], nnz[V_C], x, y, cuw, cuh, rec);
        }
    }
}
