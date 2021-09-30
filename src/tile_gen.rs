use crate::adjacency::{AdjArray, Adjacency};
use crate::tiles::Terrain;
use rand::distributions::Bernoulli;
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::iter::Enumerate;
use std::ops::AddAssign;

// #[derive(Debug, Copy, Clone, PartialEq)]
// struct Plate {
//     pub plate_type: ContinentType,
//     pub roll: Roll,
// }

#[derive(Debug, Copy, Clone, PartialEq)]
enum ContinentType {
    Land,
    Ocean,
}

struct WaterFraction(Bernoulli);

impl WaterFraction {
    fn new(fraction: f64) -> Self {
        assert!((0.0..=1.0).contains(&fraction));
        Self(Bernoulli::new(fraction).unwrap())
    }
}

impl Distribution<ContinentType> for WaterFraction {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ContinentType {
        if self.0.sample(rng) {
            ContinentType::Ocean
        } else {
            ContinentType::Land
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Continent(usize);

impl Display for Continent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "C{}", self.0)
    }
}

// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
// struct Roll(i32);

// impl Roll {
//     fn distance(&self, rhs: Self) -> i32 {
//         use std::ops::Sub;
//         self.0.sub(rhs.0).abs()
//     }
// }

// impl Distribution<Roll> for Standard {
//     fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Roll {
//         Roll(rng.gen_range(0..1000))
//     }
// }

struct Continents<I> {
    enumerate: Enumerate<I>,
}

impl<I> Iterator for Continents<I>
where
    I: Iterator,
{
    type Item = (Continent, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.enumerate
            .next()
            .map(|(i, plate)| (Continent(i), plate))
    }
}

trait IteratorExt: Sized {
    fn continents(self) -> Continents<Self>;
}

impl<'a, I: Iterator> IteratorExt for I {
    fn continents(self) -> Continents<Self> {
        Continents {
            enumerate: self.enumerate(),
        }
    }
}

pub fn create_tiles(nodes: usize, water_fraction: f64, adjacency: &Adjacency) -> Vec<Terrain> {
    let plate_type = WaterFraction::new(water_fraction);

    let adjacency = adjacency.get(nodes);

    let rng = &mut thread_rng();

    loop {
        let continent_count = rng.gen_range(10.min(nodes)..12.min(nodes));
        let iter_continents = || (0..continent_count).map(Continent);
        let mut neighbours = HashSet::<usize>::new();

        let mut tiles = vec![Option::<Continent>::None; nodes];
        let mut unassigned_count = nodes;

        for continent in iter_continents() {
            let tile = random_none(rng, &tiles);
            assign_tile(
                &mut tiles,
                &mut unassigned_count,
                &mut neighbours,
                adjacency,
                tile,
                continent,
            );
        }

        while unassigned_count > 0 {
            if let Some(tile) = random_adjacent_tile(rng, &neighbours) {
                if let Some(continent) = random_adjacent_continent(rng, tile, &tiles, adjacency) {
                    assign_tile(
                        &mut tiles,
                        &mut unassigned_count,
                        &mut neighbours,
                        adjacency,
                        tile,
                        continent,
                    );
                }
            }
        }

        for c in iter_continents() {
            tiles.iter().enumerate().for_each(|(i, t)| {
                let t = t.unwrap();
                if t == c {
                    println!("{}: {}", i, c);
                }
            });
            println!();
        }

        // loop many times to make these continents
        for _ in 0..20 {
            let continent_types = iter_continents()
                .map(|_| plate_type.sample(rng))
                .collect::<Vec<_>>();

            let water_tiles = tiles
                .iter()
                .filter_map(|t| *t)
                .filter(|t| continent_types[t.0] == ContinentType::Ocean)
                .count();

            let result_fraction = water_tiles as f64 / nodes as f64;
            if (result_fraction - water_fraction).abs() < 0.03 {
                continent_types
                    .iter()
                    .continents()
                    .for_each(|(c, t)| println!("{}: {:?}", c, *t));

                return tiles
                    .iter()
                    .enumerate()
                    .map(|(i, t)| match continent_types[t.unwrap().0] {
                        ContinentType::Land => Terrain::new_fraction(
                            rng.gen_range(0.0..0.05),
                            rng.gen_range(0.1..0.25),
                            0.0,
                        ),
                        ContinentType::Ocean => {
                            let (ocean, count) = adjacency[i]
                                .iter()
                                .filter_map(|neighbour| tiles[neighbour])
                                .fold((0u8, 0u8), |(mut ocean, mut count), c| {
                                    if let ContinentType::Ocean = continent_types[c.0] {
                                        ocean.add_assign(1);
                                    }
                                    count.add_assign(1);

                                    (ocean, count)
                                });

                            let ocean_fraction = ocean as f64 / count as f64;
                            let island_chance = 0.4 - 0.2 * ocean_fraction;
                            let has_island = rng.gen_bool(island_chance);

                            if has_island {
                                let non_zero_ratio = (ocean + 1) as f64 / (count + 1) as f64;
                                let ocean_min = 1.0 - non_zero_ratio * 0.025;
                                Terrain::new_fraction(
                                    rng.gen_range(ocean_min..1.0),
                                    rng.gen_range(0.4..0.8),
                                    0.0,
                                )
                            } else {
                                Terrain::new(255, 0, 0)
                            }
                        }
                    })
                    .collect();
            }
        }
    }
}

