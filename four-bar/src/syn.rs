//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{mh, syn};
//!
//! # let curve = vec![[0., 0.], [1., 0.], [2., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! # let res = 3;
//! let func = syn::FbSyn::from_curve(curve, syn::Mode::Closed)
//!     .expect("invalid curve")
//!     .res(res);
//! let s = mh::Solver::build(mh::Rga::default(), func)
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .solve()
//!     .unwrap();
//! ```
use crate::{efd::Curve, *};
use std::{f64::consts::*, marker::PhantomData};

/// The minimum input angle bound. (π/16)
pub const MIN_ANGLE: f64 = FRAC_PI_8 * 0.5;
/// Boundary of the planar objective variables.
pub const BOUND2D: &[[f64; 2]] = <NormFourBar as SynBound>::BOUND;
/// Boundary of the spherical objective variables.
pub const BOUND3D: &[[f64; 2]] = <SNormFourBar as SynBound>::BOUND;

/// Path generation task of planar four-bar linkage.
pub type FbSyn = Syn<efd::D2, NormFourBar>;
/// Path generation task of spherical four-bar linkage.
pub type SFbSyn = Syn<efd::D3, SNormFourBar>;

/// Path generation task of four-bar linkage.
pub struct Syn<D: efd::EfdDim, M> {
    /// Target coefficients
    pub efd: efd::Efd<D>,
    // Mode
    mode: Mode,
    // How many points need to be generated or compared
    res: usize,
    _marker: PhantomData<M>,
}

impl<D: efd::EfdDim, M> Syn<D, M> {
    /// Create a new task from target curve. The harmonic number is selected
    /// automatically.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    pub fn from_curve<C>(curve: C, mode: Mode) -> Option<Self>
    where
        C: Curve<efd::Coord<D>>,
    {
        let efd = efd::Efd::from_curve(mode.regularize(curve))?;
        Some(Self::from_efd(efd, mode))
    }

    /// Create a new task from target curve and harmonic number.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    pub fn from_curve_harmonic<C, H>(curve: C, harmonic: H, mode: Mode) -> Option<Self>
    where
        C: Curve<efd::Coord<D>>,
        Option<usize>: From<H>,
    {
        let efd = efd::Efd::from_curve_harmonic(mode.regularize(curve), harmonic)?;
        Some(Self::from_efd(efd, mode))
    }

    /// Create a new task from target curve and Fourier power gate.
    ///
    /// Return none if the curve length is less than 1.
    pub fn from_curve_gate<C, T>(curve: C, threshold: T, mode: Mode) -> Option<Self>
    where
        C: Curve<efd::Coord<D>>,
        Option<f64>: From<T>,
    {
        let efd = efd::Efd::from_curve_gate(mode.regularize(curve), threshold)?;
        Some(Self::from_efd(efd, mode))
    }

    /// Create a new task from target EFD coefficients.
    ///
    /// Please use threshold or harmonic to create the EFD object. The curve
    /// must preprocess with [`Mode::regularize()`] method before turned into
    /// EFD.
    pub fn from_efd(efd: efd::Efd<D>, mode: Mode) -> Self {
        Self { efd, mode, res: 720, _marker: PhantomData }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }
}

/// Synthesis bounds.
pub trait SynBound: Clone {
    /// Lower & upper bounds
    const BOUND: &'static [[f64; 2]];
    /// Bound number for non-partial matching
    const BOUND_NUM: usize;

    /// Create entity from slice.
    fn from_slice(xs: &[f64]) -> Self;
    /// Change inverse symbol.
    fn with_inv(self, inv: bool) -> Self;
}

impl SynBound for NormFourBar {
    const BOUND: &'static [[f64; 2]] = {
        const BOUND_F: f64 = 6.;
        const BOUND_FF: f64 = 1. / BOUND_F;
        &[
            [BOUND_FF, BOUND_F],
            [BOUND_FF, BOUND_F],
            [BOUND_FF, BOUND_F],
            [BOUND_FF, BOUND_F],
            [0., TAU],
            [0., TAU],
            [0., TAU],
        ]
    };
    const BOUND_NUM: usize = 5;

    fn from_slice(xs: &[f64]) -> Self {
        Self::try_from(xs).unwrap()
    }

    fn with_inv(self, inv: bool) -> Self {
        self.with_inv(inv)
    }
}

