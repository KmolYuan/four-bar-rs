//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{mh, syn};
//!
//! # let curve = vec![[0., 0.], [1., 0.], [2., 0.]];
//! # let gen = 1;
//! # let pop = 2;
//! # let res = 3;
//! let func = syn::FbSyn::from_curve(curve, syn::Mode::Closed).res(res);
//! let s = mh::Solver::build(mh::Rga::default(), func)
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .solve()
//!     .unwrap();
//! ```
use crate::*;
use std::{f64::consts::*, marker::PhantomData};

/// Boundary of the planar objective variables.
pub const BOUND2D: &[[f64; 2]] = <NormFourBar as SynBound>::BOUND;
/// Boundary of the spherical objective variables.
pub const BOUND3D: &[[f64; 2]] = <SNormFourBar as SynBound>::BOUND;

/// Path generation task of planar four-bar linkage.
pub type FbSyn = Syn<efd::D2, NormFourBar>;
/// Path generation task of spherical four-bar linkage.
pub type SFbSyn = Syn<efd::D3, SNormFourBar>;

/// Path generation task of a mechanism.
pub struct Syn<D: efd::EfdDim, M> {
    /// Target coefficients
    pub efd: efd::Efd<D>,
    // Mode
    mode: Mode,
    // How many points need to be generated or compared
    res: usize,
    // Constrain the scale of the mechanism
    scale: Option<f64>,
    // Marker of the mechanism
    _marker: PhantomData<M>,
}

impl<D: efd::EfdDim, M> Syn<D, M> {
    /// Create a new task from target curve. The harmonic number is selected
    /// automatically.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    pub fn from_curve<C>(curve: C, mode: Mode) -> Self
    where
        C: efd::Curve<efd::Coord<D>>,
    {
        Self::from_efd(efd::Efd::from_curve(curve, mode.is_target_open()), mode)
    }

    /// Create a new task from target EFD coefficients.
    pub fn from_efd(efd: efd::Efd<D>, mode: Mode) -> Self {
        Self {
            efd,
            mode,
            res: 180,
            scale: None,
            _marker: PhantomData,
        }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }

    /// Specify the scale of the mechanism.
    pub fn scale(self, scale: Option<f64>) -> Self {
        if let Some(scale) = scale {
            assert!(scale > 0.);
        }
        Self { scale, ..self }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }
}

/// Synthesis bounds.
pub trait SynBound: Clone + Sync + Send {
    /// Lower & upper bounds
    const BOUND: &'static [[f64; 2]];
}

impl SynBound for NormFourBar {
    const BOUND: &'static [[f64; 2]] = {
        let bound = 6.;
        let bound_f: f64 = 1. / bound;
        &[
            [bound_f, bound],
            [bound_f, bound],
            [bound_f, bound],
            [bound_f, bound],
            [0., TAU],
            [0., TAU],
            [0., TAU],
        ]
    };
}

impl SynBound for SNormFourBar {
    const BOUND: &'static [[f64; 2]] = &[[1e-4, PI]; 8];
}

impl<D, M> mh::Bounded for Syn<D, M>
where
    D: efd::EfdDim + Sync + Send,
    D::Trans: Sync + Send,
    efd::Coord<D>: Sync + Send,
    M: SynBound,
{
    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if matches!(self.mode, Mode::Partial) {
            M::BOUND
        } else {
            &M::BOUND[..M::BOUND.len() - 2]
        }
    }
}

impl<D, M> mh::ObjFunc for Syn<D, M>
where
    D: efd::EfdDim + Sync + Send,
    D::Trans: Sync + Send,
    efd::Coord<D>: Sync + Send,
    M: SynBound + fb::Statable + fb::FromVectorized + fb::Normalized<D> + fb::CurveGen<D>,
    M::De: Default + Clone + fb::CurveGen<D> + Sync + Send + 'static,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        const INFEASIBLE: f64 = 1e10;
        let infeasible = || mh::Product::new(INFEASIBLE, M::De::default());
        let fb = M::from_vectorized(&xs[..M::BOUND.len() - 2], 0).unwrap();
        let bound = fb.angle_bound().check_mode(self.mode.is_result_open());
        let is_open = self.mode.is_target_open();
        let states = fb.get_states();
        let f = |[t1, t2]: [f64; 2]| {
            #[cfg(feature = "rayon")]
            let iter = states.par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = states.iter();
            iter.map(move |fb| (fb.curve_in(t1, t2, self.res), fb))
                .filter(|(c, _)| c.len() > 2)
                .map(|(c, fb)| {
                    let efd = efd::Efd::<D>::from_curve_harmonic(c, is_open, self.efd.harmonic());
                    let trans = efd.as_trans().to(self.efd.as_trans());
                    let fb = fb.clone().trans_denorm(&trans);
                    let s_err = self.scale.map(|s| (trans.scale() - s).abs()).unwrap_or(0.);
                    let err = efd.distance(&self.efd).max(s_err);
                    mh::Product::new(err, fb)
                })
        };
        match self.mode {
            Mode::Closed | Mode::Open => bound
                .check_min()
                .to_value()
                .and_then(|t| f(t).min_by(|a, b| a.partial_cmp(b).unwrap()))
                .unwrap_or_else(infeasible),
            Mode::Partial => {
                if !bound.is_valid() {
                    return infeasible();
                }
                let bound = {
                    let end = M::BOUND.len() - 1;
                    fb::AngleBound::open_and_rev_at(xs[end], xs[end - 1])
                };
                #[cfg(feature = "rayon")]
                let iter = bound.into_par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = bound.into_iter();
                iter.filter_map(|b| b.check_min().to_value())
                    .flat_map(f)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or_else(infeasible)
            }
        }
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
        !matches!(self, Self::Closed)
    }

    /// Return true if the synthesis curve is open.
    pub const fn is_result_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}
