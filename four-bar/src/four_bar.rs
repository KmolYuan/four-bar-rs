use crate::{Formula, Linkage, Mechanism, Point as _};
use std::{
    f64::consts::FRAC_PI_6,
    ops::{Div, DivAssign},
};

/// The classification of the four-bar linkage.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Class {
    /// Grashof double crank
    GCCC,
    /// Grashof crank rocker
    GCRR,
    /// Grashof double rocker
    GRCR,
    /// Grashof rocker crank
    GRRC,
    /// Non-Grashof Double rocker (type 1)
    RRR1,
    /// Non-Grashof Double rocker (type 2)
    RRR2,
    /// Non-Grashof Double rocker (type 3)
    RRR3,
    /// Non-Grashof Double rocker (type 4)
    RRR4,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::GCCC => "Grashof double crank (GCCC)",
            Self::GCRR => "Grashof crank rocker (GCRR)",
            Self::GRCR => "Grashof double rocker (GRCR)",
            Self::GRRC => "Grashof rocker crank (GRRC)",
            Self::RRR1 => "Non-Grashof Double rocker (RRR1)",
            Self::RRR2 => "Non-Grashof Double rocker (RRR2)",
            Self::RRR3 => "Non-Grashof Double rocker (RRR3)",
            Self::RRR4 => "Non-Grashof Double rocker (RRR4)",
        };
        f.write_str(s)
    }
}

impl Class {
    /// Return true if the type is Grashof linkage.
    pub fn is_grashof(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR | Self::GRCR | Self::GRRC)
    }
}

/// Data type of the four-bar mechanism.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, PartialEq, Debug)]
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

impl Default for FourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl FourBar {
    /// Zeros data. (Default value)
    pub const ZERO: Self = Self {
        p0: [0.; 2],
        a: 0.0,
        l0: 0.0,
        l1: 0.0,
        l2: 0.0,
        l3: 0.0,
        l4: 0.0,
        g: 0.0,
        inv: false,
    };

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
        macro_rules! arms {
            ($d:expr => $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == self.l0 => $c1,
                    d if d == self.l1 => $c2,
                    d if d == self.l2 => $c3,
                    d if d == self.l3 => $c4,
                    _ => unreachable!(),
                }
            };
        }
        let mut d = [self.l0, self.l1, self.l2, self.l3];
        d.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if d[0] + d[3] < d[1] + d[2] {
            arms! { d[0] => Class::GCCC, Class::GCRR, Class::GRCR, Class::GRRC }
        } else {
            arms! { d[3] => Class::RRR1, Class::RRR2, Class::RRR3, Class::RRR4 }
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

impl Linkage for FourBar {
    type Joint = [[f64; 2]; 5];

    fn allocate(&self) -> (Self::Joint, Vec<Formula>) {
        let Self {
            p0,
            a,
            l0,
            l1,
            l2,
            l3,
            l4,
            g,
            inv,
        } = self;
        let joints = [
            [p0.x(), p0.y()],
            [p0.x() + l0 * a.cos(), p0.y() + l0 * a.sin()],
            [0., 0.],
            [0., 0.],
            [0., 0.],
        ];
        let mut fs = Vec::with_capacity(3);
        fs.push(Formula::Pla(0, *l1, 0., 2));
        if (l0 - l2).abs() < 1e-20 && (l1 - l3).abs() < 1e-20 {
            // Special case
            fs.push(Formula::Ppp(0, 2, 1, 3));
        } else {
            fs.push(Formula::Pllp(2, *l2, *l3, 1, *inv, 3));
        }
        fs.push(Formula::Plap(2, *l4, *g, 3, 4));
        (joints, fs)
    }

    fn apply<const N: usize>(
        m: &Mechanism<Self>,
        angle: f64,
        joint: [usize; N],
        ans: &mut [[f64; 2]; N],
    ) {
        let mut joints = m.joints;
        let mut formulas = m.fs.clone();
        match formulas.first_mut() {
            Some(Formula::Pla(_, _, ref mut a, _)) => *a = angle,
            _ => panic!("invalid four bar"),
        }
        for f in formulas {
            f.apply(&mut joints);
        }
        if joints[4][0].is_nan() || joints[4][1].is_nan() {
            ans.clone_from(&[[f64::NAN; 2]; N]);
        } else {
            for (ans, joint) in ans.iter_mut().zip(joint) {
                ans.clone_from(&joints[joint]);
            }
        }
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
        *self = Self {
            l0: self.l0 / rhs,
            l1: self.l1 / rhs,
            l2: self.l2 / rhs,
            l3: self.l3 / rhs,
            l4: self.l4 / rhs,
            ..*self
        };
    }
}
