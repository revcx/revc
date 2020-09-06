use super::def::*;
use super::plane::*;
use super::region::*;
use super::tbl::*;
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
        let mut src = pred;
        for j in 0..cuh {
            let dst = &mut rec[y + j];
            dst[x..x + cuw].copy_from_slice(&src[..cuw]);
            src = &src[cuw..];
        }
    } else {
        /* add b/w pred and coef and copy it into rec */
        let mut src1 = coef;
        let mut src2 = pred;
        for j in 0..cuh {
            let dst = &mut rec[y + j];
            for i in 0..cuw {
                let t0 = src1[i] as i32 + src2[i] as i32;
                dst[x + i] = EVC_CLIP3(0i32, MAX_SAMPLE_VAL_I32, t0) as u16;
            }
            src1 = &src1[cuw..];
            src2 = &src2[cuw..];
        }
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
        let cuwh = cuw * cuh;
        rec[..cuwh].copy_from_slice(&pred[..cuwh]);
    } else {
        /* add b/w pred and coef and copy it into rec */
        let mut src1 = coef;
        let mut src2 = pred;
        let mut dst = &mut rec[..];
        for j in 0..cuh {
            for i in 0..cuw {
                let t0 = src1[i] as i32 + src2[i] as i32;
                dst[i] = EVC_CLIP3(0i32, MAX_SAMPLE_VAL_I32, t0) as u16;
            }
            src1 = &src1[cuw..];
            src2 = &src2[cuw..];
            dst = &mut dst[cuw..];
        }
    }

    TRACE_RECO(tracer, ch_type, cuw, cuh, rec);
}

pub(crate) fn evc_recon_yuv(
    tracer: &mut Option<Tracer>,
    mut x: usize,
    mut y: usize,
    mut cuw: usize,
    mut cuh: usize,
    coef: &[i16],
    pred: &Vec<Vec<pel>>, //[[pel; MAX_CU_DIM]; N_C],
    nnz: &[bool; N_C],
    planes: &mut [Plane<pel>; N_C],
) {
    /* Y */
    let rec = &mut planes[Y_C].as_region_mut();
    evc_recon_plane_region(
        tracer,
        &coef[tbl_cu_dim_offset[Y_C]..],
        &pred[Y_C],
        nnz[Y_C],
        x,
        y,
        cuw,
        cuh,
        rec,
        Y_C,
    );

    /* chroma */
    x >>= 1;
    y >>= 1;
    cuw >>= 1;
    cuh >>= 1;

    let rec = &mut planes[U_C].as_region_mut();
    evc_recon_plane_region(
        tracer,
        &coef[tbl_cu_dim_offset[U_C]..],
        &pred[U_C],
        nnz[U_C],
        x,
        y,
        cuw,
        cuh,
        rec,
        U_C,
    );

    let rec = &mut planes[V_C].as_region_mut();
    evc_recon_plane_region(
        tracer,
        &coef[tbl_cu_dim_offset[V_C]..],
        &pred[V_C],
        nnz[V_C],
        x,
        y,
        cuw,
        cuh,
        rec,
        V_C,
    );
}
