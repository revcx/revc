use criterion::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

use revc::bench::frame::*;
use revc::bench::mc::*;
use revc::bench::plane::*;

criterion_group!(
    mc,
    bench_evc_mc_l_00 /* bench_evc_mc_l_n0,
                      bench_evc_mc_l_0n,
                      bench_evc_mc_l_nn,
                      bench_evc_mc_c_00,
                      bench_evc_mc_c_n0,
                      bench_evc_mc_c_0n,
                      bench_evc_mc_c_nn*/
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
    let mut p = Plane::new(width, height, 0, 0, 128 + 8, 128 + 8);

    fill_plane(ra, &mut p);

    p
}

fn bench_evc_mc_l_00(c: &mut Criterion) {
    let mut ra = ChaChaRng::from_seed([0; 32]);
    let w = 640;
    let h = 480;
    let cuw = 8;
    let cuh = 8;
    let plane = new_plane::<u16>(&mut ra, w, h);
    let mut pred = vec![0; cuw * cuh];

    c.bench_function("evc_mc_l_00", |b| {
        b.iter(|| {
            let _ = black_box(evc_mc_l(
                0, 0, &plane, 0, 0, &mut pred, cuw as i16, cuh as i16,
            ));
        })
    });
}
