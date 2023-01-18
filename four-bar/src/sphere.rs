use crate::plane::*;
use std::f64::consts::TAU;

macro_rules! impl_shared_method {
    ($self:ident, $v:expr, $norm:expr) => {
        /// Return the position of the input angle.
        pub fn pos(&$self, theta: f64) -> [[f64; 3]; 5] {
            curve_interval($v, $norm, theta)
        }

        /// Generator for all curves in specified angle.
        pub fn curves_in(&$self, start: f64, end: f64, res: usize) -> Vec<[[f64; 3]; 3]> {
            curve_in(start, end, res, |theta| $self.pos(theta), |[.., p2, p3, p4]| [p2, p3, p4])
        }

        /// Generator for coupler curve in specified angle.
        pub fn curve_in(&$self, start: f64, end: f64, res: usize) -> Vec<[f64; 3]> {
            curve_in(start, end, res, |theta| $self.pos(theta), |[.., p4]| p4)
        }

        /// Generator for curves.
        pub fn curves(&$self, res: usize) -> Vec<[[f64; 3]; 3]> {
            $self.angle_bound().map(|[start, end]| $self.curves_in(start, end, res)).unwrap_or_default()
        }

        /// Generator for coupler curve.
        pub fn curve(&$self, res: usize) -> Vec<[f64; 3]> {
            $self.angle_bound().map(|[start, end]| $self.curve_in(start, end, res)).unwrap_or_default()
        }

        /// Clone a instance with `!self.inv`.
        pub fn clone_inversed(&$self) -> Self {
            let mut fb = $self.clone();
            *fb.inv_mut() = !fb.inv();
            fb
        }

        /// Return true if the linkage length is valid.
        pub fn is_valid(&$self) -> bool {
            let mut v = [$self.l0(), $self.l1(), $self.l2(), $self.l3()];
            v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            v[3] < v[..3].iter().sum()
        }

        /// Input angle bounds of the linkage.
        ///
        /// Return `None` if unsupported.
        pub fn angle_bound(&self) -> Option<[f64; 2]> {
            self.is_valid()
                .then(|| angle_bound([self.l0(), self.l1(), self.l2(), self.l3()]))
        }
    };
}

fn angle_bound([l0, l1, l2, l3]: [f64; 4]) -> [f64; 2] {
    // TODO
    let min = ((l3 - l2).cos() >= (l0 - l1).cos())
        .then(|| ((l3 - l2).cos() - l1.cos() * l0.cos()) / (l1.sin() * l0.sin()))
        .filter(|c| (-1.0..1.).contains(c))
        .map(|c| c.acos())
        .unwrap_or(0.);
    let max = ((l2 + l3).cos() <= (l1 + l0).cos())
        .then(|| ((l2 + l3).cos() - l1.cos() * l0.cos()) / (l1.sin() * l0.sin()))
        .filter(|c| (-1.0..1.).contains(c))
        .map(|c| c.acos())
        .unwrap_or(TAU);
    [max, min]
}

/// Spherical normalized four-bar linkage.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
#[derive(Clone, PartialEq, Debug)]
pub struct SNormFourBar {
    // l0, l1, l2, l3, l4, g
    v: [f64; 6],
    inv: bool,
}

impl Default for SNormFourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl SNormFourBar {
    /// Zeros data. (Default value)
    ///
    /// This is an invalid linkage.
    pub const ZERO: Self = Self::new([0.; 6], false);

    /// Create with linkage lengths in radians.
    ///
    /// Order: `[l0, l1, l2, l3, l4, g]`
    pub const fn new(v: [f64; 6], inv: bool) -> Self {
        Self { v, inv }
    }

