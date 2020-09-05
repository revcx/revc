use criterion::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use revc::bench::itdq::*;

criterion_group!(
    itdq,
    bench_itdq_2x2,
    bench_itdq_4x4,
    bench_itdq_8x8,
    bench_itdq_16x16,
    bench_itdq_32x32,
    bench_itdq_64x64,
);

fn bench_itdq_2x2(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..2 * 2).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_2x2", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}

fn bench_itdq_4x4(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..4 * 4).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_4x4", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}

fn bench_itdq_8x8(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..8 * 8).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_8x8", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}

fn bench_itdq_16x16(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..16 * 16).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_16x16", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}

fn bench_itdq_32x32(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..32 * 32).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_32x32", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}

fn bench_itdq_64x64(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let mut coef: Vec<i16> = (0..64 * 64).map(|_| ra.gen()).collect();

    c.bench_function("bench_itdq_64x64", move |b| {
        b.iter(|| evc_itdq(&mut coef[..], 1, 1, 816))
    });
}
