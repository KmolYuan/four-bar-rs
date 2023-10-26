//! Planar four-bar linkages.
use super::*;
use std::f64::consts::FRAC_PI_6;

/// Unnormalized part of four-bar linkage.
///
/// Please see [`FourBar`] for more information.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
}

/// Normalized part of four-bar linkage.
///
/// + Buffer order: `[l1, l3, l4, l5, g]`
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2=1`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
/// + Inverse coupler and follower to another circuit
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
    pub stat: bool,
}

impl FromVectorized for NormFourBar {
    type Dim = crate::efd::na::U5;

    fn from_vectorized(v: &[f64], stat: u8) -> Result<Self, std::array::TryFromSliceError> {
        let [l1, l3, l4, l5, g] = <[f64; 5]>::try_from(v)?;
        Ok(Self { l1, l3, l4, l5, g, stat: stat != 0 })
    }
}

impl IntoVectorized for NormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, u8) {
        let code = vec![self.l1, self.l3, self.l4, self.l5, self.g];
        (code, self.stat as u8)
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
pub type FourBar = FourBarBase<UnNorm, NormFourBar>;

impl Normalized<efd::D2> for NormFourBar {
    type De = FourBar;

    fn denormalize(self) -> Self::De {
        FourBar { unnorm: UnNorm::new(), norm: self }
    }

    fn normalize(mut de: Self::De) -> Self {
        Self::normalize_inplace(&mut de);
        de.norm
    }

    fn normalize_inplace(de: &mut Self::De) {
        let l2 = de.unnorm.l2;
        de.norm.l1 /= l2;
        de.norm.l3 /= l2;
        de.norm.l4 /= l2;
        de.norm.l5 /= l2;
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
            stat: false,
        };
        Self::new(UnNorm::from_driver(35.), norm)
    }
}

impl Statable for NormFourBar {
    fn stat(&self) -> u8 {
        self.stat as u8
    }

    fn set_stat(&mut self, stat: u8) {
        self.stat = stat != 0;
    }

    fn get_states(self) -> Vec<Self> {
        let s1 = self.clone().with_stat(1);
        vec![self, s1]
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

impl Transformable<efd::D2> for FourBar {
    fn transform_inplace(&mut self, trans: &efd::Transform2) {
        let fb = &mut self.unnorm;
        let [p1x, p1y] = trans.trans();
        fb.p1x += p1x;
        fb.p1y += p1y;
        fb.a += trans.rot().angle();
        let scale = trans.scale();
        fb.l2 *= scale;
        self.l1 *= scale;
        self.l3 *= scale;
        self.l4 *= scale;
        self.l5 *= scale;
    }
}

impl CurveGen<efd::D2> for FourBar {
    fn pos(&self, t: f64) -> Option<[efd::Coord<efd::D2>; 5]> {
        curve_interval(self, t)
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
        return [f64::NAN; 2];
    }
    let a = 0.5 * (r1 * r1 - r2 * r2 + r * r) / r;
    let h = (r1 * r1 - a * a).sqrt() * if inv { -1. } else { 1. };
    let c = dx / r;
    let s = dy / r;
    let xm = x1 + a * c;
    let ym = y1 + a * s;
    [xm - h * s, ym + h * c]
}

fn curve_interval(fb: &FourBar, b: f64) -> Option<[[f64; 2]; 5]> {
    let UnNorm { p1x, p1y, a, l2 } = fb.unnorm;
    let NormFourBar { l1, l3, l4, l5, g, stat: inv } = fb.norm;
    let p1 = [p1x, p1y];
    let p2 = angle(p1, l1, a);
    let p3 = angle(p1, l2, a + b);
    let p4 = if (l1 - l3).abs() < f64::EPSILON && (l2 - l4).abs() < f64::EPSILON {
        // Special case
        let [p1x, p1y] = p1;
        let [p2x, p2y] = p2;
        let [p3x, p3y] = p3;
        let dx = p3x - p1x;
        let dy = p3y - p1y;
        let d = dx.hypot(dy);
        let a = dy.atan2(dx);
        [p2x + d * a.cos(), p2y + d * a.sin()]
    } else {
        circle2(p3, p2, l3, l4, inv)
    };
    let p5 = angle_with(p3, p4, l5, g);
    let js = [p1, p2, p3, p4, p5];
    js.iter().flatten().all(|x| x.is_finite()).then_some(js)
}
