use criterion::*;

mod df;
mod mc;
mod transform;

criterion_main!(df::df, mc::mc, transform::itdq);
