use std::io;
use std::mem;

#[inline(always)]
const fn num_bits<T>() -> usize {
    mem::size_of::<T>() * 8
}

#[inline(always)]
fn ulog2(v: u32) -> u32 {
    num_bits::<u32>() as u32 - 1 - v.leading_zeros()
}

#[inline(always)]
fn inv_recenter(r: u32, v: u32) -> u32 {
    if v > (r << 1) {
        v
    } else if (v & 1) == 0 {
        (v >> 1) + r
    } else {
        r - ((v + 1) >> 1)
    }
}

pub struct GetBits<'a> {
    error: bool,
    eof: bool,
    state: u64,
    bits_left: u32,
    data: &'a [u8],
    ptr: usize,
    ptr_end: usize,
}

impl<'a> GetBits<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        GetBits {
            error: false,
            eof: false,
            state: 0,
            bits_left: 0,
            data,
            ptr: 0,
            ptr_end: data.len(),
        }
    }

    fn refill(&mut self, n: u32) {
        debug_assert!(self.bits_left <= 56);
        let mut state: u64 = 0;
        let mut more = true;
        while more {
            state <<= 8;
            self.bits_left += 8;
            if !self.eof {
                state |= self.data[self.ptr] as u64;
                self.ptr += 1;
            }
            if self.ptr >= self.ptr_end {
                self.error = self.eof;
                self.eof = true;
            }
            more = n > self.bits_left;
        }
        self.state |= state << (64 - self.bits_left as u64);
    }

    pub fn check_error(&self) -> io::Result<()> {
        if self.error {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Error parsing frame header",
            ))
        } else {
            Ok(())
        }
    }

    // Check that we haven't read more than obu_len bytes from the buffer
    // since init_bit_pos.
    pub fn check_for_overrun(&self, init_bit_pos: u32, obu_len: u32) -> io::Result<()> {
        // Make sure we haven't actually read past the end of the gb buffer
        if self.error {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Overrun in OBU bit buffer",
            ));
        }

        let pos = self.get_bits_pos();

        // We assume that init_bit_pos was the bit position of the buffer
        // at some point in the past, so cannot be smaller than pos.
        debug_assert!(init_bit_pos <= pos);

        if pos - init_bit_pos > 8 * obu_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Overrun in OBU bit buffer into next OBU",
            ));
        }

        Ok(())
    }

    pub fn get_bits(&mut self, n: u32) -> u32 {
        debug_assert!(n <= 32 /* can go up to 57 if we change return type */);
        debug_assert!(n != 0 /* can't shift state by 64 */);

        if n > self.bits_left {
            self.refill(n);
        }

        let state = self.state;
        self.bits_left -= n;
        self.state <<= n as u64;

        return (state >> (64 - n as u64)) as u32;
    }

    pub fn get_sbits(&mut self, n: u32) -> i32 {
        let shift = 31 - n as i32;
        let res = (self.get_bits(n + 1) as i32) << shift;
        return res >> shift;
    }

    pub fn get_uleb128(&mut self) -> u32 {
        let (mut val, mut more, mut i) = (0, 1, 0);
        while more != 0 {
            more = self.get_bits(1);
            let bits = self.get_bits(7);
            if i <= 3 || (i == 4 && bits < 1 << 4) {
                val |= bits << (i * 7);
            } else if bits != 0 {
                self.error = true;
                return 0;
            }
            i += 1;
            if more != 0 && i == 8 {
                self.error = true;
                return 0;
            }
        }

        return val;
    }

    pub fn get_uniform(&mut self, max: u32) -> u32 {
        // Output in range [0..max-1]
        // max must be > 1, or else nothing is read from the bitstream
        debug_assert!(max > 1);
        let l = ulog2(max) + 1;
        debug_assert!(l > 1);
        let m = (1 << l) - max;
        let v = self.get_bits(l as u32 - 1);
        if v < m {
            v
        } else {
            (v << 1) - m + self.get_bits(1)
        }
    }

    pub fn get_vlc(&mut self) -> u32 {
        let mut n_bits = 0;
        while self.get_bits(1) == 0 {
            n_bits += 1;
            if n_bits == 32 {
                return 0xFFFFFFFF;
            }
        }

        if n_bits != 0 {
            ((1 << n_bits as u32) - 1) + self.get_bits(n_bits as u32)
        } else {
            0
        }
    }

    fn get_bits_subexp_u(&mut self, r: u32, n: u32) -> u32 {
        let mut v = 0;
        let mut i = 0;

        loop {
            let b = if i != 0 { 3 + i - 1 } else { 3 };

            if n < v + 3 * (1 << b) {
                v += self.get_uniform(n - v + 1);
                break;
            }

            if self.get_bits(1) == 0 {
                v += self.get_bits(b);
                break;
            }

            v += 1 << b;
            i += 1;
        }

        if r * 2 <= n {
            inv_recenter(r, v)
        } else {
            n - inv_recenter(n - r, v)
        }
    }

    pub fn get_bits_subexp(&mut self, r: i32, n: u32) -> i32 {
        self.get_bits_subexp_u((r + (1 << n) as i32) as u32, 2 << n) as i32 - (1 << n) as i32
    }

    pub fn bytealign_get_bits(&mut self) {
        // bits_left is never more than 7, because it is only incremented
        // by refill(), called by dav1d_get_bits and that never reads more
        // than 7 bits more than it needs.
        //
        // If this wasn't true, we would need to work out how many bits to
        // discard (bits_left % 8), subtract that from bits_left and then
        // shift state right by that amount.
        debug_assert!(self.bits_left <= 7);

        self.bits_left = 0;
        self.state = 0;
    }

    // Return the current bit position relative to the start of the buffer.
    pub fn get_bits_pos(&self) -> u32 {
        self.ptr as u32 * 8 - self.bits_left
    }
}
