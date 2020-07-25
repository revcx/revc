use super::bsr::*;
use crate::api::*;
use crate::def::*;
use crate::tracer::*;

/*****************************************************************************
 * SBAC structure
 *****************************************************************************/
#[derive(Default)]
pub(crate) struct EvcdSbac {
    pub(crate) range: u32,
    pub(crate) value: u32,
}

impl EvcdSbac {
    pub(crate) fn reset(
        &mut self,
        bs: &mut EvcdBsr,
        sbac_ctx: &mut EvcSbacCtx,
        slice_type: SliceType,
        slice_qp: u8,
    ) -> Result<(), EvcError> {
        /* Initialization of the internal variables */
        self.range = 16384;
        self.value = 0;
        for i in 0..14 {
            let t0 = bs.read1(None)?;
            self.value = ((self.value << 1) | t0) & 0xFFFF;
        }

        /* Initialization of the context models */
        for i in 0..NUM_CTX_SPLIT_CU_FLAG {
            sbac_ctx.split_cu_flag[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CC_RUN {
            sbac_ctx.run[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CC_LAST {
            sbac_ctx.last[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CC_LEVEL {
            sbac_ctx.level[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CBF_LUMA {
            sbac_ctx.cbf_luma[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CBF_CB {
            sbac_ctx.cbf_cb[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CBF_CR {
            sbac_ctx.cbf_cr[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_CBF_ALL {
            sbac_ctx.cbf_all[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_PRED_MODE {
            sbac_ctx.pred_mode[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_DIRECT_MODE_FLAG {
            sbac_ctx.direct_mode_flag[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_INTER_PRED_IDC {
            sbac_ctx.inter_dir[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_INTRA_PRED_MODE {
            sbac_ctx.intra_dir[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_MVP_IDX {
            sbac_ctx.mvp_idx[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_MVD {
            sbac_ctx.mvd[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_REF_IDX {
            sbac_ctx.refi[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_DELTA_QP {
            sbac_ctx.delta_qp[i] = PROB_INIT;
        }
        for i in 0..NUM_CTX_SKIP_FLAG {
            sbac_ctx.skip_flag[i] = PROB_INIT;
        }

        Ok(())
    }

    pub(crate) fn decode_bin(
        &mut self,
        bs: &mut EvcdBsr,
        model: &mut SBAC_CTX_MODEL,
    ) -> Result<u32, EvcError> {
        let mut state: u16 = (*model) >> 1;
        let mut mps: u16 = (*model) & 1;

        let mut lps: u32 = (state as u32 * self.range) >> 9;
        lps = if lps < 437 { 437 } else { lps };

        let mut bin: u32 = mps as u32;

        self.range -= lps;

        TRACE_BIN(&mut bs.tracer, *model, self.range, lps);

        if self.value >= self.range {
            bin = 1 - mps as u32;
            self.value -= self.range;
            self.range = lps;

            state = state + ((512 - state + 16) >> 5);
            if state > 256 {
                mps = 1 - mps;
                state = 512 - state;
            }
            *model = (state << 1) + mps;
        } else {
            bin = mps as u32;
            state = state - ((state + 16) >> 5);
            *model = (state << 1) + mps;
        }

        while self.range < 8192 {
            self.range <<= 1;
            let t0 = bs.read1(None)?;
            self.value = ((self.value << 1) | t0) & 0xFFFF;
        }

        Ok(bin)
    }

    pub(crate) fn decode_bin_ep(&mut self, bs: &mut EvcdBsr) -> Result<u32, EvcError> {
        self.range >>= 1;

        let bin = if self.value >= self.range {
            self.value -= self.range;
            1
        } else {
            0
        };

        self.range <<= 1;
        let t0 = bs.read1(None)?;
        self.value = ((self.value << 1) | t0) & 0xFFFF;

        Ok(bin)
    }

    pub(crate) fn decode_bin_trm(&mut self, bs: &mut EvcdBsr) -> Result<u32, EvcError> {
        self.range -= 1;
        if self.value >= self.range {
            while !bs.is_byte_aligned() {
                let t0 = bs.read1(None)?;
                evc_assert_rv(t0 == 0, EvcError::EVC_ERR_MALFORMED_BITSTREAM)?;
            }
            Ok(1)
        } else {
            while self.range < 8192 {
                self.range <<= 1;
                let t0 = bs.read1(None)?;
                self.value = ((self.value << 1) | t0) & 0xFFFF;
            }
            Ok(0)
        }
    }

    pub(crate) fn read_unary_sym(
        &mut self,
        bs: &mut EvcdBsr,
        models: &mut [SBAC_CTX_MODEL],
        num_ctx: u32,
    ) -> Result<u32, EvcError> {
        let mut symbol = self.decode_bin(bs, &mut models[0])?;

        if symbol == 0 {
            return Ok(symbol);
        }

        symbol = 0;
        let mut t32u = 1;
        let mut ctx_idx = 0;
        while t32u != 0 {
            if ctx_idx < num_ctx - 1 {
                ctx_idx += 1;
            }
            t32u = self.decode_bin(bs, &mut models[ctx_idx as usize])?;
            symbol += 1;
        }

        Ok(symbol)
    }

    pub(crate) fn read_truncate_unary_sym(
        &mut self,
        bs: &mut EvcdBsr,
        models: &mut [SBAC_CTX_MODEL],
        num_ctx: u32,
        max_num: u32,
    ) -> Result<u32, EvcError> {
        let mut ctx_idx = 0;
        if max_num > 1 {
            while ctx_idx < max_num - 1 {
                let symbol = self.decode_bin(
                    bs,
                    &mut models[if ctx_idx > num_ctx - 1 {
                        num_ctx - 1
                    } else {
                        ctx_idx
                    } as usize],
                )?;
                if symbol == 0 {
                    break;
                }
                ctx_idx += 1;
            }
        }

        Ok(ctx_idx)
    }
}
