use physics_types::{Pressure, Temperature};
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ColonyCost(f64);

impl ColonyCost {
    pub fn new(temp: Range<Temperature>, pressure: Pressure, shielding: Shielding) -> Self {
        let t_min = Self::temperature_min(temp);
        let p_min = Self::pressure_min(pressure);
        let s_min = shielding.min_cost();
        let min = t_min.max(p_min).max(s_min);
        Self(min)
    }

    fn pressure_min(pressure: Pressure) -> f64 {
        let atm = pressure / Pressure::in_atm(1.0);

        if atm < 1.0 {
            (1.0 - atm) * 4.0
        } else {
            (atm - 1.0) * 0.5 + 1.0
        }
    }

    fn temperature_min(temperature: Range<Temperature>) -> f64 {
        const LOWER_BOUND: Temperature = Temperature::in_c(5.0);
        const UPPER_BOUND: Temperature = Temperature::in_c(30.0);
        const SLOPE: Temperature = Temperature::in_k(25.0);

        let Range {
            start: lower,
            end: upper,
        } = temperature;

        let lower = (LOWER_BOUND - lower) / SLOPE;
        let upper = (upper - UPPER_BOUND) / SLOPE;

        lower.max(upper).max(0.0) + 1.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Shielding {
    Shielded,
    Partial,
    Unshielded,
}

impl Shielding {
    pub fn min_cost(self) -> f64 {
        match self {
            Shielding::Shielded => 1.0,
            Shielding::Partial => 2.0,
            Shielding::Unshielded => 4.0,
        }
    }
}
