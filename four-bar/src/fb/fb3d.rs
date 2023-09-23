use super::*;
use crate::efd::na;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

/// Spherical normalized four-bar linkage.
///
/// + Buffer order: `[l1, l2, l3, l4, l5, g]`
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
pub type SNormFourBar = NormFourBarBase<[f64; 6]>;
/// Spherical four-bar linkage.
///
/// + Buffer 1 order: `[ox, oy, oz, r, p0i, p0j, a]`
/// + Buffer 2 order: `[l1, l2, l3, l4, l5, g]`
///
/// # Parameters
///
/// + Sphere X offset `ox`
/// + Sphere Y offset `oy`
/// + Sphere Z offset `oz`
/// + Sphere radius `r`
/// + Sphere polar offset `p0i` (theta)
/// + Sphere azimuth offset `p0j` (phi)
/// + Angle offset `a`
/// + Ground link `l1`
/// + Driver link `l2`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
///
/// # Spherical Coordinate System
///
/// ![](https://upload.wikimedia.org/wikipedia/commons/thumb/4/4f/3D_Spherical.svg/512px-3D_Spherical.svg.png)
pub type SFourBar = FourBarBase<[f64; 7], [f64; 6]>;

impl Normalized<efd::D3> for SNormFourBar {
    type De = SFourBar;

    fn denormalize(&self) -> Self::De {
        SFourBar { buf: SFourBar::ORIGIN, norm: self.clone() }
    }

    fn normalize(de: &Self::De) -> Self {
        de.norm.clone()
    }
}

impl SNormFourBar {
    /// Create with linkage lengths in degrees.
    pub fn new_degrees(buf: [f64; 6], inv: bool) -> Self {
        Self { buf: buf.map(f64::to_radians), inv }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &NormFourBar, w: f64) -> Self {
        let NormFourBar { buf: [l1, l3, l4, l5, g], inv } = *fb;
        let [l1, l2, l3, l4, l5] = [l1, fb.l2(), l3, l4, l5].map(|x| x / w * TAU);
        Self { buf: [l1, l2, l3, l4, l5, g], inv }
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
        fn l1, l1_mut(self) -> f64 { self.buf[0] }
        /// Length of the driver link.
        fn l2, l2_mut(self) -> f64 { self.buf[1] }
        /// Length of the coupler link.
        fn l3, l3_mut(self) -> f64 { self.buf[2] }
        /// Length of the follower link.
        fn l4, l4_mut(self) -> f64 { self.buf[3] }
        /// Length of the extended link.
        fn l5, l5_mut(self) -> f64 { self.buf[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.buf[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.inv }
    }

    /// Reduce angles and spread out to planar coordinate.
    pub fn planar_loop(&self) -> [f64; 4] {
        let mut ls = [self.l1(), self.l2(), self.l3(), self.l4()]
            .map(|d| d.rem_euclid(TAU))
            .map(|d| if d > PI { TAU - d } else { d });
        let mut longer = Vec::with_capacity(4);
        let mut shorter = Vec::with_capacity(4);
        for d in ls.iter_mut() {
            if *d > FRAC_PI_2 {
                longer.push(d);
            } else {
                shorter.push(d);
            }
        }
        match longer.len() {
            1 => {
                let longest = longer.into_iter().next().unwrap();
                let d = shorter
                    .into_iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let d_changed = PI - *d;
                if d_changed < *longest {
                    *d = d_changed;
                    *longest = PI - *longest;
                }
            }
            3 if *shorter[0] != FRAC_PI_2 => {
                longer.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                longer.into_iter().skip(1).for_each(|d| *d = PI - *d);
            }
            _ => longer.into_iter().for_each(|d| *d = PI - *d),
        }
        ls
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.planar_loop())
    }
}

impl SFourBar {
    const ORIGIN: [f64; 7] = [0., 0., 0., 1., 0., 0., 0.];

    /// Create with linkage lengths in degrees.
    pub fn new_degrees(mut buf: [f64; 7], buf_norm: [f64; 6], inv: bool) -> Self {
        buf[4..].iter_mut().for_each(|x| *x = x.to_radians());
        let norm = SNormFourBar { buf: buf_norm.map(f64::to_radians), inv };
        Self { buf, norm }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &FourBar, center: [f64; 3], r: f64, w: f64) -> Self {
        assert!(r > 0.);
        let [p0x, p0y, a, l2] = fb.buf;
        let NormFourBar { buf: [l1, l3, l4, l5, g], inv } = fb.norm;
        let [p0i, p0j, l1, l2, l3, l4, l5] = [p0x, p0y, l1, l2, l3, l4, l5].map(|x| x / w * TAU);
        let norm = SNormFourBar { buf: [l1, l2, l3, l4, l5, g], inv };
        let [ox, oy, oz] = center;
        Self {
            buf: [ox, oy, oz, r, FRAC_PI_2 - p0j, p0i, a],
            norm,
        }
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::new(
            Self::ORIGIN,
            [
                FRAC_PI_2,
                0.6108652381980153,
                1.2217304763960306,
                1.2217304763960306,
                FRAC_PI_4,
                0.5235987755982988,
            ],
            false,
        )
    }

