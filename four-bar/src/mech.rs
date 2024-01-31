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

    /// Generator for all positions in specified angle.
    ///
    /// For optimization reason, this method is required to specialize if
    /// [`CurveGen::pos_s()`] is inherited from a base type.
    fn pos_iter<I>(&self, iter: I) -> impl Iterator<Item = [efd::Coord<D>; 5]>
    where
        I: IntoIterator<Item = f64>,
    {
        let inv = self.inv();
        iter.into_iter().filter_map(move |t| self.pos_s(t, inv))
    }

    /// Generator for all curves in specified angle.
    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        self.pos_iter(linspace(start, end, res))
            .map(|[.., p3, p4, p5]| [p3, p4, p5])
            .collect()
    }

    /// Generator for coupler curve in specified angle.
    fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<efd::Coord<D>> {
        self.pos_iter(linspace(start, end, res))
            .map(|[.., p5]| p5)
            .collect()
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

    /// Generator for coupler curve by an input angle list.
    fn curve_by(&self, t: &[f64]) -> Vec<efd::Coord<D>> {
        self.pos_iter(t.iter().copied())
            .map(|[.., p5]| p5)
            .collect()
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

    fn pos_iter<I>(&self, iter: I) -> impl Iterator<Item = [efd::Coord<D>; 5]>
    where
        I: IntoIterator<Item = f64>,
    {
        let de = self.clone().denormalize();
        let inv = de.inv();
        iter.into_iter().filter_map(move |t| de.pos_s(t, inv))
    }
}

fn linspace(start: f64, end: f64, res: usize) -> impl Iterator<Item = f64> {
    use std::f64::consts::TAU;
    let end = if end > start { end } else { end + TAU };
    let step = (end - start) / res as f64;
    (0..res).map(move |n| start + n as f64 * step)
}

/// Pose generation behavior.
pub trait PoseGen<const D: usize>: CurveGen<D> {
    /// Obtain the pose (an unit vector) from known position.
    fn uvec(&self, pos: [efd::Coord<D>; 5]) -> efd::Coord<D>;

    /// Generator for all poses in specified angle.
    ///
    /// For optimization reason, this method is required to specialize if
    /// [`CurveGen::pos_s()`] and [`PoseGen::uvec()`] are inherited from a base
    /// type.
    fn uvec_iter<I>(&self, iter: I) -> impl Iterator<Item = (efd::Coord<D>, efd::Coord<D>)>
    where
        I: IntoIterator<Item = f64>,
    {
        self.pos_iter(iter)
            .map(|pos @ [.., p5]| (p5, self.uvec(pos)))
    }

    /// Obtain the continuous pose from known position.
    fn pose_in(
        &self,
        start: f64,
        end: f64,
        res: usize,
    ) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        self.uvec_iter(linspace(start, end, res)).unzip()
    }

    /// Obtain the continuous pose in the range of motion.
    fn pose(&self, res: usize) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        self.angle_bound()
            .to_value()
            .map(|[start, end]| self.pose_in(start, end, res))
            .unwrap_or_default()
    }

    /// Obtain the continuous pose by an input angle list.
    fn pose_by(&self, t: &[f64]) -> (Vec<efd::Coord<D>>, Vec<efd::Coord<D>>) {
        self.uvec_iter(t.iter().copied()).unzip()
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

    fn uvec_iter<I>(&self, iter: I) -> impl Iterator<Item = (efd::Coord<D>, efd::Coord<D>)>
    where
        I: IntoIterator<Item = f64>,
    {
        let de = self.clone().denormalize();
        let inv = de.inv();
        iter.into_iter().filter_map(move |t| {
            let pos @ [.., p5] = de.pos_s(t, inv)?;
            Some((p5, de.uvec(pos)))
        })
    }
}
