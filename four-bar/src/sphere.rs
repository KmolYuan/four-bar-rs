use std::f64::consts::FRAC_PI_2;

use crate::four_bar::*;

/// Spherical normalized four-bar linkage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
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

    impl_parm_method! {
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
}

/// Spherical four-bar linkage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
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
    /// Order: `[ox, oy, r, p0i, p0j, a, l0, l1, l2, l3, l4, g]`
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

    /// Create from normalized linkage.
    pub const fn from_norm(norm: SNormFourBar) -> Self {
        Self { v: [0., 0., 0., 1., FRAC_PI_2, 0., 0.], norm }
    }

    impl_parm_method! {
        /// X offset of the sphere center.
        fn ox, ox_mut(self) -> f64 { self.v[0] }
        /// Y offset of the sphere center.
        fn oy, oy_mut(self) -> f64 { self.v[1] }
        /// Radius of the sphere.
        fn r, r_mut(self) -> f64 { self.v[2] }
        /// Sphere polar angle offset of the driver link pivot.
        fn p0i, p0i_mut(self) -> f64 { self.v[3] }
        /// Sphere azimuth angle offset of the driver link pivot.
        fn p0j, p0j_mut(self) -> f64 { self.v[4] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.v[5] }
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

    /// Return the position of the input angle.
    pub fn pos(&self, theta: f64) -> [[f64; 3]; 5] {
        curve_interval(&self.v, &self.norm, theta)
    }

    /// Generator for coupler curve in specified angle.
    pub fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<[f64; 3]> {
        curve_in(start, end, res, |theta| self.pos(theta), |[.., p4]| p4)
    }
}

fn curve_interval(v: &[f64; 7], norm: &SNormFourBar, b: f64) -> [[f64; 3]; 5] {
    let [ox, oy, oz, r, p0i, p0j, a] = *v;
    let SNormFourBar { v: [l0, l1, l2, l3, l4, g], inv } = *norm;
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
    let o = na::Point3::new(ox, oy, oz);
    let op0 = na::Vector3::new(
        r * p0i.sin() * p0j.cos(),
        r * p0i.sin() * p0j.sin(),
        r * p0i.cos(),
    );
    let p0 = o + op0;
    let z = na::Vector3::z_axis();
    let p1 = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op0), a);
        let rot2 = na::Rotation3::from_axis_angle(&z, l0);
        rot1 * rot2 * p0
    };
    let p2 = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op0), a + b);
        let rot2 = na::Rotation3::from_axis_angle(&z, l1);
        rot1 * rot2 * p0
    };
    let op2 = p2 - o;
    let op1 = p1 - o;
    let op3 = {
        let rot1 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op1), a + d);
        let rot2 = na::Rotation3::from_axis_angle(&z, l3);
        rot1 * rot2 * op1
    };
    let p3 = o + op3;
    let i = op2;
    let k = op2.cross(&op3) / l2.sin();
    let j = k.cross(&i);
    let p4 = {
        let rot1 = na::Matrix3::from_columns(&[i, j, k]);
        let rot2 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op2), g);
        let rot3 = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(op2.cross(&op3)), l4);
        rot1 * rot2 * rot3 * p1
    };
    [
        [p0.x, p0.y, p0.z],
        [p1.x, p1.y, p1.z],
        [p2.x, p2.y, p2.z],
        [p3.x, p3.y, p3.z],
        [p4.x, p4.y, p4.z],
    ]
}
