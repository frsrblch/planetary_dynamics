#![allow(non_snake_case)]

pub use crate::adjacency::adj_array::AdjArray;
use crate::adjacency::units::*;
use fxhash::FxHashMap as HashMap;
use std::convert::TryFrom;

mod adj_array {
    use std::fmt::{Display, Formatter};
    use std::iter::FromIterator;

    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
    pub struct AdjArray([u16; Self::LEN]);

    impl FromIterator<usize> for AdjArray {
        fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
            // this isn't optimal, but it's only done at startup
            let mut array = <[u16; Self::LEN]>::default();
            let mut len = 0usize;
            let mut iter = iter.into_iter();

            array[1..].iter_mut().zip(&mut iter).for_each(|(v, item)| {
                assert!(item <= u16::MAX as usize);
                *v = item as u16;
                len += 1;
            });

            assert_eq!(None, iter.next());

            array[0] = len as u16;

            Self(array)
        }
    }

    impl Display for AdjArray {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.is_empty() {
                write!(f, "[]")
            } else {
                let last = self.len() - 1;
                write!(f, "[")?;
                for (i, n) in self.iter().enumerate() {
                    if i < last {
                        write!(f, "{}, ", n)?;
                    } else {
                        write!(f, "{}", n)?;
                    }
                }
                write!(f, "]")
            }
        }
    }

    impl AdjArray {
        const LEN: usize = 9;
        const MAX: usize = Self::LEN - 1;

        pub fn len(&self) -> usize {
            self.0[0] as usize
        }

        pub fn is_empty(&self) -> bool {
            self.0[0] == 0
        }

        pub fn iter(&self) -> Iter {
            self.into_iter()
        }

        pub fn contains(&self, value: usize) -> bool {
            for v in self {
                if v == value {
                    return true;
                }
            }

            false
        }

        pub fn push(&mut self, value: usize) {
            assert!(self.len() < Self::MAX);
            assert!(value <= u16::MAX as usize);

            self.0[self.len() + 1] = value as u16;
            self.0[0] += 1;
        }

        pub fn and(self, rhs: Self) -> Self {
            self.iter().filter(|n| rhs.contains(*n)).collect()
        }
    }

    impl<'a> IntoIterator for &'a AdjArray {
        type Item = usize;
        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Self::IntoIter {
            let end = self.len() + 1;
            Iter(self.0[1..end].iter())
        }
    }

    pub struct Iter<'a>(std::slice::Iter<'a, u16>);

    impl<'a> Iterator for Iter<'a> {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next().map(|t| *t as usize)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn default() {
            let adj_array = AdjArray::default();
            assert_eq!(0, adj_array.iter().count());
        }

        #[test]
        fn from_iter() {
            let iter = (0usize..4).into_iter();

            let microvec = AdjArray::from_iter(iter);

            assert_eq!(4, microvec.len());
            assert_eq!(vec![0usize, 1, 2, 3], microvec.iter().collect::<Vec<_>>());
        }

        #[test]
        fn display_empty() {
            assert_eq!("[]", AdjArray::from_iter(vec![]).to_string());
        }

        #[test]
        fn display_values() {
            assert_eq!("[1, 2, 3]", AdjArray::from_iter(vec![1, 2, 3]).to_string());
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Adjacency {
    map: HashMap<u16, Vec<AdjArray>>,
}

impl Adjacency {
    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn register(&mut self, nodes: usize) {
        let nodes = u16::try_from(nodes).unwrap();

        self.map
            .entry(nodes)
            .or_insert_with(|| Self::create_min_edges(nodes));
    }

    #[track_caller]
    pub fn get(&self, nodes: usize) -> &Vec<AdjArray> {
        let nodes = u16::try_from(nodes).unwrap();
        self.map
            .get(&nodes)
            .unwrap_or_else(|| panic!("unregisted size: {}", nodes))
    }

    fn create_min_edges(nodes: u16) -> Vec<AdjArray> {
        let rotations = rotations(nodes);

        let points = (0..nodes)
            .into_iter()
            .map(move |index| Node { index, nodes }.position(rotations))
            .collect::<Vec<_>>();

        let mut edges = points
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                points
                    .iter()
                    .enumerate()
                    .skip(i + 1)
                    .map(move |(j, q)| ((*p - *q).magnitude_squared(), (i, j)))
            })
            .collect::<Vec<_>>();

        edges.sort();

        // Taking 3 edges per node wasn't enough to complete the graph
        let count = (nodes as f64 * 3.05) as usize;
        let iter = edges.into_iter().take(count);
        let mut edges = vec![AdjArray::default(); nodes as usize];

        for (_, (i, j)) in iter {
            edges[i].push(j);
            edges[j].push(i);
        }

        edges
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Node {
    index: u16,
    nodes: u16,
}

impl Node {
    pub fn new(index: u16, nodes: u16) -> Self {
        assert!(index < nodes);
        Self { index, nodes }
    }

    pub fn fraction(self) -> ClosedUnitInterval {
        ClosedUnitInterval::fraction(self.index, self.nodes)
    }

    pub fn coordinate(self, rotations: f64) -> SphericalCoordinate {
        let fraction = self.fraction();
        SphericalCoordinate {
            phi: Phi::from(fraction),
            theta: Theta::fraction(fraction, rotations),
        }
    }

    pub fn position(self, rotations: f64) -> Position3 {
        self.coordinate(rotations).position()
    }
}

pub fn rotations(nodes: u16) -> f64 {
    (nodes as f64 - 0.25).sqrt() * 2.0
}

pub mod units {
    use physics_types::{Angle, Area, Length};
    use std::cmp::Ordering;
    use std::ops::{Add, Mul, Sub};

    /// Represents a number on the interval [0..1]
    #[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
    pub struct ClosedUnitInterval(f64);

    impl ClosedUnitInterval {
        pub fn fraction(n: u16, N: u16) -> Self {
            assert!((0..N).contains(&n));
            ClosedUnitInterval((n as f64 + 0.5) / N as f64)
        }

        pub fn inverse(phi: Phi) -> Self {
            Self(0.5 * (1.0 - phi.0.cos()))
        }
    }

    /// The angle φ is in the range [0..π], and represents the angle relative to the poles
    #[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Phi(Angle);

    impl From<ClosedUnitInterval> for Phi {
        fn from(fraction: ClosedUnitInterval) -> Self {
            Self(Angle::acos(1.0 - 2.0 * fraction.0))
        }
    }

    /// The angle θ represents the rotation of the spiral in the interval [0..Rτ]
    /// Where R is the number of rotations, as calculated from the number of nodes by the `rotations` function
    #[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Theta(Angle);

    impl Theta {
        pub(crate) fn fraction(fraction: ClosedUnitInterval, rotations: f64) -> Self {
            Self::rotations(Phi::from(fraction), rotations)
        }

        pub(crate) fn rotations(phi: Phi, rotations: f64) -> Self {
            Self(phi.0 * rotations)
        }
    }

    impl Add<Angle> for Theta {
        type Output = Theta;

        fn add(self, rhs: Angle) -> Self::Output {
            Theta(self.0 + rhs)
        }
    }

    impl Sub<Angle> for Theta {
        type Output = Theta;

        fn sub(self, rhs: Angle) -> Self::Output {
            Theta(self.0 - rhs)
        }
    }

    /// Represents a point on a sphere of arbitrary radius
    #[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
    pub struct SphericalCoordinate {
        pub phi: Phi,
        pub theta: Theta,
    }

    impl SphericalCoordinate {
        pub fn position(self) -> Position3 {
            Position3 {
                x: self.theta.0.cos() * self.phi.0.sin(),
                y: self.theta.0.sin() * self.phi.0.sin(),
                z: self.phi.0.cos(),
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
    pub struct Position3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }

    impl Sub for Position3 {
        type Output = Distance3;

        fn sub(self, rhs: Self) -> Self::Output {
            Distance3 {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
                z: self.z - rhs.z,
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
    pub struct Distance3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }

    impl Distance3 {
        pub fn magnitude(self) -> LengthFactor {
            LengthFactor::new(self.magnitude_inner())
        }

        fn magnitude_inner(self) -> f64 {
            self.magnitude_squared_inner().sqrt()
        }

        pub fn magnitude_squared(self) -> AreaFactor {
            AreaFactor::new(self.magnitude_squared_inner())
        }

        fn magnitude_squared_inner(self) -> f64 {
            self.x * self.x + self.y * self.y + self.z * self.z
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct AreaFactor(f64);

    impl Eq for AreaFactor {}

    impl PartialOrd for AreaFactor {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }

    impl Ord for AreaFactor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl AreaFactor {
        pub fn new(value: f64) -> Self {
            assert!(value.is_finite());
            Self(value)
        }
    }
    impl Mul<Area> for AreaFactor {
        type Output = Area;

        fn mul(self, rhs: Area) -> Self::Output {
            self.0 * rhs
        }
    }

    impl Mul<AreaFactor> for Area {
        type Output = Area;

        fn mul(self, rhs: AreaFactor) -> Self::Output {
            self * rhs.0
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct LengthFactor(f64);

    impl Eq for LengthFactor {}

    impl PartialOrd for LengthFactor {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }

    impl Ord for LengthFactor {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl LengthFactor {
        pub fn new(value: f64) -> Self {
            assert!(value.is_finite());
            Self(value)
        }
    }

    impl Mul<Length> for LengthFactor {
        type Output = Length;

        fn mul(self, rhs: Length) -> Self::Output {
            self.0 * rhs
        }
    }

    impl Mul<LengthFactor> for Length {
        type Output = Length;

        fn mul(self, rhs: LengthFactor) -> Self::Output {
            self * rhs.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn closed_unit_interval() {
        let fraction = ClosedUnitInterval::fraction(1, 4);
        let phi = Phi::from(fraction);
        let inv_phi = ClosedUnitInterval::inverse(phi);

        assert_eq!(fraction, inv_phi);
    }
}
