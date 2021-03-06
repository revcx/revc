use crate::api::*;
use crate::tracer::*;

/* Bitstream structure for encoder */
#[derive(Default)]
pub(crate) struct EvceBsw {
    /* buffer */
    code: u32,
    /* bits left in buffer */
    leftbits: isize,
    /* buffer */
    pub(crate) pkt: Option<Packet>,
    /* tracer */
    pub(crate) tracer: Option<Tracer>,
}

impl EvceBsw {
    /* is bitstream byte aligned? */
    #[inline]
    pub(crate) fn IS_BYTE_ALIGN(&self) -> bool {
        (self.leftbits & 0x7) == 0
    }

    /* get number of byte written */
    #[inline]
    pub(crate) fn GET_WRITE_BYTE(&self) -> usize {
        if let Some(pkt) = &self.pkt {
            pkt.data.len()
        } else {
            0
        }
    }

    /* number of bytes to be sunk */
    #[inline]
    pub(crate) fn GET_SINK_BYTE(&self) -> u32 {
        ((32 - self.leftbits + 7) >> 3) as u32
    }

    fn flush(&mut self) {
        let mut bytes = self.GET_SINK_BYTE();

        while bytes != 0 {
            if let Some(pkt) = &mut self.pkt {
                pkt.data.push(((self.code >> 24) & 0xFF) as u8);
            }
            self.code <<= 8;
            bytes -= 1;
        }

        self.leftbits = 32;
    }

    pub(crate) fn init(&mut self) {
        self.code = 0;
        self.leftbits = 32;
        self.pkt = Some(Packet {
            data: Vec::with_capacity(1024), // 1K?
            ts: 0,
        });
        self.tracer = None;
    }

    pub(crate) fn deinit(&mut self) {
        self.flush();
    }

    pub(crate) fn write_nalu_size(&mut self) {
        let size = self.GET_WRITE_BYTE() - 4;

        if let Some(pkt) = &mut self.pkt {
            pkt.data[0] = (size & 0x000000ff) as u8; //TBC(@Chernyak): is there a better way?
            pkt.data[1] = ((size & 0x0000ff00) >> 8) as u8;
            pkt.data[2] = ((size & 0x00ff0000) >> 16) as u8;
            pkt.data[3] = ((size & 0xff000000) >> 24) as u8;
        }
    }

    pub(crate) fn write1(&mut self, val: u32, name: Option<&str>) {
        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        self.leftbits -= 1;
        self.code |= ((val & 0x1) << self.leftbits);

        if self.leftbits == 0 {
            //evc_assert_rv(bs->cur <= bs->end, -1);
            self.flush();

            self.code = 0;
            self.leftbits = 32;
        }
    }

    pub(crate) fn write(&mut self, mut val: u32, len: isize, name: Option<&str>) {
        assert!(len > 0);

        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        let leftbits = self.leftbits;
        val <<= (32 - len);
        if leftbits == 0 {
            // val >> 32 overflow panic in rust, but val == val >> 32 == val << 32 in C/C++
            self.code |= val;
        } else {
            self.code |= (val >> (32 - leftbits));
        }

        if len < leftbits {
            self.leftbits -= len;
        } else {
            //evc_assert_rv(bs->cur + 4 <= bs->end, -1);

            self.leftbits = 0;
            self.flush();
            self.code = if leftbits < 32 { val << leftbits } else { 0 };
            self.leftbits = 32 - (len - leftbits);
        }
    }

    pub(crate) fn write_ue(&mut self, val: u32, name: Option<&str>) {
        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        let mut nn = ((val + 1) >> 1);
        let mut len_i = 0;
        while len_i < 16 && nn != 0 {
            nn >>= 1;
            len_i += 1;
        }

        let info = val + 1 - (1 << len_i);
        let code = (1 << len_i) | ((info) & ((1 << len_i) - 1));

        let len_c = (len_i << 1) + 1;

        self.write(code, len_c, None);
    }

    pub(crate) fn write_se(&mut self, val: i32, name: Option<&str>) {
        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        let v = if val <= 0 { -val * 2 } else { val * 2 - 1 };
        self.write_ue(v as u32, None);
    }
}
