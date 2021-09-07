use criterion::*;
use fxhash::FxHashMap as HashMap;
use sphere_geometry::adjacency::units::Phi;
use sphere_geometry::adjacency::*;

criterion_main! { trig }

criterion_group! {
    trig,
    trig_cached,
    trig_calc,
}

const N: u16 = 256;

fn trig_cached(c: &mut Criterion) {
    let nodes = (0..N)
        .into_iter()
        .map(|n| Node::new(n, N))
        .collect::<Vec<_>>();

    let cos = nodes
        .iter()
        .map(|n| (*n, Phi::from(n.fraction())))
        .collect::<HashMap<_, _>>();

    c.bench_function("trig cached", |b| {
        b.iter(|| {
            for n in &nodes {
                black_box(cos[n]);
            }
        })
    });
}

fn trig_calc(c: &mut Criterion) {
    let nodes = (0..N)
        .into_iter()
        .map(|n| Node::new(n, N))
        .collect::<Vec<_>>();

    c.bench_function("trig calc", |b| {
        b.iter(|| {
            for n in &nodes {
                black_box(Phi::from(n.fraction()));
            }
        })
    });
}
