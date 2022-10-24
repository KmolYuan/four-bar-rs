use std::{
    array::TryFromSliceError,
    f64::consts::{FRAC_PI_6, TAU},
    ops::{Div, DivAssign, Mul, MulAssign},
};

macro_rules! impl_parm_method {
    ($(#[doc = $doc:literal] fn $name:ident $(,$name_mut:ident)? ($self:ident) -> $ty:ty {$expr:expr})+) => {$(
        #[doc = concat![$doc, "\n\nGet the value."]]
        #[inline]
        pub const fn $name(&$self) -> $ty { $expr }
        $(#[doc = concat![$doc, "\n\nModify the value."]]
        #[inline]
        pub fn $name_mut(&mut $self) -> &mut $ty { &mut $expr })?
    )+};
}

macro_rules! impl_shared_method {
    ($self:ident, $v:expr, $norm:expr) => {
        /// Return the position of the input angle.
        pub fn pos(&$self, theta: f64) -> [[f64; 2]; 5] {
            curve_interval($v, $norm, theta)
        }

        /// Generator for curves in specified angle.
        pub fn curves_in(&$self, start: f64, end: f64, n: usize) -> Vec<[[f64; 2]; 3]> {
            curve_in(start, end, n, |theta| $self.pos(theta), |[.., j2, j3, j4]| [j2, j3, j4])
        }

        /// Generator for coupler curve in specified angle.
        pub fn curve_in(&$self, start: f64, end: f64, n: usize) -> Vec<[f64; 2]> {
            curve_in(start, end, n, |theta| $self.pos(theta), |[.., j4]| j4)
        }

        /// Generator for curves.
        pub fn curves(&$self, n: usize) -> Vec<[[f64; 2]; 3]> {
            $self.angle_bound().map(|[start, end]| $self.curves_in(start, end, n)).unwrap_or_default()
        }

        /// Generator for coupler curve.
        pub fn curve(&$self, n: usize) -> Vec<[f64; 2]> {
            $self.angle_bound().map(|[start, end]| $self.curve_in(start, end, n)).unwrap_or_default()
        }

        /// Return true if the linkage is parallel.
        pub fn is_parallel(&$self) -> bool {
            ($self.l0() - $self.l2()).abs() < f64::EPSILON && ($self.l1() - $self.l3()).abs() < f64::EPSILON
        }

        /// Return true if the linkage is diamond shape.
        pub fn is_diamond(&$self) -> bool {
            ($self.l0() - $self.l1()).abs() < f64::EPSILON && ($self.l2() - $self.l3()).abs() < f64::EPSILON
        }

        /// Return true if the linkage length is valid.
        pub fn is_valid(&$self) -> bool {
            let mut v = [$self.l0(), $self.l1(), $self.l2(), $self.l3()];
            v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            v[3] < v[..3].iter().sum()
        }

        /// Return true if the linkage has defect.
        pub fn has_defect(&$self) -> bool {
            $self.l0() + $self.l1() >= $self.l2() + $self.l3()
                || ($self.l0() - $self.l1()).abs() <= ($self.l2() - $self.l3()).abs()
        }

        /// Return true if the linkage has closed curve.
        pub fn has_closed_curve(&$self) -> bool {
            $self.l0() + $self.l1() <= $self.l2() + $self.l3()
                && ($self.l0() - $self.l1()).abs() >= ($self.l2() - $self.l3()).abs()
        }

        /// Return the type of this linkage.
        pub fn ty(&$self) -> FourBarTy {
            FourBarTy::from($self)
        }

        /// Input angle bounds of the linkage.
        ///
        /// Return `None` if unsupported.
        pub fn angle_bound(&self) -> Option<[f64; 2]> {
            self.is_valid()
                .then(|| angle_bound([self.l0(), self.l1(), self.l2(), self.l3(), self.a()]))
        }
    };
}

fn sort_link(mut fb: [f64; 4]) -> [f64; 4] {
    fb.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    fb
}