    /// Create with linkage lengths in degrees.
    ///
    /// Order: `[l0, l1, l2, l3, l4, g]`
    pub fn new_degrees(v: [f64; 6], inv: bool) -> Self {
        Self { v: v.map(f64::to_radians), inv }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &NormFourBar, w: f64) -> Self {
        let NormFourBar { v: [l0, l2, l3, l4, g], inv } = *fb;
        let [l0, l1, l2, l3, l4] = [l0, fb.l1(), l2, l3, l4].map(|x| x / w * TAU);
        Self { v: [l0, l1, l2, l3, l4, g], inv }
    }

    impl_parm_method! {
        /// X offset of the sphere center.
        fn ox(self) -> f64 { 0. }
        /// Y offset of the sphere center.
        fn oy(self) -> f64 { 0. }
        /// Z offset of the sphere center.
        fn oz(self) -> f64 { 0. }
        /// Radius of the sphere.
        fn r(self) -> f64 { 1. }
        /// Sphere polar angle offset of the driver link pivot.
        fn p0i(self) -> f64 { 0. }
        /// Sphere azimuth angle offset of the driver link pivot.
        fn p0j(self) -> f64 { 0. }
        /// Angle offset of the ground link.
        fn a(self) -> f64 { 0. }
        /// Length of the ground link.
        fn l0, l0_mut(self) -> f64 { self.v[0] }
        /// Length of the driver link.
        fn l1, l1_mut(self) -> f64 { self.v[1] }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.v[2] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.v[3] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.v[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.v[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.inv }
    }

    impl_shared_method!(self, &[0., 0., 0., 1., 0., 0., 0.], self);
}

/// Spherical four-bar linkage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(default))]
#[derive(Clone, PartialEq, Debug)]
pub struct SFourBar {
    // ox, oy, oz, r, p0i, p0j, a
    v: [f64; 7],
    norm: SNormFourBar,
}

impl Default for SFourBar {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<SNormFourBar> for SFourBar {
    fn from(norm: SNormFourBar) -> Self {
        Self::from_norm(norm)
    }
}

impl SFourBar {
    /// Zeros data. (Default value)
    ///
    /// This is an invalid linkage.
    pub const ZERO: Self = Self::new([0.; 13], false);

    /// Create with linkage lengths in radians.
    ///
    /// Order: `[ox, oy, oz, r, p0i, p0j, a, l0, l1, l2, l3, l4, g]`
    pub const fn new(v: [f64; 13], inv: bool) -> Self {
        let [ox, oy, oz, r, p0i, p0j, a, l0, l1, l2, l3, l4, g] = v;
        Self {
            v: [ox, oy, oz, r, p0i, p0j, a],
            norm: SNormFourBar { v: [l0, l1, l2, l3, l4, g], inv },
        }
    }

    /// Create a normalized linkage from a vector in radians.
    pub const fn new_norm(v: [f64; 6], inv: bool) -> Self {
        Self::from_norm(SNormFourBar::new(v, inv))
    }

