cfg_if::cfg_if! {
    if #[cfg(feature="bench")] {
        use criterion::*;
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaChaRng;

        use revc::bench::frame::*;
        use revc::bench::mc::*;
        use revc::bench::plane::*;

        criterion_group!(
            mc,
            bench_evc_mc_l_00,
            bench_evc_mc_l_n0,
            bench_evc_mc_l_0n,
            bench_evc_mc_l_nn,
            bench_evc_mc_c_00,
            bench_evc_mc_c_n0,
            bench_evc_mc_c_0n,
            bench_evc_mc_c_nn,
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

        fn bench_evc_mc_l_00(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 64;
            let cuh = 64;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 64 * 64]>::uninitialized();

            c.bench_function("evc_mc_l_00", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_l(
                        0,
                        0,
                        &plane,
                        0,
                        0,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_l_n0(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 64;
            let cuh = 64;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 64 * 64]>::uninitialized();

            c.bench_function("evc_mc_l_n0", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_l(
                        1,
                        0,
                        &plane,
                        1,
                        0,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_l_0n(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 64;
            let cuh = 64;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 64 * 64]>::uninitialized();

            c.bench_function("evc_mc_l_0n", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_l(
                        0,
                        1,
                        &plane,
                        0,
                        1,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_l_nn(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 64;
            let cuh = 64;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 64 * 64]>::uninitialized();

            c.bench_function("evc_mc_l_nn", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_l(
                        1,
                        1,
                        &plane,
                        1,
                        1,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_c_00(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 32;
            let cuh = 32;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 32 * 32]>::uninitialized();

            c.bench_function("evc_mc_c_00", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_c(
                        0,
                        0,
                        &plane,
                        0,
                        0,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_c_n0(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 32;
            let cuh = 32;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 32 * 32]>::uninitialized();

            c.bench_function("evc_mc_c_n0", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_c(
                        1,
                        0,
                        &plane,
                        1,
                        0,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_c_0n(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 32;
            let cuh = 32;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 32 * 32]>::uninitialized();

            c.bench_function("evc_mc_c_0n", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_c(
                        0,
                        1,
                        &plane,
                        0,
                        1,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }

        fn bench_evc_mc_c_nn(c: &mut Criterion) {
            let mut ra = ChaChaRng::from_seed([0; 32]);
            let w = 640;
            let h = 480;
            let cuw = 32;
            let cuh = 32;
            let plane = new_plane::<u16>(&mut ra, w, h);
            let mut pred = Aligned::<[u16; 32 * 32]>::uninitialized();

            c.bench_function("evc_mc_c_nn", |b| {
                b.iter(|| {
                    let _ = black_box(evc_mc_c(
                        1,
                        1,
                        &plane,
                        1,
                        1,
                        &mut pred.data,
                        cuw as i16,
                        cuh as i16,
                    ));
                })
            });
        }
    }
}
