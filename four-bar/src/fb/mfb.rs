//! Planar four-bar linkages used in motion synthesis.
use super::{fb2d::NormFourBar, *};
use efd::na;

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
    base: NormFourBar,
    /// Angle of the motion line on the coupler
    e: f64,
}

impl FromVectorized for MNormFourBar {
    type Dim = na::U6;

    fn from_vectorized(v: &[f64], stat: Stat) -> Result<Self, std::array::TryFromSliceError> {
        let [l1, l3, l4, l5, g, e] = <[f64; 6]>::try_from(v)?;
        Ok(Self { base: NormFourBar { l1, l3, l4, l5, g, stat }, e })
    }
}

/// Motion four-bar linkage with offset.
///
/// Similar to [`fb2d::FourBar`] but with a motion line angle.
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
pub type MFourBar = FourBarBase<fb2d::UnNorm, MNormFourBar>;
