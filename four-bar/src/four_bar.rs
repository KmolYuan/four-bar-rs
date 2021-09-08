#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::FRAC_PI_6,
    ops::{Div, DivAssign},
};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FourBar {
    pub p0: (f64, f64),
    pub a: f64,
    pub l0: f64,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
    pub g: f64,
}

impl Default for FourBar {
    fn default() -> Self {
        // A example mechanism
        Self {
            p0: (0., 0.),
            a: 0.,
            l0: 90.,
            l1: 35.,
            l2: 70.,
            l3: 70.,
            l4: 45.,
            g: FRAC_PI_6,
        }
    }
}

impl FourBar {
    pub fn reset(&mut self) {
        self.p0 = (0., 0.);
        self.a = 0.;
    }
}

impl Div<f64> for FourBar {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            l0: self.l0 / rhs,
            l1: self.l1 / rhs,
            l2: self.l2 / rhs,
            l3: self.l3 / rhs,
            l4: self.l4 / rhs,
            ..self
        }
    }
}

impl DivAssign<f64> for FourBar {
    fn div_assign(&mut self, rhs: f64) {
        *self = self.clone().div(rhs);
    }
}
