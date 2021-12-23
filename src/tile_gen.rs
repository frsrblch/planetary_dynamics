use crate::adjacency::{get_tile_count, AdjArray, Adjacency};
use crate::terrain::Terrain;
use physics_types::Length;
use rand::distributions::Bernoulli;
use rand::prelude::{Distribution, Rng, SliceRandom};
use std::collections::HashSet;
use std::ops::AddAssign;

#[derive(Debug, Default, Copy, Clone)]
pub struct TileGen {
    pub water_fraction: f64,
}

impl TileGen {
    pub fn generate<R: Rng>(
        &self,
        radius: Length,
        adjacency: &Adjacency,
        rng: &mut R,
    ) -> Vec<Terrain> {
        generate_terrain_from_radius(radius, self.water_fraction, adjacency, rng)
    }
}

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

pub fn generate_terrain_from_radius<R: Rng>(
    radius: Length,
    water_fraction: f64,
    adjacency: &Adjacency,
    rng: &mut R,
) -> Vec<Terrain> {
    let tiles = get_tile_count(radius);
    generate_terrain(tiles, water_fraction, adjacency, rng)
}

pub fn generate_terrain<R: Rng>(
    nodes: usize,
    water_fraction: f64,
    adjacency: &Adjacency,
    rng: &mut R,
) -> Vec<Terrain> {
    let plate_type = WaterFraction::new(water_fraction);

    let adjacency = adjacency.get(nodes);

    loop {
        let continent_count = rng.gen_range(10.min(nodes)..14.min(nodes));
        let iter_continents = || (0..continent_count).map(Continent);
        let mut neighbours = HashSet::<usize>::new();

        let mut unassigned_count = nodes;
        let mut tiles = vec![Option::<Continent>::None; nodes];

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

#[derive(Debug, Copy, Clone)]
struct Continent(usize);

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
    use rand::thread_rng;

    #[test]
    fn tiles_test() {
        const N: usize = 128;
        let rng = &mut thread_rng();

        let mut adj = Adjacency::default();
        adj.register(N);

        use std::time::Instant;
        let start = Instant::now();
        generate_terrain(N, 0.5, &adj, rng);
        let end = Instant::now();

        println!("done: {} us", (end - start).as_micros());

        // panic!("end");
    }

    #[test]
    fn tile_gen_for_zero_water() {
        const N: usize = 32;
        let rng = &mut thread_rng();
        let mut adj = Adjacency::default();
        adj.register(N);
        generate_terrain(N, 0.0, &adj, rng);
    }

    #[test]
    fn tile_gen_for_one_water() {
        const N: usize = 32;
        let rng = &mut thread_rng();
        let mut adj = Adjacency::default();
        adj.register(N);
        generate_terrain(N, 1.0, &adj, rng);
    }

    #[test]
    #[should_panic]
    fn tile_gen_for_out_of_bounds_water() {
        const N: usize = 32;
        let rng = &mut thread_rng();
        let mut adj = Adjacency::default();
        adj.register(N);
        generate_terrain(N, 1.1, &adj, rng);
    }

    #[test]
    fn water_fraction() {
        let rng = &mut thread_rng();
        assert_eq!(ContinentType::Land, WaterFraction::new(0.0).sample(rng));
        assert_eq!(ContinentType::Ocean, WaterFraction::new(1.0).sample(rng));
    }
}
