use log::*;

use crate::api::*;
use crate::tracer::*;

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
    pkt: Packet,
    /* tracer */
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

    pub(crate) fn new(pkt: Packet) -> Self {
        EvcdBsr {
            code: 0,
            leftbits: 0,
            cur: 0,
            pkt,
            tracer: OPEN_TRACE(false),
        }
    }

    pub(crate) fn flush(&mut self, mut byte: isize) -> Result<(), EvcError> {
        let mut shift: i32 = 24;
        let mut code: u32 = 0;

        assert_ne!(byte, 0);

        let remained = (self.pkt.data.len() as isize - self.cur as isize);
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
            code |= ((self.pkt.data[self.cur - byte as usize] as i32) << shift) as u32;
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
