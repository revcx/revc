use criterion::*;

mod mc;
mod transform;

criterion_main!(mc::mc, transform::itdq);
