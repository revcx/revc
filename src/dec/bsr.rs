use log::*;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;

#[cfg(feature = "TRACE_HLS")]
fn OPEN_TRACE() -> Option<Box<dyn Write>> {
    let fp_trace = OpenOptions::new()
        .append(true)
        .create(true)
        .open("dec_trace.txt");
    if let Ok(fp) = fp_trace {
        Some(Box::new(fp))
    } else {
        None
    }
}

#[cfg(feature = "TRACE_HLS")]
pub(crate) fn EVC_TRACE_STR(tracer: &mut Option<Box<dyn Write>>, name: &str) {
    if let Some(writer) = tracer {
        writer.write_fmt(format_args!("{}", name));
    }
}

#[cfg(feature = "TRACE_HLS")]
pub(crate) fn EVC_TRACE_HLS<T: Display>(tracer: &mut Option<Box<dyn Write>>, name: &str, val: T) {
    if let Some(writer) = tracer {
        writer.write_fmt(format_args!("{} {} \n", name, val));
    }
}

#[cfg(not(feature = "TRACE_HLS"))]
fn OPEN_TRACE() -> Option<Box<dyn Write>> {
    None
}

#[cfg(not(feature = "TRACE_HLS"))]
pub(crate) fn EVC_TRACE_STR(writer: &mut Option<Box<dyn Write>>, name: &str) {}

#[cfg(not(feature = "TRACE_HLS"))]
pub(crate) fn EVC_TRACE_HLS<T: Display>(writer: &mut Option<Box<dyn Write>>, name: &str, val: T) {}

/*
 * bitstream structure for decoder.
 *
 * NOTE: Don't change location of variable because this variable is used
 *       for assembly coding!
 */
#[derive(Default)]
pub(crate) struct EvcdBsr {
    /* temporary read code buffer */
    code: u32,
    /* left bits count in code */
    leftbits: isize,
    /* bitstream cur position */
    cur: usize,
    /* buffer */
    buf: Vec<u8>,
    /* trace */
    pub(crate) tracer: Option<Box<dyn Write>>,
}

/* Table of count of leading zero for 4 bit value */
static tbl_zero_count4: [u8; 16] = [4, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

impl EvcdBsr {
    #[inline]
    pub(crate) fn EVC_BSR_SKIP_CODE(&mut self, size: usize) {
        assert!(self.leftbits >= size as isize);
        if size == 32 {
            self.code = 0;
            self.leftbits = 0;
        } else {
            self.code <<= size as u32;
            self.leftbits -= size as isize;
        }
    }

    /* Is bitstream byte aligned? */
    #[inline]
    pub(crate) fn EVC_BSR_IS_BYTE_ALIGN(&self) -> bool {
        (self.leftbits & 0x7) == 0
    }

    /* get number of byte consumed */
    #[inline]
    pub(crate) fn EVC_BSR_GET_READ_BYTE(&self) -> isize {
        self.cur as isize - (self.leftbits >> 3)
    }

    pub(crate) fn new(buf: Vec<u8>) -> Self {
        EvcdBsr {
            code: 0,
            leftbits: 0,
            cur: 0,
            buf,
            tracer: OPEN_TRACE(),
        }
    }

    pub(crate) fn flush(&mut self, mut byte: isize) -> isize {
        let mut shift: i32 = 24;
        let mut code: u32 = 0;

        assert_ne!(byte, 0);

        let remained = (self.buf.len() as isize - self.cur as isize) + 1;
        if byte > remained {
            byte = remained;
        }

        if byte <= 0 {
            self.code = 0;
            self.leftbits = 0;
            return -1;
        }

        self.leftbits = byte << 3;

        self.cur += byte as usize;
        while byte != 0 {
            code |= ((self.buf[self.cur - byte as usize] as i32) << shift) as u32;
            byte -= 1;
            shift -= 8;
        }
        self.code = code;

        return 0;
    }

    pub(crate) fn clz_in_code(code: u32) -> isize {
        if code == 0 {
            return 32; /* to protect infinite loop */
        }

        let mut bits4: usize = 0;
        let mut clz: isize = 0;
        let mut shift = 28;

        while bits4 == 0 && shift >= 0 {
            bits4 = ((code >> shift) & 0xf) as usize;
            clz += tbl_zero_count4[bits4] as isize;
            shift -= 4;
        }
        return clz;
    }

    pub(crate) fn read(&mut self, mut size: isize, name: Option<&str>) -> u32 {
        let mut val = 0;

        assert!(size > 0);

        if self.leftbits < size {
            val = self.code >> (32 - size) as u32;
            size -= self.leftbits;

            if self.flush(4) != 0 {
                trace!("already reached the end of bitstream\n");
                return std::u32::MAX;
            }
        }
        val |= self.code >> (32 - size) as u32;

        self.EVC_BSR_SKIP_CODE(size as usize);

        if let Some(name) = name {
            EVC_TRACE_HLS(&mut self.tracer, name, val);
        }

        val
    }

    pub(crate) fn read1(&mut self, name: Option<&str>) -> u32 {
        if self.leftbits == 0 {
            if self.flush(4) != 0 {
                trace!("already reached the end of bitstream\n");
                return std::u32::MAX;
            }
        }
        let val = self.code >> 31;

        self.code <<= 1;
        self.leftbits -= 1;

        if let Some(name) = name {
            EVC_TRACE_HLS(&mut self.tracer, name, val);
        }

        val
    }

    pub(crate) fn read_ue(&mut self, name: Option<&str>) -> u32 {
        if (self.code >> 31) == 1 {
            /* early termination.
            we don't have to worry about leftbits == 0 case, because if the self.code
            is not equal to zero, that means leftbits is not zero */
            self.code <<= 1;
            self.leftbits -= 1;
            let val = 0;

            if let Some(name) = name {
                EVC_TRACE_HLS(&mut self.tracer, name, val);
            }

            return val;
        }

        let mut clz = 0;
        if self.code == 0 {
            clz = self.leftbits;

            self.flush(4);
        }

        let len = EvcdBsr::clz_in_code(self.code);

        clz += len;

        let val = if clz == 0 {
            /* early termination */
            self.code <<= 1;
            self.leftbits -= 1;
            0
        } else {
            assert!(self.leftbits >= 0);
            self.read(len + clz + 1, None) - 1
        };

        if let Some(name) = name {
            EVC_TRACE_HLS(&mut self.tracer, name, val);
        }

        val
    }

    pub(crate) fn read_se(&mut self, name: Option<&str>) -> i32 {
        let mut val = self.read_ue(None) as i32;

        val = if (val & 0x01) != 0 {
            (val + 1) >> 1
        } else {
            -(val >> 1)
        };

        if let Some(name) = name {
            EVC_TRACE_HLS(&mut self.tracer, name, val);
        }

        val
    }
}
