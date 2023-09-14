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
use std::{borrow::Cow, f64::consts::*, marker::PhantomData};

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
        Self { efd, mode, res: 180, _marker: PhantomData }
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
pub trait SynBound: Clone + Sync + Send {
    /// Lower & upper bounds
    const BOUND: &'static [[f64; 2]];
    /// Bound number for non-partial matching
    const BOUND_NUM: usize;

    /// Create entity from slice.
    fn from_slice(xs: &[f64]) -> Self;
    /// List all the states of the design.
    fn get_states(&self) -> Vec<Cow<Self>>;
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

    fn get_states(&self) -> Vec<Cow<Self>> {
        vec![Cow::Borrowed(self), Cow::Owned(self.clone().with_inv(true))]
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

    fn get_states(&self) -> Vec<Cow<Self>> {
        vec![Cow::Borrowed(self), Cow::Owned(self.clone().with_inv(true))]
    }
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
            &M::BOUND[..M::BOUND_NUM]
        }
    }
}

impl<D, M> mh::ObjFunc for Syn<D, M>
where
    D: efd::EfdDim + Sync + Send,
    D::Trans: Sync + Send,
    efd::Coord<D>: Sync + Send,
    M: SynBound + Normalized<D> + CurveGen<D>,
    M::De: Default + Clone + CurveGen<D> + Sync + Send + 'static,
{
    type Fitness = mh::Product<M::De, f64>;

    fn fitness(&self, xs: &[f64]) -> Self::Fitness {
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        const INFEASIBLE: f64 = 1e10;
        let infeasible = || mh::Product::new(INFEASIBLE, M::De::default());
        let fb = M::from_slice(&xs[..M::BOUND_NUM]);
        let bound = fb.angle_bound().check_mode(self.mode.is_result_open());
        let is_open = self.mode.is_target_open();
        let f = |[t1, t2]: [f64; 2]| {
            let states = fb.get_states();
            #[cfg(feature = "rayon")]
            let iter = states.into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = states.into_iter();
            iter.map(move |fb| (fb.curve_in(t1, t2, self.res), fb))
                .filter(|(c, _)| c.len() > 2)
                .map(|(c, fb)| {
                    let efd = efd::Efd::<D>::from_curve_harmonic(c, is_open, self.efd.harmonic());
                    let trans = efd.as_trans().to(self.efd.as_trans());
                    let fb = fb.trans_denorm(&trans);
                    let err = efd.distance(&self.efd);
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
                let bound = [
                    AngleBound::Open(xs[M::BOUND_NUM], xs[M::BOUND_NUM + 1]),
                    AngleBound::Open(xs[M::BOUND_NUM + 1], xs[M::BOUND_NUM]),
                ];
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
