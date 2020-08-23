use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Write;

use super::def::*;
use super::region::*;

pub(crate) type Tracer = (Box<dyn Write>, isize);

////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(feature = "trace")]
pub(crate) fn OPEN_TRACE(encoder: bool) -> Option<Tracer> {
    let fp_trace = if encoder {
        File::create("enc_trace.txt")
    } else {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("dec_trace.txt")
    };
    if let Ok(fp) = fp_trace {
        Some((Box::new(fp), 0))
    } else {
        None
    }
}

#[cfg(feature = "trace")]
pub(crate) fn EVC_TRACE_COUNTER(tracer: &mut Option<Tracer>) {
    if let Some((writer, counter)) = tracer {
        writer.write_fmt(format_args!("{} \t", *counter));
        *counter += 1;
    }
}

#[cfg(feature = "trace")]
pub(crate) fn EVC_TRACE_COUNTER_RESET(tracer: &mut Option<Tracer>) {
    if let Some((writer, counter)) = tracer {
        *counter = 0;
    }
}

#[cfg(feature = "trace")]
pub(crate) fn EVC_TRACE<T: Display>(tracer: &mut Option<Tracer>, name: T) {
    if let Some((writer, _)) = tracer {
        writer.write_fmt(format_args!("{}", name));
    }
}

#[cfg(feature = "trace")]
pub(crate) fn EVC_TRACE_INT_HEX(tracer: &mut Option<Tracer>, val: isize) {
    if let Some((writer, _)) = tracer {
        writer.write_fmt(format_args!("0x{:x}", val));
    }
}

#[cfg(feature = "trace_bin")]
pub(crate) fn TRACE_BIN(tracer: &mut Option<Tracer>, model: u16, range: u32, lps: u32) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "model ");
    EVC_TRACE(tracer, model);
    EVC_TRACE(tracer, " range ");
    EVC_TRACE(tracer, range);
    EVC_TRACE(tracer, " lps ");
    EVC_TRACE(tracer, lps);
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_coef")]
pub(crate) fn TRACE_COEF(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    coef: &[i16],
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Coef for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for i in 0..cuw * cuh {
        if i != 0 {
            EVC_TRACE(tracer, " , ");
        }
        EVC_TRACE(tracer, coef[i]);
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_resi")]
pub(crate) fn TRACE_RESI(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    resi: &[i16],
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Resi for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for i in 0..cuw * cuh {
        if i != 0 {
            EVC_TRACE(tracer, " , ");
        }
        EVC_TRACE(tracer, resi[i]);
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_pred")]
pub(crate) fn TRACE_PRED(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    pred: &[u16],
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Pred for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for i in 0..cuw * cuh {
        if i != 0 {
            EVC_TRACE(tracer, " , ");
        }
        EVC_TRACE(tracer, pred[i]);
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_reco")]
pub(crate) fn TRACE_RECO_PLANE_REGION(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
    reco: &PlaneRegionMut<'_, pel>,
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Reco for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for j in 0..cuh {
        for i in 0..cuw {
            if !(i == 0 && j == 0) {
                EVC_TRACE(tracer, " , ");
            }
            EVC_TRACE(tracer, reco[y + j][x + i]);
        }
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_reco")]
pub(crate) fn TRACE_RECO(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    rec: &[pel],
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Reco for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for j in 0..cuh {
        for i in 0..cuw {
            if !(i == 0 && j == 0) {
                EVC_TRACE(tracer, " , ");
            }
            EVC_TRACE(tracer, rec[j * cuw + i]);
        }
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_reco")]
pub(crate) fn TRACE_CUDATA(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    stride: usize,
    rec: &[pel],
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "CUDATA for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " : ");
    for j in 0..cuh {
        for i in 0..cuw {
            if !(i == 0 && j == 0) {
                EVC_TRACE(tracer, " , ");
            }
            EVC_TRACE(tracer, rec[j * stride + i]);
        }
    }
    EVC_TRACE(tracer, " \n");
}

#[cfg(feature = "trace_dbf")]
pub(crate) fn TRACE_DBF(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    x: usize,
    y: usize,
    size: usize,
    hor: bool,
    dbf: &PlaneRegionMut<'_, pel>,
) {
    EVC_TRACE_COUNTER(tracer);
    EVC_TRACE(tracer, "Dbf for ");
    EVC_TRACE(tracer, ch_type);
    EVC_TRACE(tracer, " x ");
    EVC_TRACE(tracer, x);
    EVC_TRACE(tracer, " y ");
    EVC_TRACE(tracer, y);
    EVC_TRACE(tracer, " size ");
    EVC_TRACE(tracer, size);
    EVC_TRACE(tracer, " hor ");
    EVC_TRACE(tracer, hor as u8);
    EVC_TRACE(tracer, " : ");
    for k in 0..size {
        if hor {
            EVC_TRACE(tracer, dbf[y - 2][x + k]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y - 1][x + k]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y + 0][x + k]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y + 1][x + k]);
            EVC_TRACE(tracer, " , ");
        } else {
            EVC_TRACE(tracer, dbf[y + k][x - 2]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y + k][x - 1]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y + k][x + 0]);
            EVC_TRACE(tracer, " , ");
            EVC_TRACE(tracer, dbf[y + k][x + 1]);
            EVC_TRACE(tracer, " , ");
        }
    }
    EVC_TRACE(tracer, "\n");
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(not(feature = "trace"))]
pub(crate) fn OPEN_TRACE(encoder: bool) -> Option<Tracer> {
    None
}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE_COUNTER(tracer: &mut Option<Tracer>) {}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE_COUNTER_RESET(tracer: &mut Option<Tracer>) {}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE<T: Display>(writer: &mut Option<Tracer>, name: T) {}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE_INT_HEX(tracer: &mut Option<Tracer>, val: isize) {}

#[cfg(not(feature = "trace_bin"))]
pub(crate) fn TRACE_BIN(tracer: &mut Option<Tracer>, model: u16, range: u32, lps: u32) {}

#[cfg(not(feature = "trace_coef"))]
pub(crate) fn TRACE_COEF(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    coef: &[i16],
) {
}

#[cfg(not(feature = "trace_resi"))]
pub(crate) fn TRACE_RESI(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    resi: &[i16],
) {
}

#[cfg(not(feature = "trace_pred"))]
pub(crate) fn TRACE_PRED(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    pred: &[u16],
) {
}

#[cfg(not(feature = "trace_reco"))]
pub(crate) fn TRACE_RECO_PLANE_REGION(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    x: usize,
    y: usize,
    cuw: usize,
    cuh: usize,
    reco: &PlaneRegionMut<'_, pel>,
) {
}

#[cfg(not(feature = "trace_reco"))]
pub(crate) fn TRACE_RECO(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    rec: &[pel],
) {
}

#[cfg(not(feature = "trace_reco"))]
pub(crate) fn TRACE_CUDATA(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    cuw: usize,
    cuh: usize,
    stride: usize,
    rec: &[pel],
) {
}

#[cfg(not(feature = "trace_dbf"))]
pub(crate) fn TRACE_DBF(
    tracer: &mut Option<Tracer>,
    ch_type: usize,
    x: usize,
    y: usize,
    size: usize,
    hor: bool,
    dbf: &PlaneRegionMut<'_, pel>,
) {
}
