use crate::tiles::fraction::FractionU8;

/// Representation of a terrain tile on a planet
pub struct TileDetails {
    /// The fraction not covered by water
    pub land: FractionU8,
    /// The fraction of usable terrain (i.e., not mountains, cliffs, etc.)
    pub plains: FractionU8,
    /// The fraction covered by thick ice sheets
    pub glacier: FractionU8,
    /// The fraction of snow cover for calculating albedo
    /// Is this a temporary value, and I should just have an albedo value?
    pub snow: FractionU8,
}

impl TileDetails {
    pub fn new(land: f64, plains: f64, glacier: f64, snow: f64) -> Self {
        TileDetails {
            land: land.into(),
            plains: plains.into(),
            glacier: glacier.into(),
            snow: snow.into(),
        }
    }

    pub fn land(&self) -> f64 {
        self.land.f64()
    }

    pub fn ocean(&self) -> f64 {
        self.land.inverse().f64()
    }

    pub fn plains(&self) -> f64 {
        self.land.raw_f64() * self.plains.raw_f64() * FractionU8::INVERSE_SQUARED
    }

    pub fn mountains(&self) -> f64 {
        self.land.raw_f64() * self.plains.inverse().raw_f64() * FractionU8::INVERSE_SQUARED
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn round_to_percent(value: f64) -> f64 {
        (value * 100.0).round() * 0.01
    }

    #[test]
    fn tile_to_fractions() {
        let tile = TileDetails::new(0.8, 0.4, 0.0, 0.0);

        assert_eq!(0.8, tile.land());
        assert_eq!(0.2, tile.ocean());
        assert_eq!(round_to_percent(0.8 * 0.4), round_to_percent(tile.plains()));
        assert_eq!(
            round_to_percent(0.8 * 0.6),
            round_to_percent(tile.mountains())
        );
    }
}

pub mod fraction {
    use std::ops::{Add, AddAssign, Sub, SubAssign};

    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct FractionU8(u8);

    impl From<f64> for FractionU8 {
        #[inline]
        fn from(value: f64) -> Self {
            FractionU8::new(value)
        }
    }

    impl From<FractionU8> for f64 {
        #[inline]
        fn from(value: FractionU8) -> Self {
            value.f64()
        }
    }

    impl FractionU8 {
        pub const ZERO: Self = Self(0);
        pub const MAX: Self = Self(u8::MAX);
        pub const INVERSE: f64 = 1.0 / u8::MAX as f64;
        pub const INVERSE_SQUARED: f64 = Self::INVERSE * Self::INVERSE;

        #[inline]
        pub fn new(value: f64) -> Self {
            let value = value.max(0.0).min(1.0) * 255.0;
            Self(value as u8)
        }

        #[inline]
        pub fn raw_f64(self) -> f64 {
            self.0 as f64
        }

        #[inline]
        pub fn f64(self) -> f64 {
            self.raw_f64() * Self::INVERSE
        }

        #[inline]
        pub fn inverse(self) -> Self {
            Self(255 - self.0)
        }
    }

    impl Add<u8> for FractionU8 {
        type Output = Self;

        #[inline]
        fn add(self, rhs: u8) -> Self::Output {
            Self(self.0.saturating_add(rhs))
        }
    }

    impl AddAssign<u8> for FractionU8 {
        #[inline]
        fn add_assign(&mut self, rhs: u8) {
            *self = *self + rhs;
        }
    }

    impl Sub<u8> for FractionU8 {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: u8) -> Self::Output {
            Self(self.0.saturating_sub(rhs))
        }
    }

    impl SubAssign<u8> for FractionU8 {
        #[inline]
        fn sub_assign(&mut self, rhs: u8) {
            *self = *self - rhs;
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn fraction_u8_new() {
            assert_eq!(FractionU8::ZERO, FractionU8::new(-1.0));
            assert_eq!(FractionU8::ZERO, FractionU8::new(0.0));
            assert_eq!(127, FractionU8::new(0.5).0);
            assert_eq!(FractionU8::MAX, FractionU8::new(1.0));
            assert_eq!(FractionU8::MAX, FractionU8::new(2.0));
            assert_eq!(FractionU8::ZERO, FractionU8::new(f64::NAN));
        }

        #[test]
        fn fraction_u8_add() {
            assert_eq!(1, (FractionU8::default() + 1).0);
            assert_eq!(FractionU8::MAX, (FractionU8::new(1.0) + 1));
        }

        #[test]
        fn fraction_u8_sub() {
            assert_eq!(FractionU8::ZERO, (FractionU8::default() - 1));
            assert_eq!(254, (FractionU8::new(1.0) - 1).0);
        }

        #[test]
        fn into_f64() {
            assert_eq!(0.0, f64::from(FractionU8::new(0.0)));
            assert_eq!(1.0, f64::from(FractionU8::new(1.0)));
        }
    }
}
