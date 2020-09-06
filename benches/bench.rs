use criterion::*;

cfg_if::cfg_if! {
    if #[cfg(feature="bench")] {
        mod df;
        mod mc;
        mod transform;

        criterion_main!(df::df, mc::mc, transform::itdq);
    } else {
        fn bench_no_op(_: &mut Criterion) {
        }
        criterion_group!(
            no_op,
            bench_no_op,
        );
        criterion_main!(no_op);
    }
}
