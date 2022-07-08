use crate::{efd::GeoInfo, Formula, Linkage, Mechanism};
use std::{
    f64::consts::{FRAC_PI_6, TAU},
    ops::{Div, DivAssign},
};

macro_rules! impl_parm_method {
    ($(#[doc = $doc:literal] fn $name:ident, $name_mut:ident => $ind:literal)+) => {$(
        #[doc = $doc]
        #[doc = "\n\nGet the value."]
        #[inline]
        pub const fn $name(&self) -> f64 { self.v[$ind] }
        #[doc = $doc]
        #[doc = "\n\nModify the value."]
        #[inline]
        pub fn $name_mut(&mut self) -> &mut f64 { &mut self.v[$ind] }
    )+};
}

fn sort_link(mut fb: [f64; 4]) -> [f64; 4] {
    fb.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    fb
}

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

/// Normalized four-bar linkage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, PartialEq, Debug)]
pub struct NormFourBar {
    v: [f64; 5],
    inv: bool,
}

impl Default for NormFourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl NormFourBar {
    /// Zeros data. (Default value)
    pub const ZERO: Self = Self::from_vec([0.; 5], false);

    /// Create a normalized four-bar linkage from a vector.
    pub const fn from_vec(v: [f64; 5], inv: bool) -> Self {
        Self { v, inv }
    }

    impl_parm_method! {
        /// Length of the ground link.
        fn l0, l0_mut => 0
        /// Length of the coupler link.
        fn l2, l2_mut => 1
        /// Length of the follower link.
        fn l3, l3_mut => 2
        /// Length of the extended link.
        fn l4, l4_mut => 3
        /// Angle of the extended link on the coupler.
        fn g, g_mut => 4
    }

    /// Inverse coupler and follower to another circuit.
    ///
    /// Get the value.
    pub const fn inv(&self) -> bool {
        self.inv
    }

    /// Inverse coupler and follower to another circuit.
    ///
    /// Modify the value.
    pub fn inv_mut(&mut self) -> &mut bool {
        &mut self.inv
    }

    /// Transform from any linkages to Grashof crank-rocker / double crank,
    /// the linkage types with continuous motion.
    ///
    /// Panic when the length of `xs` is not greater than 5.
    pub fn cr_dc_transform(xs: &[f64]) -> [f64; 5] {
        if let [l0, l2, l3, l4, g, ..] = xs[..5] {
            let [s, p, q, l] = sort_link([l0, 1., l2, l3]);
            if s + l < p + q && (s == 1. || s == l0) {
                [l0, l2, l3, l4, g]
            } else {
                let l1 = s;
                [q / l1, l / l1, p / l1, l4 / l1, g]
            }
        } else {
            panic!("invalid lengths")
        }
    }

    /// Transform from any linkages to Grashof double-rocker,
    /// the linkage type with non-continuous motion.
    ///
    /// Panic when the length of `xs` is not greater than 5.
    pub fn dr_transform(xs: &[f64]) -> [f64; 5] {
        if let [l0, l2, l3, l4, g, ..] = xs[..5] {
            let [s, p, q, l] = sort_link([l0, 1., l2, l3]);
            if s + l < p + q && l == 1. {
                [l0, l2, l3, l4, g]
            } else {
                let l1 = l;
                [q / l1, s / l1, p / l1, l4 / l1, g]
            }
        } else {
            panic!("invalid lengths")
        }
    }
}

/// Four-bar linkage with offset.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, PartialEq, Debug)]
pub struct FourBar {
    v: [f64; 4], // p0x + p0y + a + l1
    norm: NormFourBar,
}

impl Default for FourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl FourBar {
    /// Zeros data. (Default value)
    pub const ZERO: Self = Self { v: [0.; 4], norm: NormFourBar::ZERO };

    /// Create with linkage lengths.
    pub const fn new(v: [f64; 6], inv: bool) -> Self {
        let [l0, l1, l2, l3, l4, g] = v;
        let norm = NormFourBar::from_vec([l0, l2, l3, l4, g], inv);
        let v = [0., 0., 0., l1];
        Self { v, norm }
    }

    /// Create with linkage lengths and offset.
    pub const fn with_offset(v: [f64; 9], inv: bool) -> Self {
        let [p0x, p0y, a, l0, l1, l2, l3, l4, g] = v;
        let norm = NormFourBar::from_vec([l0, l2, l3, l4, g], inv);
        let v = [p0x, p0y, a, l1];
        Self { v, norm }
    }

    /// Create a normalized four-bar linkage from a vector.
    pub const fn from_vec(v: [f64; 5], inv: bool) -> Self {
        let norm = NormFourBar::from_vec(v, inv);
        Self { v: [0.; 4], norm }
    }

    /// Transform a normalized four-bar linkage from a vector.
    pub fn from_transform(v: [f64; 5], inv: bool, geo: GeoInfo<f64>) -> Self {
        let [p0x, p0y] = geo.center;
        let [l0, l2, l3, l4, g] = v;
        let v = [
            l0 * geo.scale,
            l2 * geo.scale,
            l3 * geo.scale,
            l4 * geo.scale,
            g,
        ];
        let norm = NormFourBar::from_vec(v, inv);
        let v = [p0x, p0y, geo.rot, geo.scale];
        Self { v, norm }
    }

