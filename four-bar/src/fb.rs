//! Four-bar linkage types.
pub use self::{
    fb2d::{FourBar, NormFourBar},
    fb3d::{SFourBar, SNormFourBar},
    vectorized::*,
};
use crate::efd::EfdDim;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

pub mod fb2d;
pub mod fb3d;
#[cfg(feature = "serde")]
mod fb_serde;
mod vectorized;

/// Type of the four-bar linkage.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum FourBarTy {
    /// Grashof double crank (Drag-link)
    GCCC,
    /// Grashof crank rocker
    GCRR,
    /// Grashof double rocker
    GRCR,
    /// Grashof rocker crank
    GRRC,
    /// Non-Grashof triple rocker (ground link is the longest)
    RRR1,
    /// Non-Grashof triple rocker (driver link is the longest)
    RRR2,
    /// Non-Grashof triple rocker (coupler link is the longest)
    RRR3,
    /// Non-Grashof triple rocker (follower link is the longest)
    RRR4,
}

impl FourBarTy {
    /// Detect from four-bar loop `[l1, l2, l3, l4]`.
    pub fn from_loop(mut fb_loop: [f64; 4]) -> Self {
        let [l1, l2, l3, l4] = fb_loop;
        fb_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let [s, p, q, l] = fb_loop;
        macro_rules! arms {
            ($d:expr, $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == l1 => $c1,
                    d if d == l2 => $c2,
                    d if d == l3 => $c3,
                    d if d == l4 => $c4,
                    _ => unreachable!(),
                }
            };
        }
        if s + l < p + q {
            arms!(s, Self::GCCC, Self::GCRR, Self::GRCR, Self::GRRC)
        } else {
            arms!(l, Self::RRR1, Self::RRR2, Self::RRR3, Self::RRR4)
        }
    }

    /// Name of the type.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::GCCC => "Grashof double crank (Drag-link, GCCC)",
            Self::GCRR => "Grashof crank rocker (GCRR)",
            Self::GRCR => "Grashof double rocker (GRCR)",
            Self::GRRC => "Grashof rocker crank (GRRC)",
            Self::RRR1 => "Non-Grashof triple rocker (RRR1)",
            Self::RRR2 => "Non-Grashof triple rocker (RRR2)",
            Self::RRR3 => "Non-Grashof triple rocker (RRR3)",
            Self::RRR4 => "Non-Grashof triple rocker (RRR4)",
        }
    }

    /// Return true if the type is Grashof linkage.
    pub const fn is_grashof(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR | Self::GRCR | Self::GRRC)
    }

    /// Return true if the type has continuous motion.
    pub const fn is_closed_curve(&self) -> bool {
        matches!(self, Self::GCCC | Self::GCRR)
    }

    /// Return true if the type has non-continuous motion.
    pub const fn is_open_curve(&self) -> bool {
        !self.is_closed_curve()
    }
}

/// Four-bar base.
#[cfg_attr(feature = "serde", derive(Deserialize), serde(deny_unknown_fields))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FourBarBase<UN, NM> {
    /// Buffer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub unnorm: UN,
    /// Normalized base
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub norm: NM,
}

impl<UN, NM> FourBarBase<UN, NM> {
    /// Create a new value from inner values.
    pub const fn new(unnorm: UN, norm: NM) -> Self {
        Self { unnorm, norm }
    }

    /// Build with inverter.
    pub fn with_stat(self, stat: u8) -> Self
    where
        NM: Statable,
    {
        Self { norm: self.norm.with_stat(stat), ..self }
    }

    /// Get the state.
    pub fn stat(&self) -> u8
    where
        NM: Statable,
    {
        self.norm.stat()
    }

