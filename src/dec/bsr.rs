use log::*;

/*
 * bitstream structure for decoder.
 *
 * NOTE: Don't change location of variable because this variable is used
 *       for assembly coding!
 */
pub(crate) struct EvcdBsr<'a> {
    /* temporary read code buffer */
    code: u32,
    /* left bits count in code */
    leftbits: isize,
    /* bitstream cur position */
    cur: usize,
    /* buffer */
    buf: &'a [u8],
}

/* Table of count of leading zero for 4 bit value */
static tbl_zero_count4: [u8; 16] = [4, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

impl<'a> EvcdBsr<'a> {
    #[inline]
    fn EVC_BSR_SKIP_CODE(&mut self, size: usize) {
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
    fn EVC_BSR_IS_BYTE_ALIGN(&self) -> bool {
        (self.leftbits & 0x7) == 0
    }

    /* get number of byte consumed */
    #[inline]
    fn EVC_BSR_GET_READ_BYTE(&self) -> isize {
        self.cur as isize - (self.leftbits >> 3)
    }

    pub fn new(buf: &'a [u8]) -> Self {
        EvcdBsr {
            code: 0,
            leftbits: 0,
            cur: 0,
            buf,
        }
    }

    pub fn flush(&mut self, mut byte: isize) -> isize {
        let mut shift: isize = 24;
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
            code |= (self.buf[self.cur - byte as usize] as u32) << shift as u32;
            byte -= 1;
            shift -= 8;
            assert!(shift >= 0);
        }
        self.code = code;

        return 0;
    }

    pub fn clz_in_code(code: u32) -> isize {
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

    pub fn read(&mut self, mut size: isize) -> u32 {
        let mut code = 0;

        assert!(size > 0);

        if self.leftbits < size {
            code = self.code >> (32 - size) as u32;
            size -= self.leftbits;

            if self.flush(4) != 0 {
                trace!("already reached the end of bitstream\n");
                return std::u32::MAX;
            }
        }
        code |= self.code >> (32 - size) as u32;

        self.EVC_BSR_SKIP_CODE(size as usize);

        code
    }

    pub fn read1(&mut self) -> u32 {
        if self.leftbits == 0 {
            if self.flush(4) != 0 {
                trace!("already reached the end of bitstream\n");
                return std::u32::MAX;
            }
        }
        let code = self.code >> 31;

        self.code <<= 1;
        self.leftbits -= 1;

        code
    }

    pub fn read_ue(&mut self) -> u32 {
        if (self.code >> 31) == 1 {
            /* early termination.
            we don't have to worry about leftbits == 0 case, because if the self.code
            is not equal to zero, that means leftbits is not zero */
            self.code <<= 1;
            self.leftbits -= 1;
            return 0;
        }

        let mut clz = 0;
        if self.code == 0 {
            clz = self.leftbits;

            self.flush(4);
        }

        let len = EvcdBsr::clz_in_code(self.code);

        clz += len;

        if clz == 0 {
            /* early termination */
            self.code <<= 1;
            self.leftbits -= 1;
            return 0;
        }

        assert!(self.leftbits >= 0);

        self.read(len + clz + 1) - 1
    }

    pub fn read_se(&mut self) -> i32 {
        let mut val = self.read_ue() as i32;

        if (val & 0x01) != 0 {
            (val + 1) >> 1
        } else {
            -(val >> 1)
        }
    }
}