fn angle_bound([l0, l1, l2, l3, a]: [f64; 5]) -> [f64; 2] {
    match (l0 + l1 <= l2 + l3, (l0 - l1).abs() >= (l2 - l3).abs()) {
        (true, true) => [0., TAU],
        (true, false) => {
            let l23 = l2 - l3;
            let d = (l0 * l0 + l1 * l1 - l23 * l23) / (2. * l0 * l1);
            [d.acos() + a, TAU - d.acos() + a]
        }
        (false, true) => {
            let l23 = l2 + l3;
            let d = (l0 * l0 + l1 * l1 - l23 * l23) / (2. * l0 * l1);
            [-d.acos() + a, d.acos() + a]
        }
        (false, false) => {
            let up = l0 * l0 + l1 * l1;
            let down = 2. * l0 * l1;
            let l23 = l2 - l3;
            let d1 = (up - l23 * l23) / down;
            let l23 = l2 + l3;
            let d2 = (up - l23 * l23) / down;
            [d1.acos() + a, d2.acos() + a]
        }
    }
}

fn curve_in<F, M, B>(start: f64, end: f64, n: usize, f: F, map: M) -> Vec<B>
where
    F: Fn(f64) -> [[f64; 2]; 5],
    M: Fn([[f64; 2]; 5]) -> B + Copy,
{
    let interval = (end - start) / n as f64;
    let mut iter = (0..n).map(move |n| start + n as f64 * interval).map(f);
    let mut last = Vec::new();
    while iter.len() > 0 {
        let v = iter
            .by_ref()
            .take_while(|c| c.iter().flatten().all(|x| x.is_finite()))
            .map(map)
            .collect::<Vec<_>>();
        last = v;
    }
    last
}

/// Type of the four-bar linkage.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FourBarTy {
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

impl From<[f64; 4]> for FourBarTy {
    fn from([l0, l1, l2, l3]: [f64; 4]) -> Self {
        macro_rules! arms {
            ($d:expr => $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == l0 => $c1,
                    d if d == l1 => $c2,
                    d if d == l2 => $c3,
                    d if d == l3 => $c4,
                    _ => unreachable!(),
                }
            };
        }
        let [s, p, q, l] = sort_link([l0, l1, l2, l3]);
        if s + l < p + q {
            arms! { s => Self::GCCC, Self::GCRR, Self::GRCR, Self::GRRC }
        } else {
            arms! { l => Self::RRR1, Self::RRR2, Self::RRR3, Self::RRR4 }
        }
    }
}

impl From<NormFourBar> for FourBarTy {
    fn from(fb: NormFourBar) -> Self {
        Self::from(&fb)
    }
}

impl From<FourBar> for FourBarTy {
    fn from(fb: FourBar) -> Self {
        Self::from(&fb)
    }
}

impl From<&NormFourBar> for FourBarTy {
    fn from(fb: &NormFourBar) -> Self {
        Self::from([fb.l0(), 1., fb.l2(), fb.l3()])
    }
}

impl From<&FourBar> for FourBarTy {
    fn from(fb: &FourBar) -> Self {
        Self::from([fb.l0(), fb.l1(), fb.l2(), fb.l3()])
    }
}

impl FourBarTy {
    /// Name of the type.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::GCCC => "Grashof double crank (GCCC)",
            Self::GCRR => "Grashof crank rocker (GCRR)",
            Self::GRCR => "Grashof double rocker (GRCR)",
            Self::GRRC => "Grashof rocker crank (GRRC)",
            Self::RRR1 => "Non-Grashof Double rocker (RRR1)",
            Self::RRR2 => "Non-Grashof Double rocker (RRR2)",
            Self::RRR3 => "Non-Grashof Double rocker (RRR3)",
            Self::RRR4 => "Non-Grashof Double rocker (RRR4)",
        }
    }

    /// Return true if the type is Grashof linkage.
    pub const fn is_grashof(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR | Self::GRCR | Self::GRRC)
    }

    /// Return true if the type has continuous motion.
    pub const fn is_closed_curve(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR)
    }

    /// Return true if the type has non-continuous motion.
    pub const fn is_open_curve(&self) -> bool {
        !self.is_closed_curve()
    }
}

/// Normalized four-bar linkage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Clone, PartialEq, Debug)]
pub struct NormFourBar {
    /// Vector representation
    v: [f64; 5],
    inv: bool,
}

impl Default for NormFourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<[f64; 5]> for NormFourBar {
    fn from(v: [f64; 5]) -> Self {
        Self { v, inv: false }
    }
}

impl TryFrom<&[f64]> for NormFourBar {
    type Error = TryFromSliceError;

    fn try_from(v: &[f64]) -> Result<Self, Self::Error> {
        Self::try_from_slice(v, false)
    }
}

impl NormFourBar {
    /// Zeros data. (Default value)
    ///
    /// This is an invalid linkage.
    pub const ZERO: Self = Self::from_vec([0.; 5], false);

