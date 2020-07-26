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
