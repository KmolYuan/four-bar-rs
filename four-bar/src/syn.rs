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

pub(crate) const fn slice_to_array<const N: usize>(slice: &[f64]) -> [f64; N] {
    let mut out = [0.; N];
    let mut i = 0;
    while i < N {
        out[i] = slice[i];
        i += 1;
    }
    out
}

// Concat const slices by their variable names, currently only support
// non-generic slices.
const fn concat_slices<T: Copy, const N1: usize, const N2: usize, const N3: usize>(
    a: [T; N1],
    b: [T; N2],
) -> [T; N3]
where
    efd::na::Const<N1>: efd::na::DimNameAdd<efd::na::Const<N2>, Output = efd::na::Const<N3>>,
{
    let mut out = [a[0]; N3];
    let mut i = 0;
    while i < N1 {
        out[i] = a[i];
        i += 1;
    }
    let mut j = 0;
    while j < N2 {
        out[i] = b[j];
        i += 1;
        j += 1;
    }
    out
}

/// Synthesis bounds.
pub trait SynBound<const N: usize>: Clone + Sync + Send {
    /// Lower & upper bounds
    const BOUND: [[f64; 2]; N];
    /// Lower & upper bounds for partial synthesis
    const BOUND_PARTIAL: &'static [[f64; 2]];
}

impl SynBound<5> for NormFourBar {
    const BOUND: [[f64; 2]; 5] = {
        const K: f64 = 6.;
        concat_slices([[1. / K, K]; 4], [[0., TAU]; 1])
    };
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);
}

impl SynBound<6> for MNormFourBar {
    const BOUND: [[f64; 2]; 6] = {
        const K: f64 = 6.;
        concat_slices([[1. / K, K]; 4], [[0., TAU]; 2])
    };
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);
}

impl SynBound<6> for SNormFourBar {
    const BOUND: [[f64; 2]; 6] = concat_slices([[1e-4, PI]; 5], [[0., PI]; 1]);
    const BOUND_PARTIAL: &'static [[f64; 2]] = &concat_slices(Self::BOUND, [[0., TAU]; 2]);
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
            M: SynBound<N>,
            efd::Rot<D>: Sync + Send,
            efd::Coord<D>: Sync + Send,
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
