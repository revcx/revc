use super::*;
use crate::def::*;

lazy_static! {
    pub(crate) static ref entropy_bits: Box<[i32]> = {
        let mut bits = vec![0; 1024].into_boxed_slice();
        for i in 0..1024 {
            let p = (512.0 * (i as f64 + 0.5)) / 1024.0;
            bits[i] = (-32768.0 * (p.log10() / (2.0f64).log10() - 9.0)) as i32;
        }
        bits
    };
}

pub(crate) fn biari_no_bits(symbol: usize, cm: SBAC_CTX_MODEL) -> i32 {
    let mps = cm & 1;
    let mut state = cm >> 1;
    let sym = if symbol != 0 { 1 } else { 0 };
    state = if sym != mps { state } else { 512 - state };

    entropy_bits[(state as usize) << 1]
}

impl EvceCtx {
    pub(crate) fn evce_set_qp(&mut self, qp: u8) {
        self.core.qp = qp;
        self.core.qp_y = GET_LUMA_QP(self.core.qp as i8) as u8;
        let qp_i_cb = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_u_offset,
        );
        let qp_i_cr = EVC_CLIP3(
            -6 * (BIT_DEPTH as i8 - 8),
            57,
            self.core.qp as i8 + self.sh.qp_v_offset,
        );
        self.core.qp_u = (self.core.evc_tbl_qp_chroma_dynamic_ext[0]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cb) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;
        self.core.qp_v = (self.core.evc_tbl_qp_chroma_dynamic_ext[1]
            [(EVC_TBL_CHROMA_QP_OFFSET + qp_i_cr) as usize]
            + 6 * (BIT_DEPTH as i8 - 8)) as u8;
    }
}
