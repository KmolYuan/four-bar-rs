//! Planar four-bar linkages.
#[doc(no_inline)]
pub use super::*;
use efd::na;
use std::f64::consts::FRAC_PI_6;

/// Unnormalized part of four-bar linkage.
///
/// Please see [`FourBar`] for more information.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UnNorm {
    /// X offset of the driver link pivot
    pub p1x: f64,
    /// Y offset of the driver link pivot
    pub p1y: f64,
    /// Angle offset of the ground link
    pub a: f64,
    /// Length of the driver link
    pub l2: f64,
}

impl UnNorm {
    /// Create a new instance.
    pub const fn new() -> Self {
        Self::from_driver(1.)
    }

    /// Create a new instance from the driver link length.
    pub const fn from_driver(l2: f64) -> Self {
        Self { p1x: 0., p1y: 0., a: 0., l2 }
    }

    /// Set the origin of the driver link pivot.
    pub fn set_origin(&mut self, p1x: f64, p1y: f64) {
        [self.p1x, self.p1y] = [p1x, p1y];
    }

    /// Set the angle of the ground link.
    pub fn set_rotation(&mut self, a: f64) {
        self.a = a;
    }

    pub(crate) fn transform_inplace(&mut self, geo: &efd::GeoVar2) {
        let [p1x, p1y] = geo.trans();
        self.p1x += p1x;
        self.p1y += p1y;
        self.a += geo.rot().angle();
        let scale = geo.scale();
        self.l2 *= scale;
    }
}

/// Normalized part of four-bar linkage.
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2=1`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NormFourBar {
    /// Length of the ground link
    pub l1: f64,
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

impl NormFourBar {
    pub(crate) fn scale_inplace(&mut self, scale: f64) {
        self.l1 *= scale;
        self.l3 *= scale;
        self.l4 *= scale;
        self.l5 *= scale;
    }
}

/// Four-bar linkage with offset.
///
/// # Parameters
///
/// There are 9 parameters in total.
///
/// + X offset `p1x`
/// + Y offset `p1y`
/// + Angle offset `a`
/// + Ground link `l1`
/// + Driver link `l2`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
pub type FourBar = Mech<UnNorm, NormFourBar>;

impl Normalized<2> for NormFourBar {
    type De = FourBar;

    fn denormalize(self) -> Self::De {
        FourBar { unnorm: UnNorm::new(), norm: self }
    }

    fn normalize(mut de: Self::De) -> Self {
        de.norm.scale_inplace(de.unnorm.l2.recip());
        de.norm
    }

    fn normalize_inplace(de: &mut Self::De) {
        let l2 = de.unnorm.l2;
        de.unnorm = UnNorm::new();
        de.norm.scale_inplace(l2.recip());
    }
}

impl FourBar {
    /// An example crank rocker.
    pub const fn example() -> Self {
        let norm = NormFourBar {
            l1: 90.,
            l3: 70.,
            l4: 70.,
            l5: 45.,
            g: FRAC_PI_6,
            stat: Stat::C1B1,
        };
        Self::new(UnNorm::from_driver(35.), norm)
    }
}

impl Statable for NormFourBar {
    fn stat_mut(&mut self) -> &mut Stat {
        &mut self.stat
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl PlanarLoop for NormFourBar {
    fn planar_loop(&self) -> [f64; 4] {
        [self.l1, 1., self.l3, self.l4]
    }
}

impl PlanarLoop for FourBar {
    fn planar_loop(&self) -> [f64; 4] {
        [self.l1, self.unnorm.l2, self.l3, self.l4]
    }
}

impl Transformable<2> for FourBar {
    fn transform_inplace(&mut self, geo: &efd::GeoVar2) {
        self.unnorm.transform_inplace(geo);
        self.norm.scale_inplace(geo.scale());
    }
}

impl CurveGen<2> for FourBar {
    fn pos_s(&self, t: f64, inv: bool) -> Option<[[f64; 2]; 5]> {
        curve_interval((&self.unnorm, &self.norm), t, inv)
    }
}

fn angle(p: na::Point2<f64>, d: f64, a: f64) -> na::Point2<f64> {
    p + d * na::Vector2::new(a.cos(), a.sin())
}

pub(crate) fn curve_interval(
    fb: (&UnNorm, &NormFourBar),
    b: f64,
    inv: bool,
) -> Option<[[f64; 2]; 5]> {
    let UnNorm { p1x, p1y, a, l2 } = *fb.0;
    let NormFourBar { l1, l3, l4, l5, g, .. } = *fb.1;
    let p1 = na::Point2::new(p1x, p1y);
    let p2 = angle(p1, l1, a);
    let p3 = angle(p1, l2, a + b);
    let p4 = if (l1 - l3).abs() < f64::EPSILON && (l2 - l4).abs() < f64::EPSILON {
        // Special case
        p2 + (p3 - p1)
    } else {
        let p23 = p2 - p3;
        let r_2 = p23.norm_squared();
        let r = r_2.sqrt();
        if r > l3 + l4 || r < (l3 - l4).abs() || r < f64::EPSILON {
            [f64::NAN; 2].into()
        } else {
            let l3_2 = l3 * l3;
            let c = (l3_2 - l4 * l4 + r_2) / (2. * r);
            let s = (l3_2 - c * c).sqrt();
            let rot = na::UnitComplex::from_cos_sin_unchecked(c, s);
            let rot = if inv { rot.conjugate() } else { rot };
            p3 + rot * (p23 / r)
        }
    };
    let p5 = {
        let p43 = p4 - p3;
        angle(p3, l5, g + p43.y.atan2(p43.x))
    };
    macro_rules! build_coords {
        [$($p:ident),+] => { [$([$p.x, $p.y]),+] }
    }
    let js = build_coords![p1, p2, p3, p4, p5];
    js.iter().flatten().all(|x| x.is_finite()).then_some(js)
}
