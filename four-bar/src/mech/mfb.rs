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
#[repr(C)] // Ensure the same memory layout as `NormFourBar`
pub struct MNormFourBar {
    /// Base parameters
    pub base: NormFourBar,
    /// Angle of the motion line on the coupler
    pub e: f64,
}

impl MNormFourBar {
    /// Generator for coupler curve.
    pub fn curve(&self, res: usize) -> Vec<[f64; 2]> {
        CurveGen::curve(self, res)
    }

    /// Check if the linkage is valid.
    pub fn is_valid(&self) -> bool {
        Statable::is_valid(self)
    }

    /// Check if the bounds is open.
    pub fn is_open(&self) -> bool {
        Statable::is_open(self)
    }

    /// Obtain the continuous pose in the range of motion.
    pub fn pose(&self, res: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        PoseGen::pose(self, res)
    }

    /// Pose generation for coupler curve in zipped form.
    pub fn pose_zipped(&self, res: usize) -> Vec<([f64; 2], [f64; 2])> {
        PoseGen::pose_zipped(self, res)
    }

    /// Obtain the coupler curve and the extended curve in the range of motion.
    pub fn ext_curve(&self, length: f64, res: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        PoseGen::ext_curve(self, length, res)
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

    /// Convert to a four-bar linkage reference.
    pub const fn as_fb(&self) -> &FourBar {
        // Compile time checks memory layout compatibility.
        const _: () = {
            let mfb = MFourBar::example();
            let fb = mfb.as_fb();
            assert!(fb.unnorm.p1x == mfb.unnorm.p1x);
            assert!(fb.unnorm.l2 == mfb.unnorm.l2);
            assert!(fb.norm.l1 == mfb.norm.base.l1);
            assert!(fb.norm.g == mfb.norm.base.g);
        };
        // Safety: `MFourBar` and `FourBar` have the same memory layout.
        unsafe { &*(self as *const Self as *const FourBar) }
    }

    /// Convert to a mutable four-bar linkage reference.
    pub fn as_fb_mut(&mut self) -> &mut FourBar {
        // Safety: `MFourBar` and `FourBar` have the same memory layout.
        unsafe { &mut *(self as *mut Self as *mut FourBar) }
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
