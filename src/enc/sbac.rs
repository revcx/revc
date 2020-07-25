use super::bsw::*;
use crate::api::*;
use crate::def::*;
use crate::tracer::*;

pub(crate) struct EvceSbac {
    range: u32,
    code: u32,
    code_bits: u32,
    stacked_ff: u32,
    stacked_zero: u32,
    pending_byte: u32,
    is_pending_byte: u32,
    //EVC_SBAC_CTX   ctx;
    bitcounter: u32,
    is_bitcount: bool,
    bin_counter: u32,
}

impl EvceSbac {
    pub(crate) fn reset(&mut self, sbac_ctx: &mut EvcSbacCtx, slice_type: SliceType, slice_qp: u8) {
        /* Initialization of the internal variables */
        self.range = 16384;
        self.code = 0;
        self.code_bits = 11;
        self.pending_byte = 0;
        self.is_pending_byte = 0;
        self.stacked_ff = 0;
        self.stacked_zero = 0;
        self.bin_counter = 0;

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
    }

    pub(crate) fn finish(&mut self, bs: &mut EvceBsw) {
        let mut tmp = (self.code + self.range - 1) & (0xFFFFFFFF << 14);
        if tmp < self.code {
            tmp += 8192;
        }

        self.code = tmp << self.code_bits;
        self.carry_propagate(bs);

        self.code <<= 8;
        self.carry_propagate(bs);

        while self.stacked_zero > 0 {
            bs.write(0x00, 8, None);
            self.stacked_zero -= 1;
        }

        if self.pending_byte != 0 {
            bs.write(self.pending_byte, 8, None);
        } else {
            if self.code_bits < 4 {
                bs.write(0, 4 - self.code_bits as isize, None);

                while !bs.IS_BYTE_ALIGN() {
                    bs.write1(0, None);
                }
            }
        }
    }

    fn carry_propagate(&mut self, bs: &mut EvceBsw) {
        let out_bits = self.code >> 17;

        self.code &= (1 << 17) - 1;

        if out_bits < 0xFF {
            while self.stacked_ff != 0 {
                self.put_byte(bs, 0xFF);
                self.stacked_ff -= 1;
            }
            self.put_byte(bs, out_bits as u8);
        } else if out_bits > 0xFF {
            self.pending_byte += 1;
            while self.stacked_ff != 0 {
                self.put_byte(bs, 0x00);
                self.stacked_ff -= 1;
            }
            self.put_byte(bs, (out_bits & 0xFF) as u8);
        } else {
            self.stacked_ff += 1;
        }
    }

    fn put_byte(&mut self, bs: &mut EvceBsw, writing_byte: u8) {
        if self.is_pending_byte != 0 {
            if self.pending_byte == 0 {
                self.stacked_zero += 1;
            } else {
                while self.stacked_zero > 0 {
                    if self.is_bitcount {
                        self.write_est(0x00, 8);
                    } else {
                        bs.write(0x00, 8, None);
                    }
                    self.stacked_zero -= 1;
                }
                if self.is_bitcount {
                    self.write_est(self.pending_byte, 8);
                } else {
                    bs.write(self.pending_byte, 8, None);
                }
            }
        }
        self.pending_byte = writing_byte as u32;
        self.is_pending_byte = 1;
    }

    fn write_est(&mut self, byte: u32, len: isize) {
        self.bitcounter += len as u32;
    }

    fn encode_bin_ep(&mut self, bs: &mut EvceBsw, bin: u32) {
        self.bin_counter += 1;

        self.range >>= 1;

        if bin != 0 {
            self.code += self.range;
        }

        self.range <<= 1;
        self.code <<= 1;

        self.code_bits -= 1;
        if self.code_bits == 0 {
            self.carry_propagate(bs);
            self.code_bits = 8;
        }
    }

    fn write_unary_sym_ep(&mut self, bs: &mut EvceBsw, mut sym: u32, max_val: u32) {
        let mut icounter = 0;

        self.encode_bin_ep(bs, if sym != 0 { 1 } else { 0 });
        icounter += 1;

        if sym == 0 {
            return;
        }

        while sym != 0 {
            if icounter < max_val {
                self.encode_bin_ep(bs, if sym != 0 { 1 } else { 0 });
                icounter += 1;
            }
            sym -= 1;
        }
    }

    fn write_unary_sym(
        &mut self,
        bs: &mut EvceBsw,
        model: &mut [SBAC_CTX_MODEL],
        mut sym: u32,
        num_ctx: u32,
    ) {
        let mut ctx_idx = 0;

        self.encode_bin(bs, &mut model[0], if sym != 0 { 1 } else { 0 });

        if sym == 0 {
            return;
        }

        while sym != 0 {
            if ctx_idx < num_ctx - 1 {
                ctx_idx += 1;
            }
            self.encode_bin(
                bs,
                &mut model[ctx_idx as usize],
                if sym != 0 { 1 } else { 0 },
            );
            sym -= 1;
        }
    }

    fn write_truncate_unary_sym(
        &mut self,
        bs: &mut EvceBsw,
        model: &mut [SBAC_CTX_MODEL],
        mut sym: u32,
        num_ctx: u32,
        max_num: u32,
    ) {
        if max_num > 1 {
            for ctx_idx in 0..max_num - 1 {
                let symbol = if ctx_idx == sym { 0 } else { 1 };
                let idx = if ctx_idx > max_num - 1 {
                    max_num - 1
                } else {
                    ctx_idx
                } as usize;
                self.encode_bin(bs, &mut model[idx], symbol);

                if symbol == 0 {
                    break;
                }
            }
        }
    }

    fn encode_bins_ep(&mut self, bs: &mut EvceBsw, value: u32, num_bin: isize) {
        let mut bin = num_bin - 1;
        while bin >= 0 {
            self.encode_bin_ep(bs, value & (1 << bin));
            bin -= 1;
        }
    }

    pub(crate) fn encode_bin(&mut self, bs: &mut EvceBsw, model: &mut SBAC_CTX_MODEL, bin: u32) {
        self.bin_counter += 1;

        let mut state = (*model) >> 1;
        let mut mps = (*model) & 1;

        let mut lps = (state as u32 * self.range) >> 9;
        lps = if lps < 437 { 437 } else { lps };

        self.range -= lps;

        TRACE_BIN(&mut bs.tracer, *model, self.range, lps);

        if bin != mps as u32 {
            if self.range >= lps as u32 {
                self.code += self.range;
                self.range = lps as u32;
            }

            state = state + ((512 - state + 16) >> 5);
            if state > 256 {
                mps = 1 - mps;
                state = 512 - state;
            }
            *model = (state << 1) + mps;
        } else {
            state = state - ((state + 16) >> 5);
            *model = (state << 1) + mps;
        }

        while self.range < 8192 {
            self.range <<= 1;
            self.code <<= 1;
            self.code_bits -= 1;

            if self.code_bits == 0 {
                self.carry_propagate(bs);
                self.code_bits = 8;
            }
        }
    }

    pub(crate) fn encode_bin_trm(&mut self, bs: &mut EvceBsw, bin: u32) {
        self.bin_counter += 1;

        self.range -= 1;

        if bin != 0 {
            self.code += self.range;
            self.range = 1;
        }

        while self.range < 8192 {
            self.range <<= 1;
            self.code <<= 1;
            self.code_bits -= 1;
            if self.code_bits == 0 {
                self.carry_propagate(bs);
                self.code_bits = 8;
            }
        }
    }
}
