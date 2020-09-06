use criterion::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

use revc::bench::df::*;
use revc::bench::frame::*;
use revc::bench::plane::*;

criterion_group!(
    df,
    bench_deblock_scu_hor_luma,
    bench_deblock_scu_hor_chroma,
    bench_deblock_scu_ver_luma,
    bench_deblock_scu_ver_chroma,
);

fn fill_plane<T: Pixel>(ra: &mut ChaChaRng, plane: &mut Plane<T>) {
    let stride = plane.cfg.stride;
    for row in plane.data_origin_mut().chunks_mut(stride) {
        for pixel in row {
            let v: u8 = ra.gen();
            *pixel = T::cast_from(v);
        }
    }
}

fn new_plane<T: Pixel>(ra: &mut ChaChaRng, width: usize, height: usize) -> Plane<T> {
    let mut p = Plane::new(width, height, 0, 0, 64 + 16, 64 + 16);

    fill_plane(ra, &mut p);

    p
}

fn bench_deblock_scu_hor_luma(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let w = 640;
    let h = 480;
    let cuw = 64;
    let cuh = 64;
    let qp = 27;
    let mut plane = new_plane::<u16>(&mut ra, w, h);
    let tbl = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
        2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11, 12, 12, 12, 12, 12,
    ];

    c.bench_function("deblock_scu_hor_luma", |b| {
        b.iter(|| {
            let _ = black_box(deblock_scu_hor_luma(
                &mut None,
                &mut plane.as_region_mut(),
                qp,
                0,
                tbl,
            ));
        })
    });
}

fn bench_deblock_scu_hor_chroma(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let w = 640;
    let h = 480;
    let cuw = 64;
    let cuh = 64;
    let qp = 27;
    let mut plane = new_plane::<u16>(&mut ra, w, h);
    let tbl = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
        2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11, 12, 12, 12, 12, 12,
    ];

    c.bench_function("deblock_scu_hor_chroma", |b| {
        b.iter(|| {
            let _ = black_box(deblock_scu_hor_chroma(
                &mut None,
                &mut plane.as_region_mut(),
                qp,
                1,
                tbl,
            ));
        })
    });
}

fn bench_deblock_scu_ver_luma(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let w = 640;
    let h = 480;
    let cuw = 64;
    let cuh = 64;
    let qp = 27;
    let mut plane = new_plane::<u16>(&mut ra, w, h);
    let tbl = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
        2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11, 12, 12, 12, 12, 12,
    ];

    c.bench_function("deblock_scu_ver_luma", |b| {
        b.iter(|| {
            let _ = black_box(deblock_scu_ver_luma(
                &mut None,
                &mut plane.as_region_mut(),
                qp,
                0,
                tbl,
            ));
        })
    });
}

fn bench_deblock_scu_ver_chroma(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let w = 640;
    let h = 480;
    let cuw = 64;
    let cuh = 64;
    let qp = 27;
    let mut plane = new_plane::<u16>(&mut ra, w, h);
    let tbl = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
        2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11, 12, 12, 12, 12, 12,
    ];

    c.bench_function("deblock_scu_ver_chroma", |b| {
        b.iter(|| {
            let _ = black_box(deblock_scu_ver_chroma(
                &mut None,
                &mut plane.as_region_mut(),
                qp,
                1,
                tbl,
            ));
        })
    });
}
