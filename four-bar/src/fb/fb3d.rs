//! Spherical four-bar linkages.
use super::*;
use efd::na;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

/// Unnormalized part of spherical four-bar linkage.
///
/// Please see [`SFourBar`] for more information.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UnNorm {
    /// X offset of the sphere center
    pub ox: f64,
    /// Y offset of the sphere center
    pub oy: f64,
    /// Z offset of the sphere center
    pub oz: f64,
    /// Radius of the sphere
    pub r: f64,
    /// Sphere polar angle (z axis) offset of the driver link pivot
    pub p1i: f64,
    /// Sphere azimuth angle (xy plane) offset of the driver link pivot
    pub p1j: f64,
    /// Angle offset of the ground link
    pub a: f64,
}

impl UnNorm {
    /// Create a new instance.
    pub const fn new() -> Self {
        Self::from_radius(1.)
    }

    /// Create a new instance from the sphere radius.
    pub const fn from_radius(r: f64) -> Self {
        Self { ox: 0., oy: 0., oz: 0., r, p1i: 0., p1j: 0., a: 0. }
    }

    /// Set the origin of the sphere center.
    pub fn set_origin(&mut self, ox: f64, oy: f64, oz: f64) {
        [self.ox, self.oy, self.oz] = [ox, oy, oz];
    }

    /// Set the angle of the ground link.
    pub fn set_rotation(&mut self, p1i: f64, p1j: f64, a: f64) {
        [self.p1i, self.p1j, self.a] = [p1i, p1j, a];
    }
}

/// Spherical normalized four-bar linkage.
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SNormFourBar {
    /// Length of the ground link
    pub l1: f64,
    /// Length of the driver link
    pub l2: f64,
    /// Length of the coupler link
    pub l3: f64,
    /// Length of the follower link
    pub l4: f64,
    /// Length of the extended link
    pub l5: f64,
    /// Angle of the extended link on the coupler
    pub g: f64,
    /// Inverse coupler and follower to another circuit
    pub stat: Stat,
}

impl FromVectorized for SNormFourBar {
    type Dim = na::U6;

    fn from_vectorized(v: &[f64], stat: Stat) -> Result<Self, std::array::TryFromSliceError> {
        let [l1, l2, l3, l4, l5, g] = <[f64; 6]>::try_from(v)?;
        Ok(Self { l1, l2, l3, l4, l5, g, stat })
    }
}

impl IntoVectorized for SNormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, Stat) {
        let Self { l1, l2, l3, l4, l5, g, stat } = self;
        (vec![l1, l2, l3, l4, l5, g], stat)
    }
}

/// Spherical four-bar linkage.
///
/// # Parameters
///
/// There have 13 parameters in total.
///
/// + Sphere X offset `ox`
/// + Sphere Y offset `oy`
/// + Sphere Z offset `oz`
/// + Sphere radius `r`
/// + Sphere polar offset `p1i` (theta)
/// + Sphere azimuth offset `p1j` (phi)
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
pub type SFourBar = Mech<UnNorm, SNormFourBar>;

impl Normalized<3> for SNormFourBar {
    type De = SFourBar;

    fn denormalize(self) -> Self::De {
        SFourBar { unnorm: UnNorm::new(), norm: self }
    }

    fn normalize(de: Self::De) -> Self {
        de.norm
    }

    fn normalize_inplace(de: &mut Self::De) {
        de.unnorm = UnNorm::new();
    }
}

impl SNormFourBar {
    /// Create with linkage lengths in degrees.
    pub fn to_radians(self) -> Self {
        Self {
            l1: self.l1.to_radians(),
            l2: self.l2.to_radians(),
            l3: self.l3.to_radians(),
            l4: self.l4.to_radians(),
            l5: self.l5.to_radians(),
            g: self.g.to_radians(),
            stat: self.stat,
        }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &NormFourBar, w: f64) -> Self {
        let [l1, l2, l3, l4] = fb.planar_loop().map(|x| x / w * TAU);
        let l5 = fb.l5 / w * TAU;
        Self { l1, l2, l3, l4, l5, g: fb.g, stat: fb.stat }
    }
}

