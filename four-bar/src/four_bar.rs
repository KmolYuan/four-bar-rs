use std::{
    f64::consts::FRAC_PI_6,
    ops::{Div, DivAssign},
};

/// The classification of the four-bar linkage.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Class {
    GCCC,
    GCRR,
    GRCR,
    GRRC,
    RRR1,
    RRR2,
    RRR3,
    RRR4,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::GCCC => "GCCC",
            Self::GCRR => "GCRR",
            Self::GRCR => "GRCR",
            Self::GRRC => "GRRC",
            Self::RRR1 => "RRR1",
            Self::RRR2 => "RRR2",
            Self::RRR3 => "RRR3",
            Self::RRR4 => "RRR4",
        };
        f.write_str(s)
    }
}

impl Class {
    /// Return true if the type is Grashof linkage.
    pub fn is_grashof(&self) -> bool {
        match self {
            Self::GCCC => true,
            Self::GCRR => true,
            Self::GRCR => true,
            Self::GRRC => true,
            Self::RRR1 => false,
            Self::RRR2 => false,
            Self::RRR3 => false,
            Self::RRR4 => false,
        }
    }
}

/// Data type of the four-bar mechanism.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Clone, PartialEq, Debug)]
pub struct FourBar {
    /// Origin.
    pub p0: [f64; 2],
    /// Offset angle.
    pub a: f64,
    /// Length of the ground link.
    pub l0: f64,
    /// Length of the driver link.
    pub l1: f64,
    /// Length of the coupler link.
    pub l2: f64,
    /// Length of te follower link.
    pub l3: f64,
    /// Length of the extended link on the coupler.
    pub l4: f64,
    /// Angle of the extended link on the coupler.
    pub g: f64,
    /// Invert the direction of the follower and the  coupler.
    pub inv: bool,
}

impl FourBar {
    /// An example crank rocker.
    pub const fn example() -> Self {
        Self {
            p0: [0., 0.],
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

    /// Return the the type according to this linkage lengths.
    pub fn class(&self) -> Class {
        let mut d = [self.l0, self.l1, self.l2, self.l3];
        d.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if d[0] + d[3] < d[1] + d[2] {
            match d[0] {
                _ if d[0] == self.l0 => Class::GCCC,
                _ if d[0] == self.l1 => Class::GCRR,
                _ if d[0] == self.l2 => Class::GRCR,
                _ if d[0] == self.l3 => Class::GRRC,
                _ => unreachable!(),
            }
        } else {
            match d[3] {
                _ if d[3] == self.l0 => Class::RRR1,
                _ if d[3] == self.l1 => Class::RRR2,
                _ if d[3] == self.l2 => Class::RRR3,
                _ if d[3] == self.l3 => Class::RRR4,
                _ => unreachable!(),
            }
        }
    }

    /// Return true if the linkage has no offset and offset angle.
    pub fn is_aligned(&self) -> bool {
        self.p0[0] == 0. && self.p0[1] == 0. && self.a == 0.
    }

    /// Remove the origin offset and the offset angle.
    pub fn align(&mut self) {
        self.p0 = [0., 0.];
        self.a = 0.;
    }

    /// Transform into normalized four-bar linkage.
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
