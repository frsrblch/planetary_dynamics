use crate::solar_radiation::RadiativeAbsorption;
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

    pub fn absorption(
        &self,
        ground: RadiativeAbsorption,
        clouds: FractionalU8,
    ) -> RadiativeAbsorption {
        let iceless_ocean = (!self.glacier).min(self.ocean);
        let iceless_ground = self.plains + self.mountains - self.glacier;

        let glacier = RadiativeAbsorption::ICE * self.glacier;
        let ocean = RadiativeAbsorption::WATER * iceless_ocean;
        let land = ground * iceless_ground;

        let surface = glacier.add(ocean).add(land) * !clouds;
        let clouds = RadiativeAbsorption::CLOUD * clouds;

        surface.add(clouds)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::solar_radiation::Albedo;

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

    #[test]
    fn earth_albedo() {
        use std::ops::Not;

        let tile = Terrain::new_fraction(0.7, 0.24, 0.03);
        let absorption = tile.absorption(Albedo::new(0.18).not(), FractionalU8::new_f64(0.51));

        let min = RadiativeAbsorption::new(0.69);
        let max = RadiativeAbsorption::new(0.71);

        assert!(absorption < max, "{:.2} < {:.2}", absorption.0, max.0);
        assert!(absorption > min, "{:.2} > {:.2}", absorption.0, min.0);
    }
}