    /// Create a normalized four-bar linkage from a vector.
    pub const fn from_vec(v: [f64; 5], inv: bool) -> Self {
        Self { v, inv }
    }

    /// Create from a slice, equivalent to calling `<[f64; 5]>::try_from()`.
    ///
    /// The `TryFrom` implementation has the same effects.
    pub fn try_from_slice(v: &[f64], inv: bool) -> Result<Self, TryFromSliceError> {
        Ok(Self { v: v.try_into()?, inv })
    }

    /// Construct with `inv` option.
    pub const fn with_inv(&self, inv: bool) -> Self {
        Self { inv, ..*self }
    }

    impl_parm_method! {
        /// X offset of the driver link pivot.
        fn p0x(self) -> f64 { 0. }
        /// Y offset of the driver link pivot.
        fn p0y(self) -> f64 { 0. }
        /// Angle offset of the ground link.
        fn a(self) -> f64 { 0. }
        /// Length of the ground link.
        fn l0, l0_mut(self) -> f64 { self.v[0] }
        /// Length of the driver link.
        fn l1(self) -> f64 { 1. }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.v[1] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.v[2] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.v[3] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.v[4] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.inv }
    }

    /// Get the vector representation of the normalized linkage.
    pub fn vec(&self) -> [f64; 5] {
        self.v
    }

    /// Transform from any linkages to Grashof crank-rocker / double crank,
    /// the linkage types with continuous motion.
    ///
    /// The result might be invalid.
    pub fn to_closed_curve(self) -> Self {
        let [s, p, q, l] = sort_link([self.l0(), self.l1(), self.l2(), self.l3()]);
        if s + l < p + q && (s == self.l1() || s == self.l0()) {
            self
        } else {
            let l1 = s;
            let v = [q / l1, l / l1, p / l1, self.l4() / l1, self.g()];
            Self { v, ..self }
        }
    }

    /// Transform from any linkages to Grashof double-rocker,
    /// the linkage type with non-continuous motion.
    ///
    /// The result might be invalid.
    pub fn to_open_curve(self) -> Self {
        let [s, p, q, l] = sort_link([self.l0(), self.l1(), self.l2(), self.l3()]);
        if s + l < p + q && l == self.l1() {
            self
        } else {
            let l1 = l;
            let v = [q / l1, s / l1, p / l1, self.l4() / l1, self.g()];
            Self { v, ..self }
        }
    }

    impl_shared_method!(self, &[0., 0., 0., self.l1()], self);
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

impl From<NormFourBar> for FourBar {
    fn from(norm: NormFourBar) -> Self {
        Self::from_norm(norm)
    }
}

impl FourBar {
    /// Zeros data. (Default value)
    ///
    /// This is an invalid linkage.
    pub const ZERO: Self = Self::new([0.; 6], false);

    /// Create with linkage lengths.
    ///
    /// Order: `[l0, l1, l2, l3, l4, g]`
    pub const fn new(v: [f64; 6], inv: bool) -> Self {
        let [l0, l1, l2, l3, l4, g] = v;
        let norm = NormFourBar::from_vec([l0, l2, l3, l4, g], inv);
        let v = [0., 0., 0., l1];
        Self { v, norm }
    }

    /// Create with linkage lengths and offset.
    ///
    /// Order: `[p0x, p0y, a, l0, l1, l2, l3, l4, g]`
    pub const fn with_offset(v: [f64; 9], inv: bool) -> Self {
        let [p0x, p0y, a, l0, l1, l2, l3, l4, g] = v;
        let norm = NormFourBar::from_vec([l0, l2, l3, l4, g], inv);
        let v = [p0x, p0y, a, l1];
        Self { v, norm }
    }

    /// Create a normalized four-bar linkage from a vector.
    pub const fn from_vec(v: [f64; 5], inv: bool) -> Self {
        Self::from_norm(NormFourBar::from_vec(v, inv))
    }

    /// Create from normalized linkage.
    pub const fn from_norm(norm: NormFourBar) -> Self {
        Self { v: [0., 0., 0., 1.], norm }
    }

    /// Transform a normalized four-bar linkage from a vector.
    pub fn from_trans(norm: NormFourBar, trans: &efd::Transform2) -> Self {
        let efd::Transform2 { center: [p0x, p0y], rot, scale } = *trans;
        Self { v: [p0x, p0y, rot, scale], norm: norm * scale }
    }

