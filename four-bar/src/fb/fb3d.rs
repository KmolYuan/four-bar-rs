use super::*;
use crate::efd::na;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

/// Spherical normalized four-bar linkage.
///
/// + Buffer order: `[l0, l1, l2, l3, l4, g]`
///
/// # Parameters
///
/// + Ground link `l0`
/// + Driver link `l1`
/// + Coupler link `l2`
/// + Follower link `l3`
/// + Extanded link `l4`
/// + Coupler link angle `g`
pub type SNormFourBar = NormFourBarBase<[f64; 6]>;
/// Spherical four-bar linkage.
///
/// + Buffer 1 order: `[ox, oy, oz, r, p0i, p0j, a]`
/// + Buffer 2 order: `[l0, l1, l2, l3, l4, g]`
///
/// # Parameters
///
/// + Sphere X offset `ox`
/// + Sphere Y offset `oy`
/// + Sphere Z offset `oz`
/// + Sphere radius `r`
/// + Sphere polar offset `p0i`
/// + Sphere azimuth offset `p0j`
/// + Angle offset `a`
/// + Ground link `l0`
/// + Driver link `l1`
/// + Coupler link `l2`
/// + Follower link `l3`
/// + Extanded link `l4`
/// + Coupler link angle `g`
pub type SFourBar = FourBarBase<[f64; 7], [f64; 6]>;

impl Normalized<efd::D3> for SNormFourBar {
    type De = SFourBar;

    fn denormalize(&self) -> Self::De {
        SFourBar {
            buf: [0., 0., 0., 1., 0., 0., 0.],
            norm: self.clone(),
        }
    }

    fn normalize(mut de: Self::De) -> Self {
        let l1 = de.l1();
        de.norm.buf[..4].iter_mut().for_each(|x| *x /= l1);
        de.norm
    }
}