    /// Return the type of this linkage.
    pub fn ty(&self) -> FourBarTy
    where
        Self: PlanarLoop,
    {
        PlanarLoop::ty(self)
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
    pub fn is_valid<D: EfdDim>(&self) -> bool
    where
        Self: CurveGen<D>,
    {
        <Self as CurveGen<D>>::angle_bound(self).is_valid()
    }

    /// Input angle bounds of the linkage.
    pub fn angle_bound<D>(&self) -> AngleBound
    where
        D: efd::EfdDim,
        Self: CurveGen<D>,
    {
        CurveGen::angle_bound(self)
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

/// State of the linkage.
pub trait Statable: Clone {
    /// Get the state.
    fn stat(&self) -> u8;
    /// Set the state.
    fn set_stat(&mut self, stat: u8);
    /// Get all states from a linkage.
    fn get_states(self) -> Vec<Self>;

    /// Build with state.
    fn with_stat(mut self, stat: u8) -> Self {
        self.set_stat(stat);
        self
    }
}

/// Planar loop of the linkage.
pub trait PlanarLoop {
    /// Get the planar loop.
    fn planar_loop(&self) -> [f64; 4];

    /// Return the type of this linkage.
    fn ty(&self) -> FourBarTy {
        FourBarTy::from_loop(self.planar_loop())
    }
}

/// Angle boundary types. The input angle range.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, PartialEq, Default)]
pub enum AngleBound {
    /// Closed curve
    Closed,
    /// Open curve
    Open(f64, f64),
    /// Invalid
    #[default]
    Invalid,
}

impl AngleBound {
    /// The minimum input angle bound. (π/2)
    pub const MIN_ANGLE: f64 = std::f64::consts::FRAC_PI_2;

    /// Check angle bound from a planar loop.
    pub fn from_planar_loop(mut planar_loop: [f64; 4]) -> Self {
        let [l1, l2, l3, l4] = planar_loop;
        planar_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        if planar_loop[3] > planar_loop[..3].iter().sum() {
            return Self::Invalid;
        }
        match (l1 + l2 <= l3 + l4, (l1 - l2).abs() >= (l3 - l4).abs()) {
            (true, true) => Self::Closed,
            (true, false) => {
                let l33 = l3 - l4;
                let d = (l1 * l1 + l2 * l2 - l33 * l33) / (2. * l1 * l2);
                Self::Open(d.acos(), TAU - d.acos())
            }
            (false, true) => {
                let l33 = l3 + l4;
                let d = (l1 * l1 + l2 * l2 - l33 * l33) / (2. * l1 * l2);
                Self::Open(-d.acos(), d.acos())
            }
            (false, false) => {
                let numerator = l1 * l1 + l2 * l2;
                let denominator = 2. * l1 * l2;
                let l33 = l3 - l4;
                let d1 = (numerator - l33 * l33) / denominator;
                let l33 = l3 + l4;
                let d2 = (numerator - l33 * l33) / denominator;
                Self::Open(d1.acos(), d2.acos())
            }
        }
    }

    /// Create a open and its reverse angle bound.
    pub fn open_and_rev_at(a: f64, b: f64) -> [Self; 2] {
        [Self::Open(a, b), Self::Open(b, a)]
    }

    /// Check the state is the same to the provided mode.
    pub fn check_mode(self, is_open: bool) -> Self {
        match (&self, is_open) {
            (Self::Closed, false) | (Self::Open(_, _), true) => self,
            _ => Self::Invalid,
        }
    }

    /// Angle range must greater than [`AngleBound::MIN_ANGLE`].
    pub fn check_min(self) -> Self {
        match self {
            Self::Open(a, b) => {
                let b = if b > a { b } else { b + TAU };
                if b - a > Self::MIN_ANGLE {
                    self
                } else {
                    Self::Invalid
                }
            }
            _ => self,
        }
    }

    /// Turn into boundary values.
    pub fn to_value(self) -> Option<[f64; 2]> {
        match self {
            Self::Closed => Some([0., TAU]),
            Self::Open(a, b) => {
                let b = if b > a { b } else { b + TAU };
                Some([a, b])
            }
            Self::Invalid => None,
        }
    }

    /// Check if the data is valid.
    pub fn is_valid(&self) -> bool {
        !matches!(self, AngleBound::Invalid)
    }
}

/// Curve-generating behavior.
pub trait CurveGen<D: efd::EfdDim>: PlanarLoop {
    /// Get the position with input angle.
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]>;

    /// Input angle bounds of the linkage.
    fn angle_bound(&self) -> AngleBound {
        AngleBound::from_planar_loop(self.planar_loop())
    }

    /// Generator for all curves in specified angle.
    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        curve_in(
            start,
            end,
            res,
            |t| self.pos(t),
            |[.., p2, p3, p4]| [p2, p3, p4],
        )
    }

    /// Generator for coupler curve in specified angle.
    fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<efd::Coord<D>> {
        curve_in(start, end, res, |t| self.pos(t), |[.., p4]| p4)
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
    N: Normalized<D> + PlanarLoop + Clone,
    N::De: CurveGen<D>,
{
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]> {
        self.clone().denormalize().pos(t)
    }

    fn curves_in(&self, start: f64, end: f64, res: usize) -> Vec<[efd::Coord<D>; 3]> {
        let de = self.clone().denormalize();
        curve_in(
            start,
            end,
            res,
            |t| de.pos(t),
            |[.., p2, p3, p4]| [p2, p3, p4],
        )
    }

    fn curve_in(&self, start: f64, end: f64, res: usize) -> Vec<efd::Coord<D>> {
        let de = self.clone().denormalize();
        curve_in(start, end, res, |t| de.pos(t), |[.., p4]| p4)
    }
}

fn curve_in<C, F, M, B>(start: f64, end: f64, res: usize, f: F, map: M) -> Vec<B>
where
    C: Clone,
    F: Fn(f64) -> Option<[C; 5]>,
    M: Fn([C; 5]) -> B + Copy,
{
    let interval = (end - start) / res as f64;
    let mut iter = (0..res).map(move |n| start + n as f64 * interval).map(f);
    let mut last = Vec::new();
    while iter.len() > 0 {
        last = iter.by_ref().map_while(|c| c).map(map).collect();
    }
    last
}