    /// Create a normalized linkage from a vector in degrees.
    pub fn new_norm_degrees(v: [f64; 6], inv: bool) -> Self {
        Self::from_norm(SNormFourBar::new_degrees(v, inv))
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    ///
    /// Additionally, the sphere `center` and radius `r` must be given.
    pub fn new_wrap(fb: &FourBar, center: [f64; 3], r: f64, w: f64) -> Self {
        let FourBar {
            v: [p0x, p0y, a, l1],
            norm: NormFourBar { v: [l0, l2, l3, l4, g], inv },
        } = *fb;
        let [ox, oy, oz] = center;
        let [p0i, p0j, l0, l1, l2, l3, l4] = [p0x, p0y, l0, l1, l2, l3, l4].map(|x| x / w * TAU);
        Self::new([ox, oy, oz, r, p0i, p0j, a, l0, l1, l2, l3, l4, g], inv)
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    ///
    /// This method supports [`NormFourBar`] type.
    pub fn new_wrap_norm(fb: &NormFourBar, w: f64) -> Self {
        Self::from_norm(SNormFourBar::new_wrap(fb, w))
    }

    /// Create from normalized linkage.
    pub const fn from_norm(norm: SNormFourBar) -> Self {
        Self { v: [0., 0., 0., 1., 0., 0., 0.], norm }
    }

    impl_parm_method! {
        /// X offset of the sphere center.
        fn ox, ox_mut(self) -> f64 { self.v[0] }
        /// Y offset of the sphere center.
        fn oy, oy_mut(self) -> f64 { self.v[1] }
        /// Z offset of the sphere center.
        fn oz, oz_mut(self) -> f64 { self.v[2] }
        /// Radius of the sphere.
        fn r, r_mut(self) -> f64 { self.v[3] }
        /// Sphere polar angle offset of the driver link pivot.
        fn p0i, p0i_mut(self) -> f64 { self.v[4] }
        /// Sphere azimuth angle offset of the driver link pivot.
        fn p0j, p0j_mut(self) -> f64 { self.v[5] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.v[6] }
        /// Length of the ground link.
        fn l0, l0_mut(self) -> f64 { self.norm.v[0] }
        /// Length of the driver link.
        fn l1, l1_mut(self) -> f64 { self.norm.v[1] }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.norm.v[2] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.norm.v[3] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.norm.v[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.norm.v[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.norm.inv }
    }

    impl_shared_method!(self, &self.v, &self.norm);
}

fn curve_interval(v: &[f64; 7], norm: &SNormFourBar, b: f64) -> [[f64; 3]; 5] {
    let [ox, oy, oz, r, p0i, p0j, a] = *v;
    let SNormFourBar { v: [l0, l1, l2, l3, l4, g], inv } = *norm;
    let e1 = {
        let rx1v = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), g);
        let rx1m = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), l4);
        let p1 = na::Vector3::new(r, 0., 0.);
        rx1v * rx1m * p1
    };
    let d = {
        let k1 = l1.cos() * l3.cos() * l0.cos();
        let k2 = l2.cos();
        let k3 = l1.sin() * l3.cos() * l0.sin();
        let k4 = l1.cos() * l3.sin() * l0.sin();
        let k5 = l1.sin() * l3.sin() * l0.cos();
        let k6 = l1.sin() * l3.sin();
        let h1 = k1 - k2 + k3 * b.cos();
        let h2 = -k4 + k5 * b.cos();
        let h3 = k6 * b.sin();
        if !inv {
            2. * (-h3 + (h3 * h3 - h1 * h1 + h2 * h2).sqrt()).atan2(h1 - h2)
        } else {
            2. * (-h3 - (h3 * h3 - h1 * h1 + h2 * h2).sqrt()).atan2(h1 - h2)
        }
    };
    let op0 = na::Vector3::new(1., 0., 0.);
    let op1 = {
        let rot = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), l0);
        rot * op0
    };
    let op2 = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), b);
        let rot2 = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), l1);
        rot1 * rot2 * op0
    };
    let op3 = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op1), d);
        let rot2 = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), l3);
        rot1 * rot2 * op1
    };
    let rot = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Vector3::z_axis(), p0j);
        let rot2 = na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), p0i);
        let rot3 = na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), a);
        rot1 * rot2 * rot3
    };
    let o = na::Point3::new(ox, oy, oz);
    let p0 = o + rot * op0;
    let p1 = o + rot * op1;
    let p2 = o + rot * op2;
    let p3 = o + rot * op3;
    let p4 = {
        let i = op2.normalize();
        let k = (op2.cross(&op3) / l2.sin()).normalize();
        let j = k.cross(&i);
        let op4 = na::Rotation3::from_basis_unchecked(&[i, j, k]) * e1;
        o + rot * op4
    };
    macro_rules! build_coords {
        ($($p:ident),+) => { [$([$p.x, $p.y, $p.z]),+] }
    }
    build_coords!(p0, p1, p2, p3, p4)
}
