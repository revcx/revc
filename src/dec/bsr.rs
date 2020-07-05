use log::*;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;

use crate::api::EvcError;

type Tracer = (Box<dyn Write>, isize);

////////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(feature = "trace")]
fn OPEN_TRACE() -> Option<Tracer> {
    let fp_trace = OpenOptions::new()
        .append(true)
        .create(true)
        .open("dec_trace.txt");
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(not(feature = "trace"))]
fn OPEN_TRACE() -> Option<Tracer> {
    None
}
#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE_COUNTER(tracer: &mut Option<Tracer>) {}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE<T: Display>(writer: &mut Option<Tracer>, name: T) {}

#[cfg(not(feature = "trace"))]
pub(crate) fn EVC_TRACE_INT_HEX(tracer: &mut Option<Tracer>, val: isize) {}

#[cfg(not(feature = "trace_bin"))]
pub(crate) fn TRACE_BIN(tracer: &mut Option<Tracer>, model: u16, range: u32, lps: u32) {}
////////////////////////////////////////////////////////////////////////////////////////////////////

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
    pub(crate) tracer: Option<Tracer>,
}

/* Table of count of leading zero for 4 bit value */
static tbl_zero_count4: [u8; 16] = [4, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

impl EvcdBsr {
    #[inline]
    pub(crate) fn skip_code(&mut self, size: usize) {
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
    pub(crate) fn is_byte_aligned(&self) -> bool {
        (self.leftbits & 0x7) == 0
    }

    /* get number of byte consumed */
    #[inline]
    pub(crate) fn get_read_byte(&self) -> isize {
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

    pub(crate) fn flush(&mut self, mut byte: isize) -> Result<(), EvcError> {
        let mut shift: i32 = 24;
        let mut code: u32 = 0;

        assert_ne!(byte, 0);

        let remained = (self.buf.len() as isize - self.cur as isize);
        if byte > remained {
            byte = remained;
        }

        if byte <= 0 {
            self.code = 0;
            self.leftbits = 0;
            return Err(EvcError::EVC_ERR);
        }

        self.leftbits = byte << 3;

        self.cur += byte as usize;
        while byte != 0 {
            code |= ((self.buf[self.cur - byte as usize] as i32) << shift) as u32;
            byte -= 1;
            shift -= 8;
        }
        self.code = code;

        Ok(())
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

    pub(crate) fn read(&mut self, mut size: isize, name: Option<&str>) -> Result<u32, EvcError> {
        let mut val = 0;

        assert!(size > 0);

        if self.leftbits < size {
            val = self.code >> (32 - size) as u32;
            size -= self.leftbits;

            self.flush(4)?;
        }
        val |= self.code >> (32 - size) as u32;

        self.skip_code(size as usize);

        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        Ok(val)
    }

    pub(crate) fn read1(&mut self, name: Option<&str>) -> Result<u32, EvcError> {
        if self.leftbits == 0 {
            self.flush(4)?;
        }
        let val = self.code >> 31;

        self.code <<= 1;
        self.leftbits -= 1;

        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        Ok(val)
    }

    pub(crate) fn read_ue(&mut self, name: Option<&str>) -> Result<u32, EvcError> {
        if (self.code >> 31) == 1 {
            /* early termination.
            we don't have to worry about leftbits == 0 case, because if the self.code
            is not equal to zero, that means leftbits is not zero */
            self.code <<= 1;
            self.leftbits -= 1;
            let val = 0;

            if let Some(name) = name {
                EVC_TRACE(&mut self.tracer, name);
                EVC_TRACE(&mut self.tracer, " ");
                EVC_TRACE(&mut self.tracer, val);
                EVC_TRACE(&mut self.tracer, " \n");
            }

            return Ok(val);
        }

        let mut clz = 0;
        if self.code == 0 {
            clz = self.leftbits;

            self.flush(4)?;
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
            self.read(len + clz + 1, None)? - 1
        };

        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        Ok(val)
    }

    pub(crate) fn read_se(&mut self, name: Option<&str>) -> Result<i32, EvcError> {
        let mut val = self.read_ue(None)? as i32;

        val = if (val & 0x01) != 0 {
            (val + 1) >> 1
        } else {
            -(val >> 1)
        };

        if let Some(name) = name {
            EVC_TRACE(&mut self.tracer, name);
            EVC_TRACE(&mut self.tracer, " ");
            EVC_TRACE(&mut self.tracer, val);
            EVC_TRACE(&mut self.tracer, " \n");
        }

        Ok(val)
    }
}