impl SNormFourBar {
    /// Create with linkage lengths in degrees.
    pub fn new_degrees(buf: [f64; 6], inv: bool) -> Self {
        Self { buf: buf.map(f64::to_radians), inv }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &NormFourBar, w: f64) -> Self {
        let NormFourBar { buf: [l0, l2, l3, l4, g], inv } = *fb;
        let [l0, l1, l2, l3, l4] = [l0, fb.l1(), l2, l3, l4].map(|x| x / w * TAU);
        Self { buf: [l0, l1, l2, l3, l4, g], inv }
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
        fn l0, l0_mut(self) -> f64 { self.buf[0] }
        /// Length of the driver link.
        fn l1, l1_mut(self) -> f64 { self.buf[1] }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.buf[2] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.buf[3] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.buf[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.buf[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.inv }
    }

    /// Reduce angles and spread out to planar coordinate.
    pub fn planar_loop(&self) -> [f64; 4] {
        let ls = [self.l0(), self.l1(), self.l2(), self.l3()];
        let mut ls = ls
            .map(|d| d.rem_euclid(TAU))
            .map(|d| if d > PI { TAU - d } else { d });
        let mut longer = Vec::new();
        let mut shorter = Vec::new();
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
    /// Create with linkage lengths in degrees.
    pub fn new_degrees(mut buf: [f64; 7], buf_norm: [f64; 6], inv: bool) -> Self {
        buf[4..].iter_mut().for_each(|x| *x = x.to_radians());
        let norm = SNormFourBar { buf: buf_norm.map(f64::to_radians), inv };
        Self { buf, norm }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &FourBar, center: [f64; 3], r: f64, w: f64) -> Self {
        let [p0x, p0y, a, l1] = fb.buf;
        let NormFourBar { buf: [l0, l2, l3, l4, g], inv } = fb.norm;
        let [p0i, p0j, l0, l1, l2, l3, l4] = [p0x, p0y, l0, l1, l2, l3, l4].map(|x| x / w * TAU);
        let norm = SNormFourBar { buf: [l0, l1, l2, l3, l4, g], inv };
        let [ox, oy, oz] = center;
        Self { buf: [ox, oy, oz, r, p0i, p0j, a], norm }
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::new_norm(
            [0., 0., 0., 1., 0., 0., 0.],
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
        /// X offset of the sphere center.
        fn ox, ox_mut(self) -> f64 { self.buf[0] }
        /// Y offset of the sphere center.
        fn oy, oy_mut(self) -> f64 { self.buf[1] }
        /// Z offset of the sphere center.
        fn oz, oz_mut(self) -> f64 { self.buf[2] }
        /// Radius of the sphere.
        fn r, r_mut(self) -> f64 { self.buf[3] }
        /// Sphere polar angle offset of the driver link pivot.
        fn p0i, p0i_mut(self) -> f64 { self.buf[4] }
        /// Sphere azimuth angle offset of the driver link pivot.
        fn p0j, p0j_mut(self) -> f64 { self.buf[5] }
        /// Angle offset of the ground link.
        fn a, a_mut(self) -> f64 { self.buf[6] }
        /// Length of the ground link.
        fn l0, l0_mut(self) -> f64 { self.norm.buf[0] }
        /// Length of the driver link.
        fn l1, l1_mut(self) -> f64 { self.norm.buf[1] }
        /// Length of the coupler link.
        fn l2, l2_mut(self) -> f64 { self.norm.buf[2] }
        /// Length of the follower link.
        fn l3, l3_mut(self) -> f64 { self.norm.buf[3] }
        /// Length of the extended link.
        fn l4, l4_mut(self) -> f64 { self.norm.buf[4] }
        /// Angle of the extended link on the coupler.
        fn g, g_mut(self) -> f64 { self.norm.buf[5] }
        /// Inverse coupler and follower to another circuit.
        fn inv, inv_mut(self) -> bool { self.norm.inv }
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.norm.planar_loop())
    }
}

impl Transformable<efd::D3> for SFourBar {
    #[allow(unused_variables)]
    fn transform_inplace(&mut self, trans: &efd::Transform<<efd::D3 as EfdDim>::Trans>) {
        // TODO
        todo!()
    }
}

impl CurveGen<efd::D3> for SFourBar {
    fn is_valid(&self) -> bool {
        let mut v = self.norm.planar_loop();
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        v[3] < v[..3].iter().sum()
    }

    fn is_open_curve(&self) -> bool {
        self.ty().is_open_curve()
    }

    fn pos(&self, t: f64) -> Option<[efd::Coord<efd::D3>; 5]> {
        curve_interval(self, t)
    }

    fn angle_bound(&self) -> Option<[f64; 2]> {
        self.is_valid()
            .then(|| angle_bound(self.norm.planar_loop()))
    }
}

fn curve_interval(fb: &SFourBar, b: f64) -> Option<[[f64; 3]; 5]> {
    let [ox, oy, oz, r, p0i, p0j, a] = fb.buf;
    let SNormFourBar { buf: [l0, l1, l2, l3, l4, g], inv } = fb.norm;
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
    Some(build_coords!(p0, p1, p2, p3, p4)).filter(|js| {
        js.iter()
            .all(|[x, y, z]| x.is_finite() && y.is_finite() && z.is_finite())
    })
}

/* Chiang, C. H. (1984). ON THE CLASSIFICATION OF SPHERICAL FOUR-BAR LINKAGES
 * (Vol. 19, Issue 3). */
#[test]
fn spherical_loop_reduce() {
    use approx::assert_abs_diff_eq;
    use FourBarTy::*;

    macro_rules! assert_fb_eq {
        ([$l0:literal, $l1:literal, $l2:literal, $l3:literal],
         [$pl0:literal, $pl1:literal, $pl2:literal, $pl3:literal],
         $ty:expr) => {
            let fb = SNormFourBar::new_degrees([$l0, $l1, $l2, $l3, 0., 0.], false);
            let [l0, l1, l2, l3] = fb.planar_loop();
            assert_abs_diff_eq!(l0.to_degrees(), $pl0, epsilon = 1e-12);
            assert_abs_diff_eq!(l1.to_degrees(), $pl1, epsilon = 1e-12);
            assert_abs_diff_eq!(l2.to_degrees(), $pl2, epsilon = 1e-12);
            assert_abs_diff_eq!(l3.to_degrees(), $pl3, epsilon = 1e-12);
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
