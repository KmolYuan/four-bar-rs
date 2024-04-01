//! Planar four-bar linkages used in motion synthesis.
#[doc(no_inline)]
pub use super::{
    fb::{NormFourBar, UnNorm},
    *,
};
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
        de.norm.scale_inplace(de.unnorm.l2.recip());
        de.norm
    }

    fn normalize_inplace(de: &mut Self::De) {
        let l2 = de.unnorm.l2;
        de.unnorm = UnNorm::new();
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
        fb::curve_interval((&self.unnorm, &self.norm), t, inv)
    }
}

impl PoseGen<2> for MFourBar {
    fn uvec(&self, [p3, p4, _]: &[[f64; 2]; 3]) -> [f64; 2] {
        let p43 = na::Point2::from(std::array::from_fn(|i| p4[i] - p3[i]));
        let angle = self.e + (p43.y).atan2(p43.x);
        [angle.cos(), angle.sin()]
    }
}