impl SFourBar {
    /// Create with linkage lengths in degrees.
    pub fn to_radians(self) -> Self {
        let unnorm = UnNorm {
            p1i: self.unnorm.p1i.to_radians(),
            p1j: self.unnorm.p1j.to_radians(),
            a: self.unnorm.a.to_radians(),
            ..self.unnorm
        };
        Self { unnorm, norm: self.norm.to_radians() }
    }

    /// Wrap unit to link angle. The argument `w` maps to the angle of [`TAU`].
    pub fn new_wrap(fb: &FourBar, center: [f64; 3], r: f64, w: f64) -> Self {
        assert!(r > 0.);
        let fb2d::UnNorm { p1x, p1y, a, l2 } = fb.unnorm;
        let NormFourBar { l1, l3, l4, l5, g, stat } = fb.norm;
        let [p1i, p1j, l1, l2, l3, l4, l5] = [p1x, p1y, l1, l2, l3, l4, l5].map(|x| x / w * TAU);
        let [ox, oy, oz] = center;
        Self {
            unnorm: UnNorm { ox, oy, oz, r, p1j: FRAC_PI_2 - p1j, p1i, a },
            norm: SNormFourBar { l1, l2, l3, l4, l5, g, stat },
        }
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        let norm = SNormFourBar {
            l1: FRAC_PI_2,
            l2: 0.6108652381980153,
            l3: 1.2217304763960306,
            l4: 1.2217304763960306,
            l5: FRAC_PI_4,
            g: 0.5235987755982988,
            stat: Stat::C1B1,
        };
        Self::new(UnNorm::from_radius(90.), norm)
    }

    /// Take the sphere part without the linkage length.
    pub fn take_sphere(&self) -> Self {
        Self { unnorm: self.unnorm.clone(), ..Default::default() }
    }

    /// Get the sphere center. (`ox`, `oy`, `oz`)
    pub fn sc(&self) -> [f64; 3] {
        [self.unnorm.ox, self.unnorm.oy, self.unnorm.oz]
    }

    /// Get the sphere center and radius. (`ox`, `oy`, `oz`, `r`)
    pub fn scr(&self) -> [f64; 4] {
        let fb = &self.unnorm;
        [fb.ox, fb.oy, fb.oz, fb.r]
    }

    /// Get the sphere radius. (`r`)
    pub fn sr(&self) -> f64 {
        self.unnorm.r
    }
}

