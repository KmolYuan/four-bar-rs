use std::{
    f64::consts::FRAC_PI_6,
    ops::{Div, DivAssign},
};

/// Data type of the four-bar mechanism.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct FourBar {
    /// Origin.
    pub p0: (f64, f64),
    /// Offset angle.
    pub a: f64,
    /// Length of the ground link.
    pub l0: f64,
    /// Length of the crank link.
    pub l1: f64,
    /// Length of the coupler link.
    pub l2: f64,
    /// Length of te follower link.
    pub l3: f64,
    /// Length of the extended link.
    pub l4: f64,
    /// Angle of the extended coupler.
    pub g: f64,
    /// Invert the direction of follower and coupler.
    pub inv: bool,
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
            inv: false,
        }
    }
}

impl FourBar {
    /// Return true if the linkage has no offset and offset angle.
    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        (self.p0, self.a) == ((0., 0.), 0.)
    }

    /// Remove the origin offset and the offset angle.
    #[inline(always)]
    pub fn align(&mut self) {
        self.p0 = (0., 0.);
        self.a = 0.;
    }

    /// Transform into normalized four-bar linkage.
    #[inline(always)]
    pub fn normalize(&mut self) {
        self.align();
        *self /= self.l1;
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
