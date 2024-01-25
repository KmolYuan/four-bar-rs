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
use std::marker::PhantomData;

mod motion;
mod path;

pub(crate) fn infeasible<P: Default>() -> mh::Product<P, f64> {
    mh::Product::new(1e2, P::default())
}

pub(crate) const fn slice_to_array<const N: usize>(slice: &[f64]) -> [f64; N] {
    let mut out = [0.; N];
    let mut i = 0;
    while i < N {
        out[i] = slice[i];
        i += 1;
    }
    out
}

/// Synthesis bounds.
pub trait SynBound<const N: usize>: mech::Statable + mech::FromVectorized<N> + Sync + Send {}
impl<T, const N: usize> SynBound<N> for T where
    T: mech::Statable + mech::FromVectorized<N> + Sync + Send
{
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
        impl<M, const N: usize, const D: usize> mh::Bounded for $ty<M, N, D>
        where
            M: mech::FromVectorized<N> + Sync + Send,
            efd::Efd<D>: Sync + Send,
            efd::U<D>: efd::EfdDim<D>,
        {
            #[inline]
            fn bound(&self) -> &[[f64; 2]] {
                if matches!(self.mode, Mode::Partial) {
                    M::BOUND_PARTIAL
                } else {
                    &M::BOUND
                }
            }
        }
    };
}

impl_bound!(PathSyn);
impl_bound!(MotionSyn);
