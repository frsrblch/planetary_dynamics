use criterion::{criterion_group, criterion_main, Criterion};
use fractional_int::FractionalU8;
use planetary_dynamics::solar_radiation::RadiativeAbsorption;
use planetary_dynamics::tiles::Terrain;
use std::iter::FromIterator;

criterion_main! {
    absorption,
}

criterion_group! {
    absorption,
    terrain_absorption, // 14 ns
}

const N: usize = 1024;

pub fn terrain_absorption(c: &mut Criterion) {
    let tiles = vec![Terrain::new_fraction(0.25, 0.25, 0.5); N];
    let mut abs = vec![RadiativeAbsorption::default(); N];
    let ra = RadiativeAbsorption::new(0.2);
    let clouds = FractionalU8::new(64);

    c.bench_function("terrain_absorption", |b| {
        b.iter(|| {
            abs.iter_mut().zip(tiles.iter()).for_each(|(abs, tile)| {
                *abs = tile.absorption(ra, clouds);
            });
        })
    });
}
