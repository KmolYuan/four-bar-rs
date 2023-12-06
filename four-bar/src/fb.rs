//! Four-bar linkage types.
pub use self::{
    fb2d::{FourBar, NormFourBar},
    fb3d::{SFourBar, SNormFourBar},
    stat::*,
    vectorized::*,
};
use crate::efd::EfdDim;

pub mod fb2d;
pub mod fb3d;
#[cfg(feature = "serde")]
mod fb_serde;
mod stat;
mod vectorized;

/// Four-bar base.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FourBarBase<UN, NM> {
    /// Buffer
    pub unnorm: UN,
    /// Normalized base
    pub norm: NM,
}

impl<UN, NM> FourBarBase<UN, NM> {
    /// Create a new value from inner values.
    pub const fn new(unnorm: UN, norm: NM) -> Self {
        Self { unnorm, norm }
    }

    /// Build with inverter.
    pub fn with_stat(self, stat: Stat) -> Self
    where
        NM: Statable,
    {
        Self { norm: self.norm.with_stat(stat), ..self }
    }

    /// Get the state.
    pub fn stat(&self) -> Stat
    where
        NM: Statable,
    {
        self.norm.stat()
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy
    where
        Self: Statable,
    {
        Statable::ty(self)
    }

    /// Normalization.
    pub fn normalize(self) -> NM {
        self.norm
    }

    /// Curve generation for coupler curve.
    pub fn curve<D: EfdDim>(&self, res: usize) -> Vec<efd::Coord<D>>
    where
        Self: CurveGen<D>,
    {
        <Self as CurveGen<D>>::curve(self, res)
    }

    /// Check if the data is valid.
    pub fn is_valid(&self) -> bool
    where
        Self: Statable,
    {
        self.ty().is_valid()
    }

    /// Input angle bounds of the linkage.
    pub fn angle_bound(&self) -> AngleBound
    where
        Self: Statable,
    {
        Statable::angle_bound(self)
    }
}

impl<UN, NM> std::ops::Deref for FourBarBase<UN, NM> {
    type Target = NM;

    fn deref(&self) -> &Self::Target {
        &self.norm
    }
}

impl<UN, NM> std::ops::DerefMut for FourBarBase<UN, NM> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.norm
    }
}

/// A normalized data type. This type can denormalized to another.
///
/// Usually, this type is smaller than the denormalized type.
pub trait Normalized<D: efd::EfdDim>: Sized {
    /// Denormalized target, which should be transformable.
    type De: Transformable<D>;
    /// Method to convert types.
    ///
    /// Usually, the data will become bigger.
    fn denormalize(self) -> Self::De;
    /// Inverse method to convert types.
    fn normalize(de: Self::De) -> Self;

    /// Normalize in-placed.
    ///
    /// For optimization reason, this method is required to specialize.
    fn normalize_inplace(de: &mut Self::De);

    /// Denormalized with transformation.
    fn trans_denorm(self, trans: &efd::Transform<D::Trans>) -> Self::De {
        self.denormalize().transform(trans)
    }
}

/// Transformation ability.
pub trait Transformable<D: efd::EfdDim>: Sized {
    /// Transform in placed.
    fn transform_inplace(&mut self, trans: &efd::Transform<D::Trans>);

    /// Build with transformation.
    fn transform(mut self, trans: &efd::Transform<D::Trans>) -> Self {
        self.transform_inplace(trans);
        self
    }
}

/// Curve-generating behavior.
pub trait CurveGen<D: efd::EfdDim>: Statable {
    /// Get the position with inversion flag.
    fn pos_s(&self, t: f64, inv: bool) -> Option<[efd::Coord<D>; 5]>;

    /// Get the position with input angle.
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]> {
        self.pos_s(t, self.inv())
    }

    /// Generator for all curves in specified angle.
    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        let inv = self.inv();
        curve_in(
            start,
            end,
            res,
            |t| self.pos_s(t, inv),
            |[.., p3, p4, p5]| [p3, p4, p5],
        )
    }

    /// Generator for coupler curve in specified angle.
    fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<efd::Coord<D>> {
        let inv = self.inv();
        curve_in(start, end, res, |t| self.pos_s(t, inv), |[.., p5]| p5)
    }

    /// Generator for curves.
    fn curves(&self, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        self.angle_bound()
            .to_value()
            .map(|[start, end]| self.curves_in(start, end, res))
            .unwrap_or_default()
    }

    /// Generator for coupler curve.
    fn curve(&self, res: usize) -> Vec<efd::Coord<D>> {
        self.angle_bound()
            .to_value()
            .map(|[start, end]| self.curve_in(start, end, res))
            .unwrap_or_default()
    }
}

impl<D, N> CurveGen<D> for N
where
    D: efd::EfdDim,
    N: Normalized<D> + Statable + Clone,
    N::De: CurveGen<D>,
{
    fn pos_s(&self, t: f64, inv: bool) -> Option<[efd::Coord<D>; 5]> {
        self.clone().denormalize().pos_s(t, inv)
    }

    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        self.clone().denormalize().curves_in(start, end, res)
    }

    fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<efd::Coord<D>> {
        self.clone().denormalize().curve_in(start, end, res)
    }
}

fn curve_in<C, F, M, B>(start: f64, end: f64, res: usize, f: F, map: M) -> Vec<B>
where
    C: Clone,
    F: Fn(f64) -> Option<[C; 5]>,
    M: Fn([C; 5]) -> B + Copy,
{
    use std::f64::consts::TAU;
    let end = if end > start { end } else { end + TAU };
    let step = (end - start) / res as f64;
    (0..res)
        .map(|n| start + n as f64 * step)
        .flat_map(f)
        .map(map)
        .collect()
}
