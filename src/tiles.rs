use crate::tiles::fraction::UnitInterval;
use std::ops::Not;

/// Representation of a terrain tile on a planet
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TileDetails {
    /// The fraction not covered by water
    pub land: UnitInterval<u8>,
    /// The fraction of usable terrain (i.e., not mountains, cliffs, etc.)
    pub plains: UnitInterval<u8>,
    /// The fraction covered by thick ice sheets
    pub glacier: UnitInterval<u8>,
    /// The fraction of snow cover for calculating albedo
    /// Is this a temporary value, and I should just have an albedo value?
    pub snow: UnitInterval<u8>,
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
        self.land.into()
    }

    pub fn ocean(&self) -> f64 {
        self.land.not().f64()
    }

    pub fn plains(&self) -> f64 {
        (self.land * self.plains).f64()
    }

    pub fn mountains(&self) -> f64 {
        (self.land * !self.plains).f64()
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
    use num_traits::{Bounded, NumCast, NumOps, SaturatingAdd, SaturatingSub};
    use std::ops::{Add, AddAssign, Mul, Not, Sub, SubAssign};

    pub trait FractionalInteger: NumCast + Bounded + NumOps + Copy {
        const MAX_F64: f64;
    }

    impl FractionalInteger for u8 {
        const MAX_F64: f64 = Self::MAX as f64;
    }

    impl FractionalInteger for u16 {
        const MAX_F64: f64 = Self::MAX as f64;
    }

    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct UnitInterval<T>(T);

    impl<T: FractionalInteger> From<f64> for UnitInterval<T> {
        #[inline]
        fn from(value: f64) -> Self {
            Self::new(value)
        }
    }

    impl<T: FractionalInteger> From<UnitInterval<T>> for f64 {
        #[inline]
        fn from(value: UnitInterval<T>) -> Self {
            value.f64()
        }
    }

    impl<T: FractionalInteger> UnitInterval<T> {
        const RECIP: f64 = 1.0 / T::MAX_F64;
        const RECIP_32: f32 = Self::RECIP as f32;

        #[inline]
        pub fn new(value: f64) -> Self {
            let value = num_traits::clamp(value, 0.0, 1.0) * T::MAX_F64; // [0..T::MAX_F64]
            let value = NumCast::from(value).unwrap(); // UNWRAP: value clamped to known range
            Self(value)
        }

        #[inline]
        pub fn byte(self) -> T {
            self.0
        }

        #[inline]
        pub fn f64(self) -> f64 {
            self.raw_f64() * Self::RECIP
        }

        #[inline]
        pub fn f32(self) -> f32 {
            self.raw_f32() * Self::RECIP_32
        }

        #[inline]
        pub fn raw_f64(self) -> f64 {
            self.0.to_f64().unwrap()
        }

        #[inline]
        pub fn raw_f32(self) -> f32 {
            self.0.to_f32().unwrap()
        }

        #[inline]
        pub fn inverse(self) -> Self {
            Self(T::max_value() - self.0)
        }
    }

    impl Mul for UnitInterval<u8> {
        type Output = UnitInterval<u16>;

        fn mul(self, rhs: Self) -> Self::Output {
            UnitInterval(self.0 as u16 * rhs.0 as u16)
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> Add<T> for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn add(self, rhs: T) -> Self::Output {
            Self(self.0.saturating_add(&rhs))
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> AddAssign<T> for UnitInterval<T> {
        #[inline]
        fn add_assign(&mut self, rhs: T) {
            *self = self.add(rhs);
        }
    }

    impl<T: FractionalInteger + SaturatingSub> Sub<T> for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: T) -> Self::Output {
            Self(self.0.saturating_sub(&rhs))
        }
    }

    impl<T: FractionalInteger + SaturatingSub> SubAssign<T> for UnitInterval<T> {
        #[inline]
        fn sub_assign(&mut self, rhs: T) {
            *self = self.sub(rhs);
        }
    }

    impl<T: FractionalInteger> Not for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn not(self) -> Self::Output {
            Self(T::max_value() - self.0)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn fraction_u8_new() {
            assert_eq!(UnitInterval::<u8>::default(), UnitInterval::new(-1.0));
            assert_eq!(UnitInterval::<u8>::default(), UnitInterval::new(0.0));
            assert_eq!(127, UnitInterval::<u8>::new(0.5).0);
            assert_eq!(UnitInterval::<u8>::new(1.0), UnitInterval::new(2.0));
        }

        #[test]
        fn fraction_u16_new() {
            assert_eq!(UnitInterval::<u16>::default(), UnitInterval::new(-1.0));
            assert_eq!(UnitInterval::<u16>::default(), UnitInterval::new(0.0));
            assert_eq!(32767, UnitInterval::<u16>::new(0.5).0);
            assert_eq!(UnitInterval::<u16>::new(1.0), UnitInterval::new(2.0));
        }

        #[test]
        fn fraction_u8_add() {
            assert_eq!(1, (UnitInterval::<u8>::default() + 1).0);
            assert_eq!(u8::MAX, (UnitInterval::<u8>::new(1.0) + 1).0);
        }

        #[test]
        fn fraction_u8_sub() {
            assert_eq!(UnitInterval::<u8>::default(), (UnitInterval::default() - 1));
            assert_eq!(254, (UnitInterval::<u8>::new(1.0) - 1).0);
        }

        #[test]
        fn fraction_u16_add() {
            assert_eq!(1, (UnitInterval::<u16>::default() + 1).0);
            assert_eq!(u16::MAX, (UnitInterval::<u16>::new(1.0) + 1).0);
        }

        #[test]
        fn fraction_u16_sub() {
            assert_eq!(
                UnitInterval::<u16>::default(),
                (UnitInterval::default() - 1)
            );
            assert_eq!(65534, (UnitInterval::<u16>::new(1.0) - 1).0);
        }

        #[test]
        fn into_f64() {
            assert_eq!(0.0, From::from(UnitInterval::<u8>::new(0.0)));
            assert_eq!(1.0, From::from(UnitInterval::<u8>::new(1.0)));
        }
    }
}