fn random_none<R: Rng, T>(rng: &mut R, slice: &[Option<T>]) -> usize {
    debug_assert!(slice.iter().any(|c| c.is_none()));
    loop {
        let index = rng.gen_range(0..slice.len());
        if slice[index].is_none() {
            return index;
        }
    }
}

fn random_adjacent_tile<R: Rng + ?Sized>(
    rng: &mut R,
    neighbours: &HashSet<usize>,
) -> Option<usize> {
    use rand::prelude::IteratorRandom;
    neighbours.iter().choose(rng).copied()
}

fn random_adjacent_continent<R: Rng>(
    rng: &mut R,
    tile: usize,
    tiles: &[Option<Continent>],
    adjacency: &[AdjArray],
) -> Option<Continent> {
    let adjacent = adjacency[tile]
        .iter()
        .filter_map(|t| tiles[t])
        .collect::<Vec<_>>();
    adjacent.choose(rng).copied()
}

fn assign_tile(
    tiles: &mut [Option<Continent>],
    unassigned_count: &mut usize,
    neighbours: &mut HashSet<usize>,
    adjacency: &[AdjArray],
    tile: usize,
    continent: Continent,
) {
    for n in adjacency[tile].iter() {
        if tiles[n].is_none() {
            neighbours.insert(n);
        }
    }
    neighbours.remove(&tile);

    tiles[tile] = Some(continent);
    *unassigned_count -= 1;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tiles_test() {
        const N: usize = 128;

        let mut adj = Adjacency::default();
        adj.register(N);

        use std::time::Instant;
        let start = Instant::now();
        create_tiles(N, 0.5, &mut adj);
        let end = Instant::now();

        println!("done: {} us", (end - start).as_micros());

        // panic!("end");
    }

    #[test]
    fn tile_gen_for_zero_water() {
        const N: usize = 32;
        let mut adj = Adjacency::default();
        adj.register(N);
        create_tiles(N, 0.0, &mut adj);
    }

    #[test]
    fn tile_gen_for_one_water() {
        const N: usize = 32;
        let mut adj = Adjacency::default();
        adj.register(N);
        create_tiles(N, 1.0, &mut adj);
    }

    #[test]
    #[should_panic]
    fn tile_gen_for_out_of_bounds_water() {
        const N: usize = 32;
        let mut adj = Adjacency::default();
        adj.register(N);
        create_tiles(N, 1.1, &mut adj);
    }

    #[test]
    fn water_fraction() {
        let rng = &mut thread_rng();
        assert_eq!(ContinentType::Land, WaterFraction::new(0.0).sample(rng));
        assert_eq!(ContinentType::Ocean, WaterFraction::new(1.0).sample(rng));
    }
}