    impl_parm_method! {
        /// Sphere center.
        fn oc(self) -> [f64; 3] { [self.buf[0], self.buf[1], self.buf[2]] }
        /// X offset of the sphere center.
        fn ox, ox_mut(self) -> f64 { self.buf[0] }
        /// Y offset of the sphere center.
        fn oy, oy_mut(self) -> f64 { self.buf[1] }
        /// Z offset of the sphere center.
        fn oz, oz_mut(self) -> f64 { self.buf[2] }
        /// Radius of the sphere.
        fn r, r_mut(self) -> f64 { self.buf[3] }
        /// Sphere polar angle offset of the driver link pivot. (theta)
        fn p0i, p0i_mut(self) -> f64 { self.buf[4] }
        /// Sphere azimuth angle offset of the driver link pivot. (phi)
        fn p0j, p0j_mut(self) -> f64 { self.buf[5] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.buf[6] }
        /// Length of the ground link.
        fn l1, l1_mut(self) -> f64 { self.norm.buf[0] }
        /// Length of the driver link.
        fn l2, l2_mut(self) -> f64 { self.norm.buf[1] }
        /// Length of the coupler link.
        fn l3, l3_mut(self) -> f64 { self.norm.buf[2] }
        /// Length of the follower link.
        fn l4, l4_mut(self) -> f64 { self.norm.buf[3] }
        /// Length of the extended link.
        fn l5, l5_mut(self) -> f64 { self.norm.buf[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.norm.buf[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.norm.inv }
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.norm.planar_loop())
    }

    /// Take the sphere part without the linkage length.
    pub fn take_sphere(&self) -> Self {
        Self { buf: self.buf, ..Default::default() }
    }
}

impl From<&SNormFourBar> for FourBarTy {
    fn from(fb: &SNormFourBar) -> Self {
        Self::from_loop(fb.planar_loop())
    }
}

impl From<&SFourBar> for FourBarTy {
    fn from(fb: &SFourBar) -> Self {
        Self::from_loop(fb.norm.planar_loop())
    }
}

impl Transformable<efd::D3> for SFourBar {
    fn transform_inplace(&mut self, trans: &efd::Transform3) {
        let [ox, oy, oz] = trans.trans();
        *self.ox_mut() += ox;
        *self.oy_mut() += oy;
        *self.oz_mut() += oz;
        let p0_axis = na::Vector3::from(to_cc([self.p0i(), self.p0j()], 1.));
        let pb = na::Point3::new(self.a().cos(), self.a().sin(), 0.) + p0_axis;
        let p0_axis = trans.rot() * p0_axis;
        [*self.p0i_mut(), *self.p0j_mut()] = to_sc([p0_axis.x, p0_axis.y, p0_axis.z]);
        let rot_inv = if let Some(axis) = p0_axis.cross(&na::Vector3::z()).try_normalize(0.) {
            let angle = p0_axis.dot(&na::Vector3::z()).acos();
            na::UnitQuaternion::from_scaled_axis(axis * angle)
        } else {
            na::UnitQuaternion::identity()
        };
        let pb = rot_inv * trans.rot() * pb;
        *self.a_mut() = pb.y.atan2(pb.x);
        *self.r_mut() *= trans.scale();
    }
}

impl CurveGen<efd::D3> for SFourBar {
    fn pos(&self, t: f64) -> Option<[efd::Coord<efd::D3>; 5]> {
        curve_interval(self, t)
    }

