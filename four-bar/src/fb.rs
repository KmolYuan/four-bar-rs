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
    /// Non-Grashof Double rocker (ground link is the longest)
    RRR1,
    /// Non-Grashof Double rocker (driver link is the longest)
    RRR2,
    /// Non-Grashof Double rocker (coupler link is the longest)
    RRR3,
    /// Non-Grashof Double rocker (follower link is the longest)
    RRR4,
}

impl FourBarTy {
    /// Detect from four-bar loop `[l0, l1, l2, l3]`.
    pub fn from_loop(mut fb_loop: [f64; 4]) -> Self {
        let [l0, l1, l2, l3] = fb_loop;
        fb_loop.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let [s, p, q, l] = fb_loop;
        macro_rules! arms {
            ($d:expr => $c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
                match $d {
                    d if d == l0 => $c1,
                    d if d == l1 => $c2,
                    d if d == l2 => $c3,
                    d if d == l3 => $c4,
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
            Self::GCCC => "Grashof double crank (Drag-link) (GCCC)",
            Self::GCRR => "Grashof crank rocker (GCRR)",
            Self::GRCR => "Grashof double rocker (GRCR)",
            Self::GRRC => "Grashof rocker crank (GRRC)",
            Self::RRR1 => "Non-Grashof Double rocker (RRR1)",
            Self::RRR2 => "Non-Grashof Double rocker (RRR2)",
            Self::RRR3 => "Non-Grashof Double rocker (RRR3)",
            Self::RRR4 => "Non-Grashof Double rocker (RRR4)",
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

pub(crate) fn angle_bound([l0, l1, l2, l3]: [f64; 4]) -> [f64; 2] {
    match (l0 + l1 <= l2 + l3, (l0 - l1).abs() >= (l2 - l3).abs()) {
        (true, true) => [0., TAU],
        (true, false) => {
            let l23 = l2 - l3;
            let d = (l0 * l0 + l1 * l1 - l23 * l23) / (2. * l0 * l1);
            [d.acos(), TAU - d.acos()]
        }
        (false, true) => {
            let l23 = l2 + l3;
            let d = (l0 * l0 + l1 * l1 - l23 * l23) / (2. * l0 * l1);
            [-d.acos(), d.acos()]
        }
        (false, false) => {
            let up = l0 * l0 + l1 * l1;
            let down = 2. * l0 * l1;
            let l23 = l2 - l3;
            let d1 = (up - l23 * l23) / down;
            let l23 = l2 + l3;
            let d2 = (up - l23 * l23) / down;
            [d1.acos(), d2.acos()]
        }
    }
}

/// Normalized four-bar base.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(default))]
#[derive(Clone, Default, PartialEq)]
pub struct NormFourBarBase<B> {
    /// Buffer
    pub buf: B,
    /// Inverse
    pub inv: bool,
}

/// Four-bar base.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(default))]
#[derive(Clone, Default, PartialEq)]
pub struct FourBarBase<B, NB> {
    /// Buffer
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

    /// Generator for coupler curve.
    pub fn curve<D: EfdDim>(&self, res: usize) -> Vec<efd::Coord<D>>
    where
        Self: CurveGen<D>,
    {
        <Self as CurveGen<D>>::curve(self, res)
    }
}

impl<B, NB> FourBarBase<B, NB> {
    /// Create a new value from buffer.
    pub const fn new_norm(buf: B, buf_norm: NB, inv: bool) -> Self {
        Self { buf, norm: NormFourBarBase { buf: buf_norm, inv } }
    }

    /// Build with inverter.
    pub const fn with_inv(mut self, inv: bool) -> Self {
        self.norm.inv = inv;
        self
    }

    /// Generator for coupler curve.
    pub fn curve<D: EfdDim>(&self, res: usize) -> Vec<efd::Coord<D>>
    where
        Self: CurveGen<D>,
    {
        <Self as CurveGen<D>>::curve(self, res)
    }
}

/// A normalized data type. This type can denormalized to another.
pub trait Normalized<D: efd::EfdDim>: Sized {
    /// Denormalized target.
    type De: Transformable<D>;
    /// Method to convert types.
    fn denormalize(&self) -> Self::De;
    /// Inverse method to convert types.
    fn normalize(de: Self::De) -> Self;

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

/// Curve-generating behavior.
pub trait CurveGen<D: efd::EfdDim>: Sized {
    /// Check if the data is valid.
    fn is_valid(&self) -> bool;
    /// Check if the curve is open.
    fn is_open_curve(&self) -> bool;
    /// Get the position with input angle.
    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]>;
    /// Input angle bounds of the linkage.
    ///
    /// Return `None` if unsupported.
    fn angle_bound(&self) -> Option<[f64; 2]>;

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
            .map(|[start, end]| self.curves_in(start, end, res))
            .unwrap_or_default()
    }

    /// Generator for coupler curve.
    fn curve(&self, res: usize) -> Vec<efd::Coord<D>> {
        self.angle_bound()
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
    fn is_valid(&self) -> bool {
        self.denormalize().is_valid()
    }

    fn is_open_curve(&self) -> bool {
        self.denormalize().is_open_curve()
    }

    fn pos(&self, t: f64) -> Option<[efd::Coord<D>; 5]> {
        self.denormalize().pos(t)
    }

    fn angle_bound(&self) -> Option<[f64; 2]> {
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
