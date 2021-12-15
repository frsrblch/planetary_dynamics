use fractional_int::FractionalU8;
use std::ops::Sub;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Terrain {
    /// The fraction covered by ocean, counted from the 'left'
    pub ocean: FractionalU8,
    /// The fraction covered by ocean, counted from the 'right'
    pub mountains: FractionalU8,
    /// The fraction covered by plains, counted oceans on the 'left' and mountains on the 'right'
    pub plains: FractionalU8,
    /// The fraction covered by glacier, counted from the 'right'
    /// Mountains will be covered before plains, which are covered before oceans.
    pub glacier: FractionalU8,
}

impl Terrain {
    /// Generates a terrain from the given fractions.
    ///
    /// # Arguments
    ///
    /// * `ocean`: the fraction of the tile covered by water
    /// * `mountain`: the fraction of the land covered by mountains
    /// * `glacier`: the fraction of the tile covered by glacier.
    ///
    /// returns: Terrain
    ///
    /// # Examples
    ///
    /// ```
    /// use planetary_dynamics::tiles::Terrain;
    /// let pacific = Terrain::new_fraction(0.97, 0.6, 0.0);
    /// let arizona = Terrain::new_fraction(0.0, 0.25, 0.0);
    /// let arctic = Terrain::new_fraction(0.8, 0.5, 0.8);
    /// ```
    pub fn new_fraction(ocean: f64, mountains: f64, glacier: f64) -> Self {
        let ocean = FractionalU8::new_f64(ocean);
        let land = ocean.inverse();

        let mountains = FractionalU8::new_f64(mountains * land.f64());
        let plains = land - mountains;

        let glacier = FractionalU8::new_f64(glacier);

        debug_assert_eq!(u8::MAX, ocean.u8() + plains.u8() + mountains.u8());

        Self {
            ocean,
            plains,
            mountains,
            glacier,
        }
    }

    #[inline]
    pub fn new(ocean: u8, mountains: u8, glacier: u8) -> Self {
        let plains = 255u8
            .sub(mountains)
            .checked_sub(ocean)
            .expect("mountains + ocean > 255");

        Self {
            ocean: FractionalU8::new(ocean),
            mountains: FractionalU8::new(mountains),
            plains: FractionalU8::new(plains),
            glacier: FractionalU8::new(glacier),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_fraction_fuzz() {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();

        for _ in 0..1000 {
            Terrain::new_fraction(rng.gen(), rng.gen(), rng.gen());
        }
    }

    #[test]
    #[should_panic]
    fn tile_details_new_given_ocean_mountain_gt_255() {
        Terrain::new(200, 56, 0);
    }

    #[test]
    fn tile_details_new_given_ocean_mountain_le_255() {
        Terrain::new(200, 55, 0);
    }
}
