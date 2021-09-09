use criterion::*;
use planetary_dynamics::tiles::fraction::UnitInterval;
use planetary_dynamics::tiles::TileDetails;
use std::ops::AddAssign;

criterion_main! {
    conversion
}

criterion_group! {
    conversion,
    u8_u16_comparison,
}

fn u8_to_f64(c: &mut Criterion) {
    let u8_fraction = UnitInterval::<u8>::new(0.2);
    let u8s = [UnitInterval::<u8>::new(1.0); 1024];
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

fn u8_u16_comparison(c: &mut Criterion) {
    use num_traits::*;
    use std::ops::Mul;

    const N: usize = 1024 * 1024;

    let details = vec![TileDetails::new(0.1, 0.2, 0.3, 0.4); N];
    let x = vec![2u8; N];
    let x16 = vec![2u16; N];
    let x32 = vec![2u32; N];
    let y = vec![3u8; N];
    let y16 = vec![3u16; N];
    let y32 = vec![3u32; N];
    let mut f = vec![0f32; N];
    let f0 = vec![2f32 / 255f32; N];
    let f1 = vec![3f32 / 255f32; N];
    const RECIP_SQUARED_8: f32 = 1.0 / u16::MAX as f32;
    const RECIP_SQUARED_16: f32 = 1.0 / u32::MAX as f32;
    const RECIP_SQUARED_32: f32 = 1.0 / u64::MAX as f32;

    c.bench_function("u8 to f32 mul", |b| {
        b.iter(|| {
            f.iter_mut()
                .zip(x.iter())
                .zip(y.iter())
                .for_each(|((f, x), y)| {
                    f.add_assign(x.to_f32().unwrap() * y.to_f32().unwrap() * RECIP_SQUARED_8);
                })
        })
    })
    .bench_function("u16 to f32 mul", |b| {
        b.iter(|| {
            f.iter_mut()
                .zip(x16.iter())
                .zip(y16.iter())
                .for_each(|((f, x), y)| {
                    f.add_assign(x.to_f32().unwrap() * y.to_f32().unwrap() * RECIP_SQUARED_16);
                })
        })
    })
    .bench_function("u32 to f32 mul", |b| {
        b.iter(|| {
            f.iter_mut()
                .zip(x32.iter())
                .zip(y32.iter())
                .for_each(|((f, x), y)| {
                    f.add_assign(x.to_f32().unwrap() * y.to_f32().unwrap() * RECIP_SQUARED_32);
                })
        })
    })
    .bench_function("u8 to u16 mul to f32", |b| {
        b.iter(|| {
            f.iter_mut()
                .zip(x.iter())
                .zip(y.iter())
                .for_each(|((f, x), y)| {
                    f.add_assign(
                        x.to_u16()
                            .unwrap()
                            .mul(y.to_u16().unwrap())
                            .to_f32()
                            .unwrap()
                            * RECIP_SQUARED_8,
                    );
                })
        })
    })
    .bench_function("f32 add to f32", |b| {
        b.iter(|| {
            f.iter_mut()
                .zip(f0.iter())
                .zip(f1.iter())
                .for_each(|((f, f0), f1)| {
                    f.add_assign(f0.mul(f1));
                })
        })
    })
    .bench_function("struct u8 to f32", |b| {
        const RECIP: f32 = 1.0 / u16::MAX as f32;
        b.iter(|| {
            f.iter_mut().zip(details.iter()).for_each(|(f, details)| {
                f.add_assign(details.land.byte() as f32 * details.plains.byte() as f32 * RECIP);
            })
        })
    });
}
