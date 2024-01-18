//! Planar four-bar linkages used in motion synthesis.
use super::{fb::*, *};
use efd::na;
use std::f64::consts::FRAC_PI_6;

/// Unnormalized part of motion four-bar linkage.
///
/// # Parameters
///
/// + Ground link `l1`
/// + Driver link `l2=1`
/// + Coupler link `l3`
/// + Follower link `l4`
/// + Extanded link `l5`
/// + Coupler link angle `g`
/// + Angle of the motion line `e`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MNormFourBar {
    /// Base parameters
    pub base: NormFourBar,
    /// Angle of the motion line on the coupler
    pub e: f64,
}

impl FromVectorized<6> for MNormFourBar {
    fn from_vectorized(v: [f64; 6], stat: Stat) -> Self {
        let [l1, l3, l4, l5, g, e] = v;
        Self { base: NormFourBar { l1, l3, l4, l5, g, stat }, e }
    }
}

impl IntoVectorized for MNormFourBar {
    fn into_vectorized(self) -> (Vec<f64>, Stat) {
        let code = vec![self.l1, self.l3, self.l4, self.l5, self.g, self.e];
        (code, self.stat)
    }
}

impl std::ops::Deref for MNormFourBar {
    type Target = NormFourBar;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for MNormFourBar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// Motion four-bar linkage with offset.
///
/// Similar to [`FourBar`] but with a motion line angle.
///
/// # Parameters
///
/// There are 10 parameters in total.
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
/// + Angle of the motion line `e`
pub type MFourBar = Mech<UnNorm, MNormFourBar>;

impl Normalized<2> for MNormFourBar {
    type De = MFourBar;

    fn denormalize(self) -> Self::De {
        MFourBar { unnorm: UnNorm::new(), norm: self }
    }

    fn normalize(mut de: Self::De) -> Self {
        Self::normalize_inplace(&mut de);
        de.norm
    }

    fn normalize_inplace(de: &mut Self::De) {
        let l2 = std::mem::replace(&mut de.unnorm.l2, 1.);
        de.norm.scale_inplace(l2.recip());
    }
}

impl MFourBar {
    /// Create a motion four-bar linkage from a four-bar linkage and a motion
    /// line angle.
    pub const fn from_fb_angle(fb: FourBar, e: f64) -> Self {
        Self::new(fb.unnorm, MNormFourBar { base: fb.norm, e })
    }

    /// Convert to a four-bar linkage by dropping the motion line angle.
    pub const fn into_fb(self) -> FourBar {
        FourBar { unnorm: self.unnorm, norm: self.norm.base }
    }

    /// An example crank rocker.
    pub const fn example() -> Self {
        Self::from_fb_angle(FourBar::example(), FRAC_PI_6)
    }
}

impl PlanarLoop for MNormFourBar {
    fn planar_loop(&self) -> [f64; 4] {
        [self.l1, 1., self.l3, self.l4]
    }
}

impl PlanarLoop for MFourBar {
    fn planar_loop(&self) -> [f64; 4] {
        [self.l1, self.unnorm.l2, self.l3, self.l4]
    }
}

impl Transformable<2> for MFourBar {
    fn transform_inplace(&mut self, geo: &efd::GeoVar2) {
        self.unnorm.transform_inplace(geo);
        self.norm.scale_inplace(geo.scale());
    }
}

impl CurveGen<2> for MFourBar {
    fn pos_s(&self, t: f64, inv: bool) -> Option<[[f64; 2]; 5]> {
        curve_interval((&self.unnorm, &self.norm), t, inv)
    }
}

impl PoseGen<2> for MFourBar {
    fn uvec(&self, [.., p3, p4, _]: [efd::Coord<2>; 5]) -> efd::Coord<2> {
        let p43 = na::Point2::from(p4) - na::Point2::from(p3);
        let angle = self.e + p43.y.atan2(p43.x);
        [angle.cos(), angle.sin()]
    }
}