impl SynBound for SNormFourBar {
    const BOUND: &'static [[f64; 2]] = &[
        [0., PI],
        [0., PI],
        [0., PI],
        [0., PI],
        [0., PI],
        [0., PI],
        [0., PI],
        [0., PI],
    ];
    const BOUND_NUM: usize = 6;

    fn from_slice(xs: &[f64]) -> Self {
        Self::try_from(xs).unwrap()
    }

    fn with_inv(self, inv: bool) -> Self {
        self.with_inv(inv)
    }
}

impl<D, M> mh::Bounded for Syn<D, M>
where
    D: efd::EfdDim + Sync + Send,
    D::Trans: Sync + Send,
    efd::Coord<D>: Sync + Send,
    M: SynBound + Sync + Send,
{
    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if matches!(self.mode, Mode::Partial) {
            M::BOUND
        } else {
            &M::BOUND[..M::BOUND_NUM]
        }
    }
}

impl<D, M> mh::ObjFactory for Syn<D, M>
where
    D: efd::EfdDim + Sync + Send,
    D::Trans: Sync + Send,
    efd::Coord<D>: Sync + Send,
    M: SynBound + Normalized + CurveGen<D> + Sync + Send,
    <M as Normalized>::Target: Default + CurveGen<D> + Transformable<D> + Sync + Send,
{
    type Product = (f64, <M as Normalized>::Target);
    type Eval = f64;

    fn produce(&self, xs: &[f64]) -> Self::Product {
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        const INFEASIBLE: f64 = 1e10;
        let fb = M::from_slice(&xs[..M::BOUND_NUM]);
        if self.mode.is_result_open() != fb.is_open_curve() {
            return (INFEASIBLE, Default::default());
        }
        let f = |[t1, t2]: [f64; 2]| {
            let fb = &fb;
            #[cfg(feature = "rayon")]
            let iter = [false, true].into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = [false, true].into_iter();
            iter.map(move |inv| {
                let fb = fb.clone().with_inv(inv);
                let curve = fb.curve_in(t1, t2, self.res);
                (curve, fb)
            })
            .filter(|(c, _)| c.len() > 1)
            .map(|(c, fb)| {
                let c = self.mode.regularize(c);
                let efd = efd::Efd::<D>::from_curve_harmonic(c, self.efd.harmonic()).unwrap();
                let fb = fb
                    .denormalize()
                    .transform(&efd.as_trans().to(self.efd.as_trans()));
                (efd.l1_norm(&self.efd), fb)
            })
        };
        match self.mode {
            Mode::Closed | Mode::Open => fb
                .angle_bound()
                .filter(|[t1, t2]| t2 - t1 > MIN_ANGLE)
                .and_then(|t| f(t).min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap()))
                .unwrap_or((INFEASIBLE, Default::default())),
            Mode::Partial => {
                let bound = [
                    [xs[M::BOUND_NUM], xs[M::BOUND_NUM + 1]],
                    [xs[M::BOUND_NUM + 1], xs[M::BOUND_NUM]],
                ];
                #[cfg(feature = "rayon")]
                let iter = bound.into_par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = bound.into_iter();
                iter.map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                    .filter(|[t1, t2]| t2 - t1 > MIN_ANGLE)
                    .flat_map(f)
                    .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                    .unwrap_or((INFEASIBLE, Default::default()))
            }
        }
    }

    fn evaluate(&self, (f, _): Self::Product) -> Self::Eval {
        f
    }
}

/// Synthesis mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    /// Closed path matching
    Closed,
    /// Use closed path to match open path
    Partial,
    /// Open path matching
    Open,
}

impl Mode {
    /// Return true if the target curve is open.
    pub const fn is_target_open(&self) -> bool {
        !self.is_target_close()
    }

    /// Return true if the target curve is open.
    pub const fn is_target_close(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Return true if the synthesis curve is open.
    pub const fn is_result_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Return true if the synthesis curve is close.
    pub const fn is_result_closed(&self) -> bool {
        !self.is_result_open()
    }

    /// Regularize curve with the mode.
    pub fn regularize<A, C>(&self, curve: C) -> Vec<A>
    where
        A: PartialEq + Clone,
        C: Curve<A>,
    {
        match self {
            _ if curve.is_closed() => curve.to_curve(),
            Self::Closed => curve.closed_lin(),
            Self::Partial | Self::Open => curve.closed_rev(),
        }
    }
}
