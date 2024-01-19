//! Linkage mechanism types.
pub use self::{
    fb::{FourBar, NormFourBar},
    mfb::{MFourBar, MNormFourBar},
    sfb::{SFourBar, SNormFourBar},
    stat::*,
    vectorized::*,
};

pub mod fb;
pub mod mfb;
pub mod sfb;

#[cfg(feature = "serde")]
mod impl_ser;
mod stat;
mod vectorized;

/// Mechanism base type. Includes normalized and unnormalized data.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Mech<UN, NM> {
    /// Unnormalized part
    pub unnorm: UN,
    /// Normalized part
    pub norm: NM,
}

impl<UN, NM> Mech<UN, NM> {
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
    pub fn curve<const D: usize>(&self, res: usize) -> Vec<efd::Coord<D>>
    where
        Self: CurveGen<D>,
    {
        <Self as CurveGen<D>>::curve(self, res)
    }

    /// Pose generation for coupler curve.
    pub fn pose<const D: usize>(&self, res: usize) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>)
    where
        Self: PoseGen<D>,
    {
        <Self as PoseGen<D>>::pose(self, res)
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

impl<UN, NM> std::ops::Deref for Mech<UN, NM> {
    type Target = NM;

    fn deref(&self) -> &Self::Target {
        &self.norm
    }
}

impl<UN, NM> std::ops::DerefMut for Mech<UN, NM> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.norm
    }
}

/// A normalized data type. This type can denormalized to another.
///
/// Usually, this type is smaller than the denormalized type.
pub trait Normalized<const D: usize>: Sized
where
    efd::U<D>: efd::EfdDim<D>,
{
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
    fn trans_denorm(self, geo: &efd::GeoVar<efd::Rot<D>, D>) -> Self::De {
        self.denormalize().transform(geo)
    }
}

/// Transformation ability.
pub trait Transformable<const D: usize>: Sized
where
    efd::U<D>: efd::EfdDim<D>,
{
    /// Transform in placed.
    fn transform_inplace(&mut self, geo: &efd::GeoVar<efd::Rot<D>, D>);

    /// Build with transformation.
    fn transform(mut self, geo: &efd::GeoVar<efd::Rot<D>, D>) -> Self {
        self.transform_inplace(geo);
        self
    }
}

/// Curve generation behavior.
pub trait CurveGen<const D: usize>: Statable {
    /// Get the position with inversion flag.
    fn pos_s(&self, t: f64, inv: bool) -> Option<[efd::Coord<D>; 5]>;

    /// Get the position with input angle.
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]> {
        self.pos_s(t, self.inv())
    }

    /// Generator for all curves in specified angle.
    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        let inv = self.inv();
        let f = |t| self.pos_s(t, inv);
        curve_in(start, end, res, f, |[.., p3, p4, p5]| [p3, p4, p5])
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

impl<N, const D: usize> CurveGen<D> for N
where
    N: Normalized<D> + Statable + Clone,
    N::De: CurveGen<D>,
    efd::U<D>: efd::EfdDim<D>,
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

/// Pose generation behavior.
pub trait PoseGen<const D: usize>: CurveGen<D> {
    /// Obtain the pose (an unit vector) from known position.
    fn uvec(&self, pos: [efd::Coord<D>; 5]) -> efd::Coord<D>;

    /// Obtain the continuous pose from known position.
    fn pose_in(
        &self,
        start: f64,
        end: f64,
        res: usize,
    ) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        let inv = self.inv();
        let f = |t| self.pos_s(t, inv);
        let map = |c @ [.., p5]: [_; 5]| (p5, self.uvec(c));
        curve_in(start, end, res, f, map).into_iter().unzip()
    }

    /// Obtain the continuous pose in the range of motion.
    fn pose(&self, res: usize) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        self.angle_bound()
            .to_value()
            .map(|[start, end]| self.pose_in(start, end, res))
            .unwrap_or_default()
    }
}

impl<N, const D: usize> PoseGen<D> for N
where
    N: Normalized<D> + Statable + Clone,
    N::De: PoseGen<D>,
    efd::U<D>: efd::EfdDim<D>,
{
    fn uvec(&self, pos: [efd::Coord<D>; 5]) -> efd::Coord<D> {
        self.clone().denormalize().uvec(pos)
    }

    fn pose_in(
        &self,
        start: f64,
        end: f64,
        res: usize,
    ) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        self.clone().denormalize().pose_in(start, end, res)
    }
}
