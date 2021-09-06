use criterion::*;
use sphere_geometry::tiles::fraction::FractionU8;
use std::ops::AddAssign;

criterion_main! {
    conversion
}

criterion_group! {
    conversion,
    u8_to_f64,
}

fn u8_to_f64(c: &mut Criterion) {
    let u8_fraction = FractionU8::new(0.2);
    let u8s = [FractionU8::new(1.0); 1024];
    let mut f64s = [0f64; 1024];
    let other_f64s = [1f64; 1024];
    c.bench_function("u8_to_f64", |b| {
        b.iter(|| {
            black_box(u8_fraction.f64());
        })
    })
    .bench_function("u8_to_raw_f64", |b| {
        b.iter(|| {
            black_box(u8_fraction.raw_f64());
        })
    })
    .bench_function("u8_inverse_f64", |b| {
        b.iter(|| {
            black_box(u8_fraction.inverse().f64());
        })
    })
    .bench_function("f64_mul", |b| {
        b.iter(|| {
            black_box(2f64 * 3f64);
        })
    })
    .bench_function("simd_add_assign_u8s_to_f64s", |b| {
        b.iter(|| {
            f64s.iter_mut().zip(&u8s).for_each(|(f, u)| {
                f.add_assign(u.f64());
            });
        })
    })
    .bench_function("simd_add_assign_f64s_to_f64s", |b| {
        b.iter(|| {
            f64s.iter_mut().zip(&other_f64s).for_each(|(f, u)| {
                f.add_assign(u);
            });
        })
    });
}
