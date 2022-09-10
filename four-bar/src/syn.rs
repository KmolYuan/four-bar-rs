//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{mh, syn};
//!
//! # let curve = [[0., 0.], [1., 0.], [2., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! # let n = 3;
//! let s = mh::Solver::build(mh::Rga::default())
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .record(|ctx| ctx.best_f)
//!     .solve(syn::PathSyn::from_curve(&curve, None, n, syn::Mode::Close))
//!     .unwrap();
//! ```
use crate::{curve, efd::Efd2, mh::ObjFunc, FourBar, Mechanism, NormFourBar};
use std::f64::consts::{FRAC_PI_8, TAU};

// π/16
const MIN_ANGLE: f64 = FRAC_PI_8 * 0.5;
const BOUND: [[f64; 2]; 7] = [
    [1e-4, 10.],
    [1e-4, 10.],
    [1e-4, 10.],
    [1e-4, 10.],
    [0., TAU],
    [0., TAU],
    [0., TAU],
];

/// Synthesis mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    /// Close path matching
    Close,
    /// Use close path to match open path
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
        matches!(self, Self::Close)
    }

    /// Return true if the synthesis curve is open.
    pub const fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Return true if the synthesis curve is close.
    pub const fn is_close(&self) -> bool {
        !self.is_open()
    }

    /// Compare with an open/close curve boolean flag.
    pub const fn eq_bool(&self, is_open: bool) -> bool {
        matches!((self, is_open), (Self::Open, true) | (Self::Close, false))
    }

    /// Regularize path with the mode.
    pub fn regularize(&self, curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
        match self {
            _ if curve::is_closed(&curve) => curve,
            Self::Close => curve::close_line(curve),
            Self::Partial | Self::Open => curve::close_rev(curve),
        }
    }
}

/// Path generation task of planar four-bar linkage.
pub struct PathSyn {
    /// Target coefficient
    pub efd: Efd2,
    // How many points need to be generated / compared
    n: usize,
    mode: Mode,
}

impl PathSyn {
    /// Create a new task.
    ///
    /// Panic if target curve is not long enough,
    /// or `n` is not longer than target curve.
    pub fn from_curve<H>(curve: &[[f64; 2]], harmonic: H, n: usize, mode: Mode) -> Self
    where
        H: Into<Option<usize>>,
    {
        let curve = curve::get_valid_part(curve);
        assert!(curve.len() > 2, "target curve is not long enough");
        assert!(n > curve.len() - 1, "n must longer than target curve");
        let efd = Efd2::from_curve(&mode.regularize(curve), harmonic);
        Self::from_efd(efd, n, mode)
    }

    /// Create a new task using a four-bar linkage as the target curve.
    pub fn from_four_bar<F, H>(fb: F, harmonic: H, n: usize, mode: Mode) -> Option<Self>
    where
        F: Into<FourBar>,
        H: Into<Option<usize>>,
    {
        curve::from_four_bar(fb, n).map(|curve| Self::from_curve(&curve, harmonic, n, mode))
    }

    /// Create a new task from target EFD coefficients.
    pub fn from_efd(efd: Efd2, n: usize, mode: Mode) -> Self {
        Self { efd, n, mode }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }

    fn domain_search(&self, xs: &[f64]) -> (f64, FourBar) {
        // Only parallelize here!!
        #[cfg(feature = "rayon")]
        use crate::mh::rayon::prelude::*;
        const INFEASIBLE: (f64, FourBar) = (1e10, FourBar::ZERO);
        let fb = NormFourBar::try_from(&xs[..5]).unwrap();
        let fb = match self.mode {
            Mode::Close | Mode::Partial => fb.to_close_curve(),
            Mode::Open => fb.to_open_curve(),
        };
        if self.mode.is_open() != fb.ty().is_open_curve() {
            return INFEASIBLE;
        }
        let f = |[t1, t2]: [f64; 2]| {
            let fb = &fb;
            #[cfg(feature = "rayon")]
            let iter = [false, true].into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = [false, true].into_iter();
            iter.map(move |inv| {
                let fb = fb.with_inv(inv);
                let curve = Mechanism::new(&fb).curve(t1, t2, self.n);
                (curve::get_valid_part(curve), fb)
            })
            .filter(|(curve, _)| curve.len() > 2)
            .map(|(curve, fb)| {
                let efd = Efd2::from_curve(&self.mode.regularize(curve), Some(self.efd.harmonic()));
                let fitness = efd.manhattan(&self.efd);
                let fb = FourBar::from_transform(fb, efd.to(&self.efd));
                (fitness, fb)
            })
        };
        match self.mode {
            Mode::Close | Mode::Open => fb
                .angle_bound()
                .filter(|[t1, t2]| t2 - t1 > MIN_ANGLE)
                .and_then(|t| f(t).min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap()))
                .unwrap_or(INFEASIBLE),
            Mode::Partial => {
                #[cfg(feature = "rayon")]
                let iter = [[xs[5], xs[6]], [xs[6], xs[5]]].into_par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = [[xs[5], xs[6]], [xs[6], xs[5]]].into_iter();
                iter.map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                    .filter(|[t1, t2]| t2 - t1 > MIN_ANGLE)
                    .flat_map(f)
                    .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                    .unwrap_or(INFEASIBLE)
            }
        }
    }
}

impl ObjFunc for PathSyn {
    type Result = FourBar;
    type Fitness = f64;

    fn fitness(&self, xs: &[f64], _: f64) -> Self::Fitness {
        self.domain_search(xs).0
    }

    fn result(&self, xs: &[f64]) -> Self::Result {
        self.domain_search(xs).1
    }

    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if matches!(self.mode, Mode::Partial) {
            &BOUND
        } else {
            &BOUND[..5]
        }
    }
}
