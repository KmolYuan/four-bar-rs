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
pub use self::{motion::*, path::*};
use crate::*;
use std::{
    f64::consts::{PI, TAU},
    marker::PhantomData,
};

mod motion;
mod path;

pub(crate) fn infeasible<P: Default>() -> mh::Product<P, f64> {
    mh::Product::new(1e2, P::default())
}

// Concat const slices by their variable names, currently only support
// non-generic slices.
macro_rules! concat_slices {
    ($a:ident $(, $b:ident)+) => {&{
        let mut out = [$a[0]; { $a.len() $(+ $b.len())+ }];
        let mut i = 0;
        while i < $a.len() {
            out[i] = $a[i];
            i += 1;
        }
        $(let mut j = 0;
        while j < $b.len() {
            out[i] = $b[j];
            i += 1;
            j += 1;
        })+
        out
    }};
}

/// Synthesis bounds.
pub trait SynBound: Clone + Sync + Send {
    /// Lower & upper bounds
    const BOUND: &'static [[f64; 2]];
}

impl SynBound for NormFourBar {
    const BOUND: &'static [[f64; 2]] = {
        const K: f64 = 6.;
        const LNK: &[[f64; 2]] = &[[1. / K, K]; 4];
        const ANG: &[[f64; 2]] = &[[0., TAU]; 3];
        concat_slices!(LNK, ANG)
    };
}

impl SynBound for MNormFourBar {
    const BOUND: &'static [[f64; 2]] = {
        const K: f64 = 6.;
        const LNK: &[[f64; 2]] = &[[1. / K, K]; 4];
        const ANG: &[[f64; 2]] = &[[0., TAU]; 4];
        concat_slices!(LNK, ANG)
    };
}

impl SynBound for SNormFourBar {
    const BOUND: &'static [[f64; 2]] = {
        const LNK: &[[f64; 2]] = &[[1e-4, PI]; 5];
        const GMA: &[[f64; 2]] = &[[0., PI]; 1];
        const ANG: &[[f64; 2]] = &[[0., TAU]; 2];
        concat_slices!(LNK, GMA, ANG)
    };
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

macro_rules! impl_bound {
    ($ty:ident) => {
        impl<M, const D: usize> mh::Bounded for $ty<M, D>
        where
            M: SynBound,
            efd::Rot<D>: Sync + Send,
            efd::Coord<D>: Sync + Send,
            efd::U<D>: efd::EfdDim<D>,
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
    };
}

impl_bound!(PathSyn);
impl_bound!(MotionSyn);