    fn angle_bound(&self) -> AngleBound {
        AngleBound::from_planar_loop(self.norm.planar_loop())
    }
}

fn curve_interval(fb: &SFourBar, b: f64) -> Option<[[f64; 3]; 5]> {
    // a=alpha, b=beta, g=gamma, d=delta
    let [ox, oy, oz, r, p0i, p0j, a] = fb.buf;
    let SNormFourBar { buf: [l1, l2, l3, l4, l5, g], inv } = fb.norm;
    let op0 = r * na::Vector3::z();
    let e1 = {
        let rx1v = na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), g);
        let rx1m = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l5);
        rx1v * rx1m * op0
    };
    let d = {
        let h1 =
            l2.cos() * l4.cos() * l1.cos() - l3.cos() + l2.sin() * l4.cos() * l1.sin() * b.cos();
        let h2 = -l2.cos() * l4.sin() * l1.sin() + l2.sin() * l4.sin() * l1.cos() * b.cos();
        let h3 = l2.sin() * l4.sin() * b.sin();
        let h = (h3 * h3 - h1 * h1 + h2 * h2).sqrt() * if inv { -1. } else { 1. };
        2. * (-h3 + h).atan2(h1 - h2)
    };
    let op1 = {
        let rot = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l1);
        rot * op0
    };
    let op2 = {
        let rot1 = na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), b);
        let rot2 = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l2);
        rot1 * rot2 * op0
    };
    let op3 = {
        let rot1 = na::UnitQuaternion::from_scaled_axis(op1.normalize() * d);
        let rot2 = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l4);
        rot1 * rot2 * op1
    };
    let rot = {
        let p0_axis = na::Vector3::from(to_cc([p0i, p0j], 1.));
        let rot1 = na::UnitQuaternion::from_scaled_axis(p0_axis * a);
        let z_axis = na::Vector3::z();
        let rot2 = if let Some(axis) = z_axis.cross(&p0_axis).try_normalize(0.) {
            let angle = z_axis.dot(&p0_axis).acos();
            na::UnitQuaternion::from_scaled_axis(axis * angle)
        } else {
            na::UnitQuaternion::identity()
        };
        rot1 * rot2
    };
    let o = na::Point3::new(ox, oy, oz);
    let p0 = o + rot * op0;
    let p1 = o + rot * op1;
    let p2 = o + rot * op2;
    let p3 = o + rot * op3;
    let p4 = {
        let i = op2.normalize();
        let k = op2.cross(&op3).normalize();
        let j = k.cross(&i);
        let op4 = na::UnitQuaternion::from_basis_unchecked(&[i, j, k]) * e1;
        o + rot * op4
    };
    macro_rules! build_coords {
        [$($p:ident),+] => { [$([$p.x, $p.y, $p.z]),+] }
    }
    let js = build_coords![p0, p1, p2, p3, p4];
    js.iter().flatten().all(|x| x.is_finite()).then_some(js)
}

// To spherical coordinate
fn to_sc([x, y, z]: [f64; 3]) -> [f64; 2] {
    [x.hypot(y).atan2(z), y.atan2(x)]
}

// To Cartesian coordinate
fn to_cc([theta, phi]: [f64; 2], sr: f64) -> [f64; 3] {
    let x = sr * theta.sin() * phi.cos();
    let y = sr * theta.sin() * phi.sin();
    let z = sr * theta.cos();
    [x, y, z]
}

/*
Chiang, C. H. (1984). ON THE CLASSIFICATION OF SPHERICAL FOUR-BAR LINKAGES
(Vol. 19, Issue 3).
*/
#[test]
fn spherical_loop_reduce() {
    use approx::assert_abs_diff_eq;
    use FourBarTy::*;

    macro_rules! assert_fb_eq {
        ([$l1:literal, $l2:literal, $l3:literal, $l4:literal],
         [$pl1:literal, $pl2:literal, $pl3:literal, $pl4:literal],
         $ty:expr) => {
            let fb = SNormFourBar::new_degrees([$l1, $l2, $l3, $l4, 0., 0.], false);
            let [l1, l2, l3, l4] = fb.planar_loop();
            assert_abs_diff_eq!(l1.to_degrees(), $pl1, epsilon = 1e-12);
            assert_abs_diff_eq!(l2.to_degrees(), $pl2, epsilon = 1e-12);
            assert_abs_diff_eq!(l3.to_degrees(), $pl3, epsilon = 1e-12);
            assert_abs_diff_eq!(l4.to_degrees(), $pl4, epsilon = 1e-12);
            assert_eq!(fb.ty(), $ty);
        };
    }

    assert_fb_eq!([80., 20., 60., 75.], [80., 20., 60., 75.], GCRR);
    assert_fb_eq!([30., 60., 60., 75.], [30., 60., 60., 75.], GCCC);
    assert_fb_eq!([80., 75., 25., 70.], [80., 75., 25., 70.], GRCR);
    assert_fb_eq!([85., 75., 65., 70.], [85., 75., 65., 70.], RRR1);
    assert_fb_eq!([100., 160., 120., 105.], [80., 20., 60., 75.], GCRR);
    assert_fb_eq!([120., 25., 110., 100.], [60., 25., 70., 100.], GCRR);
    assert_fb_eq!([155., 60., 70., 80.], [25., 60., 70., 100.], GCCC);
    assert_fb_eq!([155., 50., 65., 80.], [25., 50., 65., 100.], RRR4);
    assert_fb_eq!([60., 80., 25., 110.], [60., 100., 25., 70.], GRCR);
    assert_fb_eq!([100., 40., 90., 60.], [80., 40., 90., 60.], GCRR);
}