impl Statable for SNormFourBar {
    fn stat_mut(&mut self) -> &mut Stat {
        &mut self.stat
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl PlanarLoop for SNormFourBar {
    fn planar_loop(&self) -> [f64; 4] {
        // Reduce angles and spread out to planar coordinate.
        let mut ls = [self.l1, self.l2, self.l3, self.l4]
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

    fn set_to_planar_loop(&mut self) {
        [self.l1, self.l2, self.l3, self.l4] = self.planar_loop();
    }
}

impl PlanarLoop for SFourBar {
    fn planar_loop(&self) -> [f64; 4] {
        self.norm.planar_loop()
    }

    fn set_to_planar_loop(&mut self) {
        self.norm.set_to_planar_loop();
    }
}

impl Transformable<3> for SFourBar {
    fn transform_inplace(&mut self, geo: &efd::GeoVar3) {
        let [ox, oy, oz] = geo.trans();
        let fb = &mut self.unnorm;
        fb.ox += ox;
        fb.oy += oy;
        fb.oz += oz;
        let p1_axis = na::Vector3::from(to_cc(fb.p1i, fb.p1j, 1.));
        let pb = na::Point3::new(fb.a.cos(), fb.a.sin(), 0.) + p1_axis;
        let p1_axis = geo.rot() * p1_axis;
        [fb.p1i, fb.p1j] = to_sc(p1_axis.x, p1_axis.y, p1_axis.z);
        let rot_inv =
            na::UnitQuaternion::rotation_between(&p1_axis, &na::Vector3::z()).unwrap_or_default();
        let pb = rot_inv * geo.rot() * pb;
        fb.a = pb.y.atan2(pb.x);
        fb.r *= geo.scale();
    }
}

impl CurveGen<3> for SFourBar {
    fn pos_s(&self, t: f64, inv: bool) -> Option<[efd::Coord<3>; 5]> {
        curve_interval(self, t, inv)
    }
}

fn curve_interval(fb: &SFourBar, b: f64, inv: bool) -> Option<[[f64; 3]; 5]> {
    // a=alpha, b=beta, g=gamma, d=delta
    let UnNorm { ox, oy, oz, r, p1i, p1j, a } = fb.unnorm;
    let SNormFourBar { l1, l2, l3, l4, l5, g, .. } = fb.norm;
    let op1 = r * na::Vector3::z();
    let e1 = {
        let rx1v = na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), g);
        let rx1m = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l5);
        rx1v * rx1m * op1
    };
    let d = {
        let h1 =
            l2.cos() * l4.cos() * l1.cos() - l3.cos() + l2.sin() * l4.cos() * l1.sin() * b.cos();
        let h2 = -l2.cos() * l4.sin() * l1.sin() + l2.sin() * l4.sin() * l1.cos() * b.cos();
        let h3 = l2.sin() * l4.sin() * b.sin();
        let h = (h3 * h3 - h1 * h1 + h2 * h2).sqrt() * if inv { -1. } else { 1. };
        2. * (-h3 + h).atan2(h1 - h2)
    };
    let op2 = {
        let rot = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l1);
        rot * op1
    };
    let op3 = {
        let rot1 = na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), b);
        let rot2 = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l2);
        rot1 * rot2 * op1
    };
    let op4 = {
        let rot1 = na::UnitQuaternion::from_scaled_axis(op2.normalize() * d);
        let rot2 = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), l4);
        rot1 * rot2 * op2
    };
    let rot = {
        let p1_axis = na::Vector3::from(to_cc(p1i, p1j, 1.));
        let rot1 = na::UnitQuaternion::from_scaled_axis(p1_axis * a);
        let z_axis = na::Vector3::z();
        let rot2 = na::UnitQuaternion::rotation_between(&z_axis, &p1_axis).unwrap_or_default();
        rot1 * rot2
    };
    let o = na::Point3::new(ox, oy, oz);
    let p1 = o + rot * op1;
    let p2 = o + rot * op2;
    let p3 = o + rot * op3;
    let p4 = o + rot * op4;
    let p5 = {
        let i = op3.normalize();
        let k = op3.cross(&op4).normalize();
        let j = k.cross(&i);
        let op5 = na::UnitQuaternion::from_basis_unchecked(&[i, j, k]) * e1;
        o + rot * op5
    };
    macro_rules! build_coords {
        [$($p:ident),+] => { [$([$p.x, $p.y, $p.z]),+] }
    }
    let js = build_coords![p1, p2, p3, p4, p5];
    js.iter().flatten().all(|x| x.is_finite()).then_some(js)
}

/// To spherical coordinate.
///
/// Return `[p1i, p1j]`, ignore the radius.
pub fn to_sc(x: f64, y: f64, z: f64) -> [f64; 2] {
    [x.hypot(y).atan2(z), y.atan2(x)]
}

/// To Cartesian coordinate.
///
/// Return `[x, y, z]`.
pub fn to_cc(p1i: f64, p1j: f64, sr: f64) -> [f64; 3] {
    let x = sr * p1i.sin() * p1j.cos();
    let y = sr * p1i.sin() * p1j.sin();
    let z = sr * p1i.cos();
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
            let fb = SNormFourBar {
                l1: $l1,
                l2: $l2,
                l3: $l3,
                l4: $l4,
                l5: 0.,
                g: 0.,
                stat: Stat::C1B1,
            }
            .to_radians();
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
