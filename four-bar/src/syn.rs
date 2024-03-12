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
pub use self::{
    motion::{MFbSyn, MotionSyn},
    path::{FbSyn, PathSyn, SFbSyn},
    pp_motion::{MFbPPSyn, PPMotionSyn},
    pp_path::{FbPPSyn, PPPathSyn, SFbPPSyn},
};
use crate::*;

mod motion;
mod path;
mod pp_motion;
mod pp_path;

/// Base type of a mechanism `M` synthesis.
pub struct Syn<T, M, const N: usize, const D: usize> {
    /// Target data
    pub tar: T,
    /// Mode
    pub(crate) mode: Mode,
    // How many points need to be generated and compared
    pub(crate) res: usize,
    // Constrain the origin of the mechanism
    origin: Option<[f64; D]>,
    // Constrain the scale of the mechanism
    scale: Option<f64>,
    // Marker of the mechanism
    _marker: std::marker::PhantomData<M>,
}

impl<T, M, const N: usize, const D: usize> Syn<T, M, N, D> {
    pub(crate) fn new(tar: T, mode: Mode) -> Self {
        Self {
            tar,
            mode,
            res: 180,
            origin: None,
            scale: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the resolution during synthesis.
    pub fn res(self, res: usize) -> Self {
        assert!(res > 0);
        Self { res, ..self }
    }

    /// Specify the mechanism is on origin and unit scale.
    pub fn on_unit(self) -> Self {
        self.origin([0.; D]).scale(1.)
    }

    /// Specify the origin of the mechanism.
    pub fn origin(self, origin: [f64; D]) -> Self {
        Self { origin: Some(origin), ..self }
    }

    /// Specify the scale of the mechanism.
    pub fn scale(self, scale: f64) -> Self {
        assert!(scale > 0.);
        Self { scale: Some(scale), ..self }
    }

    pub(crate) fn unit_err(&self, geo: &efd::GeoVar<efd::Rot<D>, D>) -> f64
    where
        efd::U<D>: efd::EfdDim<D>,
    {
        use efd::Distance as _;
        let o_err = self.origin.map(|o| geo.trans().l2_err(&o)).unwrap_or(0.);
        let s_err = self.scale.map(|s| (geo.scale() - s).abs()).unwrap_or(0.);
        o_err.max(s_err)
    }
}

/// Synthesis bounds.
pub trait SynBound<const N: usize>: mech::Statable + mech::FromVectorized<N> + Sync + Send {}
impl<T, const N: usize> SynBound<N> for T where
    T: mech::Statable + mech::FromVectorized<N> + Sync + Send
{
}

/// Synthesis mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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

fn infeasible<P: Default>() -> mh::Product<P, f64> {
    mh::Product::new(1e2, P::default())
}

const fn slice_to_array<const N: usize>(slice: &[f64]) -> [f64; N] {
    let mut out = [0.; N];
    let mut i = 0;
    while i < N {
        out[i] = slice[i];
        i += 1;
    }
    out
}

pub(crate) fn impl_fitness<M, S, F1, F2, const N: usize, const D: usize>(
    mode: Mode,
    xs: &[f64],
    get_series: F1,
    get_err: F2,
) -> mh::Product<M::De, f64>
where
    M: SynBound<N> + mech::Normalized<D>,
    M::De: Default + Clone + Sync + Send + 'static,
    S: Send,
    F1: Fn(&M, f64, f64) -> Option<S> + Sync + Send,
    F2: Fn((S, &M)) -> mh::Product<M::De, f64> + Sync + Send,
    efd::U<D>: efd::EfdDim<D>,
{
    #[cfg(feature = "rayon")]
    use mh::rayon::prelude::*;
    let mut fb = M::from_vectorized_s1(slice_to_array(xs));
    fb.set_to_planar_loop();
    let bound = fb.angle_bound().check_mode(mode.is_result_open());
    let states = fb.states_from_bound(bound);
    let gen_series = &get_series;
    let f = |[t1, t2]: [f64; 2]| {
        #[cfg(feature = "rayon")]
        let iter = states.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = states.iter();
        iter.filter_map(move |fb| Some((gen_series(fb, t1, t2)?, fb)))
            .map(&get_err)
    };
    match mode {
        Mode::Closed | Mode::Open => bound
            .check_min()
            .to_value()
            .and_then(|t| f(t).min_by(|a, b| a.partial_cmp(b).unwrap()))
            .unwrap_or_else(infeasible),
        Mode::Partial if !bound.is_valid() => infeasible(),
        Mode::Partial => {
            let bound = mech::AngleBound::open_and_rev_at(xs[N], xs[N + 1]);
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

// Constantly assert that these types implement a certain trait
macro_rules! assert_impl {
    ($fn_name:ident, $trait_name:path, $($ty:ty),+) => {
        #[allow(unused)]
        const fn $fn_name<T: $trait_name>() {}
        $(const _: () = $fn_name::<$ty>();)+
    };
}

assert_impl!(
    assert_mh_objfunc,
    mh::ObjFunc,
    FbSyn,
    SFbSyn,
    MFbSyn,
    FbPPSyn,
    SFbPPSyn,
    MFbPPSyn
);