    impl_parm_method! {
        /// X offset of the driver link pivot.
        fn p0x, p0x_mut => 0
        /// Y offset of the driver link pivot.
        fn p0y, p0y_mut  => 1
        /// Angle offset of the ground link.
        fn a, a_mut  => 2
        /// Length of the driver link.
        fn l1, l1_mut => 3
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::new([90., 35., 70., 70., 45., FRAC_PI_6], false)
    }

    /// Return the the type according to this linkage lengths.
    pub fn class(&self) -> Class {
        macro_rules! arms {
            ($d:expr => $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == self.l0() => $c1,
                    d if d == self.l1() => $c2,
                    d if d == self.l2() => $c3,
                    d if d == self.l3() => $c4,
                    _ => unreachable!(),
                }
            };
        }
        let mut d = [self.l0(), self.l1(), self.l2(), self.l3()];
        d.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if d[0] + d[3] < d[1] + d[2] {
            arms! { d[0] => Class::GCCC, Class::GCRR, Class::GRCR, Class::GRRC }
        } else {
            arms! { d[3] => Class::RRR1, Class::RRR2, Class::RRR3, Class::RRR4 }
        }
    }

    /// Return true if the linkage has no offset and offset angle.
    pub fn is_aligned(&self) -> bool {
        self.p0x() == 0. && self.p0y() == 0. && self.a() == 0.
    }

    /// Remove the origin offset and the offset angle.
    pub fn align(&mut self) {
        self.v[..3].fill(0.);
    }

    /// Transform into normalized four-bar linkage.
    pub fn normalize(&mut self) {
        self.align();
        *self /= self.l1();
    }

    /// Input angle bounds of Grashof linkage.
    ///
    /// Return `None` if unsupported.
    pub fn grashof_bound(&self) -> Option<[f64; 2]> {
        let d_func = |l23: f64| {
            (self.l0() * self.l0() + self.l1() * self.l1() - l23 * l23)
                / (2. * self.l0() * self.l1())
        };
        if self.l0() + self.l1() <= self.l2() + self.l3()
            && (self.l0() - self.l1()).abs() >= (self.l2() - self.l3()).abs()
        {
            Some([0., TAU])
        } else if self.l0() + self.l1() >= self.l2() + self.l3()
            && (self.l0() - self.l1()).abs() >= (self.l2() - self.l3()).abs()
        {
            let d = d_func(self.l2() + self.l3());
            Some([-d.acos(), d.acos()])
        } else if self.l0() + self.l1() >= self.l2() + self.l3()
            && self.l0() + self.l1() <= self.l2() + self.l3()
        {
            let d1 = d_func(self.l2() - self.l3());
            let d2 = d_func(self.l2() + self.l3());
            Some([d1.acos(), d2.acos()])
        } else if self.l0() + self.l1() <= self.l2() + self.l3()
            && (self.l0() - self.l1()).abs() <= (self.l2() - self.l3()).abs()
        {
            let d = d_func(self.l2() - self.l3());
            Some([d.acos(), TAU - d.acos()])
        } else {
            None
        }
    }
}

impl From<NormFourBar> for FourBar {
    fn from(norm: NormFourBar) -> Self {
        Self { v: [0.; 4], norm }
    }
}

impl std::ops::Deref for FourBar {
    type Target = NormFourBar;

    fn deref(&self) -> &Self::Target {
        &self.norm
    }
}

impl std::ops::DerefMut for FourBar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.norm
    }
}

impl Div<f64> for FourBar {
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<f64> for FourBar {
    fn div_assign(&mut self, rhs: f64) {
        *self.l0_mut() /= rhs;
        *self.l1_mut() /= rhs;
        *self.l2_mut() /= rhs;
        *self.l3_mut() /= rhs;
        *self.l4_mut() /= rhs;
    }
}

impl Linkage for NormFourBar {
    type Joint = [[f64; 2]; 5];

    fn allocate(&self) -> (Self::Joint, Vec<Formula>) {
        let Self { v: [l0, l2, l3, l4, g], inv } = self;
        let joints = [[0., 0.], [*l0, 0.], [0., 0.], [0., 0.], [0., 0.]];
        let mut fs = Vec::with_capacity(3);
        fs.push(Formula::Pla(0, 1., 0., 2));
        if (l0 - l2).abs() < 1e-20 && (1. - l3).abs() < 1e-20 {
            // Special case
            fs.push(Formula::Ppp(0, 2, 1, 3));
        } else {
            fs.push(Formula::Pllp(2, *l2, *l3, 1, *inv, 3));
        }
        fs.push(Formula::Plap(2, *l4, *g, 3, 4));
        (joints, fs)
    }

    fn apply<L, const N: usize>(
        m: &Mechanism<L>,
        angle: f64,
        joint: [usize; N],
        ans: &mut [[f64; 2]; N],
    ) where
        L: Linkage<Joint = Self::Joint>,
    {
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

impl Linkage for FourBar {
    type Joint = [[f64; 2]; 5];

    fn allocate(&self) -> (Self::Joint, Vec<Formula>) {
        let Self {
            v: [p0x, p0y, a, l1],
            norm: NormFourBar { v: [l0, l2, l3, l4, g], inv },
        } = self;
        let joints = [
            [*p0x, *p0y],
            [p0x + l0 * a.cos(), p0y + l0 * a.sin()],
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

    fn apply<L, const N: usize>(
        m: &Mechanism<L>,
        angle: f64,
        joint: [usize; N],
        ans: &mut [[f64; 2]; N],
    ) where
        L: Linkage<Joint = Self::Joint>,
    {
        <NormFourBar as Linkage>::apply(m, angle, joint, ans)
    }
}
