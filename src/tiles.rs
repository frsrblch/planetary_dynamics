use crate::tiles::unit_interval::UnitInterval;
use std::ops::Sub;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Terrain {
    /// The fraction covered by ocean, counted from the 'left'
    pub ocean: UnitInterval<u8>,
    /// The fraction covered by ocean, counted from the 'right'
    pub mountains: UnitInterval<u8>,
    /// The fraction covered by plains, counted oceans on the 'left' and mountains on the 'right'
    pub plains: UnitInterval<u8>,
    /// The fraction covered by glacier, counted from the 'right'
    /// Mountains will be covered before plains, which are covered before oceans.
    pub glacier: UnitInterval<u8>,
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
    pub fn new_fraction(ocean: f64, mountain: f64, glacier: f64) -> Self {
        let ocean = UnitInterval::<u8>::from(ocean);
        let land = ocean.inverse();

        let mountains = UnitInterval::<u8>::from(mountain);
        let mountains = (land * mountains).u8();
        let plains = land - mountains;

        let glacier = UnitInterval::from(glacier);

        debug_assert_eq!(u8::MAX, ocean.byte() + plains.byte() + mountains.byte());

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
            ocean: UnitInterval::<u8>::new(ocean),
            mountains: UnitInterval::<u8>::new(mountains),
            plains: UnitInterval::new(plains),
            glacier: UnitInterval::<u8>::new(glacier),
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

pub mod unit_interval {
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
            let value = num_traits::clamp(value, 0.0, 1.0) * T::MAX_F64; // [0..T::MAX_F64]
            let value = NumCast::from(value.round()).unwrap(); // UNWRAP: value clamped to known range
            Self(value)
        }
    }

    impl<T: FractionalInteger> From<T> for UnitInterval<T> {
        fn from(value: T) -> Self {
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
        pub fn new(value: T) -> Self {
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

    impl UnitInterval<u16> {
        pub fn u8(self) -> UnitInterval<u8> {
            let value = (self.0 / u8::MAX as u16) as u8;
            UnitInterval::new(value)
        }
    }

    impl Mul for UnitInterval<u8> {
        type Output = UnitInterval<u16>;

        fn mul(self, rhs: Self) -> Self::Output {
            UnitInterval(self.0 as u16 * rhs.0 as u16)
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> Add for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0.saturating_add(&rhs.0))
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> Add<T> for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn add(self, rhs: T) -> Self::Output {
            Self(self.0.saturating_add(&rhs))
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> AddAssign for UnitInterval<T> {
        #[inline]
        fn add_assign(&mut self, rhs: Self) {
            *self = self.add(rhs);
        }
    }

    impl<T: FractionalInteger + SaturatingAdd> AddAssign<T> for UnitInterval<T> {
        #[inline]
        fn add_assign(&mut self, rhs: T) {
            *self = self.add(rhs);
        }
    }

    impl<T: FractionalInteger + SaturatingSub> Sub for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output {
            Self(self.0.saturating_sub(&rhs.0))
        }
    }

    impl<T: FractionalInteger + SaturatingSub> Sub<T> for UnitInterval<T> {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: T) -> Self::Output {
            Self(self.0.saturating_sub(&rhs))
        }
    }

    impl<T: FractionalInteger + SaturatingSub> SubAssign for UnitInterval<T> {
        #[inline]
        fn sub_assign(&mut self, rhs: Self) {
            *self = self.sub(rhs);
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
            assert_eq!(UnitInterval::<u8>::default(), UnitInterval::from(-1.0));
            assert_eq!(UnitInterval::<u8>::default(), UnitInterval::from(0.0));
            assert_eq!(128, UnitInterval::<u8>::from(0.5).0);
            assert_eq!(UnitInterval::<u8>::from(1.0), UnitInterval::from(2.0));
            assert_eq!(255, UnitInterval::<u8>::from(0.999).0);
        }

        #[test]
        fn fraction_u16_new() {
            assert_eq!(UnitInterval::<u16>::default(), UnitInterval::from(-1.0));
            assert_eq!(UnitInterval::<u16>::default(), UnitInterval::from(0.0));
            assert_eq!(32768, UnitInterval::<u16>::from(0.5).0);
            assert_eq!(UnitInterval::<u16>::from(1.0), UnitInterval::from(2.0));
        }

        #[test]
        fn fraction_u8_add() {
            assert_eq!(1, (UnitInterval::<u8>::default() + 1).0);
            assert_eq!(u8::MAX, (UnitInterval::<u8>::from(1.0) + 1).0);
        }

        #[test]
        fn fraction_u8_sub() {
            assert_eq!(UnitInterval::<u8>::default(), (UnitInterval::default() - 1));
            assert_eq!(254, (UnitInterval::<u8>::from(1.0) - 1).0);
        }

        #[test]
        fn fraction_u16_add() {
            assert_eq!(1, (UnitInterval::<u16>::default() + 1).0);
            assert_eq!(u16::MAX, (UnitInterval::<u16>::from(1.0) + 1).0);
        }

        #[test]
        fn fraction_u16_sub() {
            assert_eq!(
                UnitInterval::<u16>::default(),
                (UnitInterval::default() - 1)
            );
            assert_eq!(65534, (UnitInterval::<u16>::from(1.0) - 1).0);
        }

        #[test]
        fn into_f64() {
            assert_eq!(0.0, UnitInterval::<u8>::from(0.0).f64());
            assert_eq!(1.0, UnitInterval::<u8>::from(1.0).f64());
        }

        #[test]
        fn fraction_u8_inverse() {
            assert_eq!(UnitInterval(200u8), UnitInterval(55u8).inverse());
        }
    }
}