    impl_parm_method! {
        /// X offset of the driver link pivot.
        fn p0x, p0x_mut(self) -> f64 { self.v[0] }
        /// Y offset of the driver link pivot.
        fn p0y, p0y_mut(self) -> f64 { self.v[1] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.v[2] }
        /// Length of the ground link.
        fn l0, l0_mut(self) -> f64 { self.norm.v[0] }
        /// Length of the driver link.
        fn l1, l1_mut(self) -> f64 { self.v[3] }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.norm.v[1] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.norm.v[2] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.norm.v[3] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.norm.v[4] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.norm.inv }
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::new([90., 35., 70., 70., 45., FRAC_PI_6], false)
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

    /// Get the normalized four-bar linkage from this one.
    pub fn to_norm(self) -> NormFourBar {
        let l1 = self.l1();
        self.norm / l1
    }

    impl_shared_method!(self, &self.v, &self.norm);
}

impl Mul<f64> for NormFourBar {
    type Output = Self;
    fn mul(mut self, rhs: f64) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<f64> for NormFourBar {
    fn mul_assign(&mut self, rhs: f64) {
        *self.l0_mut() *= rhs;
        *self.l2_mut() *= rhs;
        *self.l3_mut() *= rhs;
        *self.l4_mut() *= rhs;
    }
}

impl Div<f64> for NormFourBar {
    type Output = Self;
    fn div(mut self, rhs: f64) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<f64> for NormFourBar {
    fn div_assign(&mut self, rhs: f64) {
        *self.l0_mut() /= rhs;
        *self.l2_mut() /= rhs;
        *self.l3_mut() /= rhs;
        *self.l4_mut() /= rhs;
    }
}

impl Mul<f64> for FourBar {
    type Output = Self;
    fn mul(mut self, rhs: f64) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<f64> for FourBar {
    fn mul_assign(&mut self, rhs: f64) {
        self.norm *= rhs;
        *self.l1_mut() *= rhs;
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
        self.norm /= rhs;
        *self.l1_mut() /= rhs;
    }
}

fn angle([x, y]: [f64; 2], d: f64, a: f64) -> [f64; 2] {
    [x + d * a.cos(), y + d * a.sin()]
}

fn angle_with([x1, y1]: [f64; 2], [x2, y2]: [f64; 2], d: f64, a: f64) -> [f64; 2] {
    let a = (y2 - y1).atan2(x2 - x1) + a;
    [x1 + d * a.cos(), y1 + d * a.sin()]
}

fn circle2([x1, y1]: [f64; 2], [x2, y2]: [f64; 2], r1: f64, r2: f64, inv: bool) -> [f64; 2] {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let r = dx.hypot(dy);
    if r > r1 + r2 || r < (r1 - r2).abs() || (r < f64::EPSILON && (r1 - r2).abs() < f64::EPSILON) {
        return [f64::NAN, f64::NAN];
    }
    let a = 0.5 * (r1 * r1 - r2 * r2 + r * r) / r;
    let h = (r1 * r1 - a * a).sqrt();
    let c = dx / r;
    let s = dy / r;
    let xm = x1 + a * c;
    let ym = y1 + a * s;
    if inv {
        [xm + h * s, ym - h * c]
    } else {
        [xm - h * s, ym + h * c]
    }
}

fn parallel([x1, y1]: [f64; 2], [x2, y2]: [f64; 2], [x3, y3]: [f64; 2]) -> [f64; 2] {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let d = dx.hypot(dy);
    let a = dy.atan2(dx);
    [x3 + d * a.cos(), y3 + d * a.sin()]
}

fn curve_interval(v: &[f64; 4], norm: &NormFourBar, theta: f64) -> [[f64; 2]; 5] {
    let [p0x, p0y, a, l1] = v;
    let NormFourBar { v: [l0, l2, l3, l4, g], inv } = norm;
    let j0 = [*p0x, *p0y];
    let j1 = angle(j0, *l0, *a);
    let j2 = angle(j0, *l1, theta);
    let j3 = if (l0 - l2).abs() < f64::EPSILON && (l1 - l3).abs() < f64::EPSILON {
        // Special case
        parallel(j0, j2, j1)
    } else {
        circle2(j2, j1, *l2, *l3, *inv)
    };
    let j4 = angle_with(j2, j3, *l4, *g);
    [j0, j1, j2, j3, j4]
}
