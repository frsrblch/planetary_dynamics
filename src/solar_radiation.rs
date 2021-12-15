use iter_context::ContextualIterator;
use physics_types::{Duration, MolecularMass};

// TODO incorporate chemicals that increase albedo

/// https://en.wikipedia.org/wiki/Atmospheric_escape
/// https://en.wikipedia.org/wiki/Greenhouse_gas
/// https://en.wikipedia.org/wiki/Scale_height
/// https://en.wikipedia.org/wiki/Global_warming_potential
/// Modern and pre-industrial concentrations:  https://cdiac.ess-dive.lbl.gov/pns/current_ghg.html
/// Radiative Forcing of Climate Change: https://www.ipcc.ch/site/assets/uploads/2018/03/TAR-06.pdf
///
/// Greenhouse gas data points:
///     Pre-industrial Earth
///     Modern-day Earth
///     Venus
///     Mars

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Element {
    Hydrogen,
    Helium,
    Carbon,
    Oxygen,
    Nitrogen,
}

impl Element {
    pub const fn mass(self) -> MolecularMass {
        let grams_per_mole = match self {
            Element::Hydrogen => 1.008,
            Element::Helium => 4.0026,
            Element::Carbon => 12.011,
            Element::Oxygen => 15.999,
            Element::Nitrogen => 14.007,
        };
        MolecularMass::in_g_per_mol(grams_per_mole)
    }
}

pub const H: Element = Element::Hydrogen;
pub const HE: Element = Element::Hydrogen;
pub const C: Element = Element::Carbon;
pub const O: Element = Element::Oxygen;
pub const N: Element = Element::Nitrogen;

use gen_id_enum_derive::multi_enum_array;

multi_enum_array! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Gas {
        Hydrogen,
        Helium,
        Nitrogen,
        Oxygen,
        Water,
        Methane,
        CarbonDioxide,
    }
}

impl Gas {
    pub const fn molecular_mass(&self) -> MolecularMass {
        match self {
            Gas::Hydrogen => H.mass() * 2.0,
            Gas::Helium => HE.mass(),
            Gas::Nitrogen => N.mass() * 2.0,
            Gas::Oxygen => O.mass() * 2.0,
            Gas::Water => H.mass() * 2.0 + O.mass(),
            Gas::Methane => C.mass() + H.mass() * 4.0,
            Gas::CarbonDioxide => C.mass() + O.mass() * 2.0,
        }
    }

    /// https://en.wikipedia.org/wiki/Global_warming_potential#Values
    pub fn co2_equivalence(&self) -> f64 {
        match self {
            Gas::CarbonDioxide => 1.0,
            Gas::Methane => 84.0,
            Gas::Water => todo!(),
            _ => 0.0,
        }
    }

    /// https://en.wikipedia.org/wiki/Global_warming_potential#Values
    /// https://en.wikipedia.org/wiki/Atmospheric_methane#Natural_sinks_of_atmospheric_methane
    /// https://en.wikipedia.org/wiki/Hydroxyl_radical
    /// Methane decomposed by bacteria (1/4) and hydroxyl radicals produced from water vapour
    /// and excited atomic oxygen, which is created by plant terpenes from water and light
    /// Both cases require life, which assumes the presence of oxygen
    pub fn half_life(&self) -> Option<Duration> {
        match self {
            Gas::Methane => Some(Duration::in_yr(12.4)),
            _ => None,
        }
    }

    pub fn annual_decay_multiplier(&self) -> Option<f64> {
        self.half_life()
            .map(|t| 0.5_f64.powf(Duration::in_yr(1.0) / t))
    }
}

impl GasArray<f64> {
    pub fn molecular_mass(&self) -> MolecularMass {
        let mut value_sum = 0f64;
        let mut mass_sum = MolecularMass::default();

        for (value, gas) in self.iter().zip(Gas::iter()) {
            mass_sum += gas.molecular_mass() * value;
            value_sum += value;
        }

        mass_sum / value_sum
    }

    pub fn annual_decay(&mut self) {
        self.iter_mut().zip(Gas::iter()).for_each(|(value, gas)| {
            if let Some(m) = gas.annual_decay_multiplier() {
                *value *= m;
            }
        });
    }
}

/// Earth's emissivity: https://phzoe.com/2019/11/05/what-is-earths-surface-emissivity/
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct Emissivity(f64);

impl Emissivity {
    #[inline]
    pub fn new(value: f64) -> Self {
        assert!(value >= 0.0 && value <= 1.0);
        Self(value)
    }
}

/// radiative absorption = 1 - albedo
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct RadiativeAbsorption(f64);

impl RadiativeAbsorption {
    #[inline]
    pub fn new(value: f64) -> Self {
        assert!(value > 0.0 && value <= 1.0);
        Self(value)
    }
}

/// infrared transparency = 1 - fraction reflected back to surface
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct InfraredTransparency(f64);

impl InfraredTransparency {
    pub fn new(value: f64) -> Self {
        assert!(value > 0.0 && value <= 1.0);
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn emissivity_lt_zero() {
        Emissivity::new(-0.01);
    }

    #[test]
    #[should_panic]
    fn emissivity_gt_one() {
        Emissivity::new(1.01);
    }

    #[test]
    #[should_panic]
    fn emissivity_nan() {
        Emissivity::new(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn relative_absorption_zero() {
        RadiativeAbsorption::new(0.0);
    }

    #[test]
    #[should_panic]
    fn relative_absorption_gt_one() {
        RadiativeAbsorption::new(1.01);
    }

    #[test]
    #[should_panic]
    fn relative_absorption_nan() {
        RadiativeAbsorption::new(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn infrared_transparency_zero() {
        InfraredTransparency::new(0.0);
    }

    #[test]
    #[should_panic]
    fn infrared_transparency_gt_one() {
        InfraredTransparency::new(1.01);
    }

    #[test]
    #[should_panic]
    fn infrared_transparency_nan() {
        InfraredTransparency::new(f64::NAN);
    }

    #[test]
    fn gas_array_mass() {
        let mut array = GasArray::<f64>::default();
        array[Gas::Hydrogen] = 0.5;
        array[Gas::Oxygen] = 0.5;

        assert_eq!(
            (Gas::Hydrogen.molecular_mass() + Gas::Oxygen.molecular_mass()) / 2.0,
            array.molecular_mass()
        );
    }
}
