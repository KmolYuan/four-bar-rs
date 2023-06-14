pub use self::{fb2d::*, fb3d::*};
use crate::efd::EfdDim;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::f64::consts::TAU;

mod fb2d;
mod fb3d;

macro_rules! impl_parm_method {
    ($(#[doc = $doc:literal] fn $name:ident $(,$name_mut:ident)? ($self:ident) -> $ty:ty {$expr:expr})+) => {$(
        #[doc = concat![$doc, "\n\nGet the value."]]
        #[inline]
        pub const fn $name(&$self) -> $ty { $expr }
        $(#[doc = concat![$doc, "\n\nModify the value."]]
        #[inline]
        pub fn $name_mut(&mut $self) -> &mut $ty { &mut $expr })?
    )+};
}

pub(crate) use impl_parm_method;

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
            ($d:expr => $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
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
            arms! { s => Self::GCCC, Self::GCRR, Self::GRCR, Self::GRRC }
        } else {
            arms! { l => Self::RRR1, Self::RRR2, Self::RRR3, Self::RRR4 }
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

/// Normalized four-bar base.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(default))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct NormFourBarBase<B> {
    /// Buffer
    #[cfg_attr(feature = "serde", serde(alias = "v"))]
    pub buf: B,
    /// Inverse
    pub inv: bool,
}

/// Four-bar base.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(default))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FourBarBase<B, NB> {
    /// Buffer
    #[cfg_attr(feature = "serde", serde(alias = "v"))]
    pub buf: B,
    /// Normalized base
    #[cfg_attr(
        feature = "serde",
        serde(bound(deserialize = "NormFourBarBase<NB>: DeserializeOwned"))
    )]
    pub norm: NormFourBarBase<NB>,
}

impl<B> From<B> for NormFourBarBase<B> {
    fn from(buf: B) -> Self {
        Self { buf, inv: false }
    }
}

impl<B, NB> From<NB> for FourBarBase<B, NB>
where
    B: Default,
{
    fn from(norm: NB) -> Self {
        Self {
            buf: B::default(),
            norm: NormFourBarBase::from(norm),
        }
    }
}

impl<A: Copy, const N: usize> TryFrom<&[A]> for NormFourBarBase<[A; N]> {
    type Error = std::array::TryFromSliceError;

    fn try_from(buf: &[A]) -> Result<Self, Self::Error> {
        Ok(Self::from(<[A; N]>::try_from(buf)?))
    }
}

impl<A: Copy, const N: usize, const NB: usize> TryFrom<&[A]> for FourBarBase<[A; N], [A; NB]> {
    type Error = std::array::TryFromSliceError;

    fn try_from(buf: &[A]) -> Result<Self, Self::Error> {
        let buf_norm = &buf[N..];
        let buf = buf[..N].try_into()?;
        Ok(Self { buf, norm: NormFourBarBase::try_from(buf_norm)? })
    }
}

impl<B> NormFourBarBase<B> {
    /// Create a new value from buffer.
    pub const fn new(buf: B, inv: bool) -> Self {
        Self { buf, inv }
    }

    /// Build with inverter.
    pub const fn with_inv(mut self, inv: bool) -> Self {
        self.inv = inv;
        self
    }

    /// Denormalization.
    pub fn denormalize<D: EfdDim>(&self) -> <Self as Normalized<D>>::De
    where
        Self: Normalized<D>,
    {
        <Self as Normalized<D>>::denormalize(self)
    }

    /// Generator for coupler curve.
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
}

impl<B, NB> FourBarBase<B, NB> {
    /// Create a new value from buffer.
    pub const fn new(buf: B, buf_norm: NB, inv: bool) -> Self {
        Self { buf, norm: NormFourBarBase { buf: buf_norm, inv } }
    }

    /// Build with inverter.
    pub const fn with_inv(mut self, inv: bool) -> Self {
        self.norm.inv = inv;
        self
    }

    /// Normalization.
    pub fn normalize<D: EfdDim, N>(&self) -> N
    where
        N: Normalized<D, De = Self>,
    {
        N::normalize(self)
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
}

/// A normalized data type. This type can denormalized to another.
pub trait Normalized<D: efd::EfdDim>: Sized {
    /// Denormalized target, which should be transformable.
    type De: Transformable<D>;
    /// Method to convert types.
    fn denormalize(&self) -> Self::De;
    /// Inverse method to convert types.
    fn normalize(de: &Self::De) -> Self;

    /// Normalize in-placed.
    fn normalize_inplace(de: &mut Self::De) {
        *de = Self::normalize(de).denormalize();
    }

    /// Denormalized with transformation.
    fn trans_denorm(&self, trans: &efd::Transform<D::Trans>) -> Self::De {
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

/// Angle boundary types. The input angle range.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, Default)]
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
pub trait CurveGen<D: efd::EfdDim>: Sized {
    /// Get the position with input angle.
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]>;
    /// Input angle bounds of the linkage.
    fn angle_bound(&self) -> AngleBound;

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
    N: Normalized<D>,
    N::De: CurveGen<D>,
{
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]> {
        self.denormalize().pos(t)
    }

    fn angle_bound(&self) -> AngleBound {
        self.denormalize().angle_bound()
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
