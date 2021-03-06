//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{
//!     mh::{Rga, Solver},
//!     syn::{Mode, PathSyn},
//! };
//!
//! # let curve = [[0., 0.], [1., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! let s = Solver::build(Rga::default())
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .record(|ctx| ctx.best_f)
//!     .solve(PathSyn::new(&curve, 720, None, Mode::Close));
//! let result = s.result();
//! ```
use crate::{curve, efd::Efd2, mh::ObjFunc, FourBar, Mechanism, NormFourBar};
use std::f64::consts::{FRAC_PI_4, TAU};

/// Synthesis mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Copy, Clone)]
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
        matches!(self, Self::Partial | Self::Open)
    }

    /// Return true if the synthesis curve is open.
    pub const fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

/// Path generation task of planar four-bar linkage.
pub struct PathSyn {
    /// Target curve
    pub curve: Vec<[f64; 2]>,
    /// Target coefficient
    pub efd: Efd2,
    // How many points need to be generated / compared
    n: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
    mode: Mode,
}

impl PathSyn {
    /// Create a new task.
    ///
    /// Panic if target curve is not long enough,
    /// or `n` is not longer than target curve.
    pub fn new<H>(curve: &[[f64; 2]], n: usize, harmonic: H, mode: Mode) -> Self
    where
        H: Into<Option<usize>>,
    {
        let curve = curve::get_valid_part(curve);
        let curve = match mode {
            Mode::Close if curve::is_closed(&curve) => curve,
            Mode::Close => curve::close_line(curve),
            Mode::Partial | Mode::Open => curve::close_rev(curve),
        };
        assert!(curve.len() > 2, "target curve is not long enough");
        assert!(n > curve.len() - 1, "n must longer than target curve");
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        if mode == Mode::Partial {
            ub.extend_from_slice(&[TAU; 2]);
            lb.extend_from_slice(&[0.; 2]);
        }
        let efd = Efd2::from_curve(&curve, harmonic);
        Self { curve, efd, n, ub, lb, mode }
    }

    /// Create a new task using a four-bar linkage as the target curve.
    pub fn from_four_bar<F, H>(fb: F, n: usize, harmonic: H, mode: Mode) -> Option<Self>
    where
        F: Into<FourBar>,
        H: Into<Option<usize>>,
    {
        let fb = fb.into();
        if let Some([t1, t2]) = fb.angle_bound() {
            let curve = Mechanism::new(&fb).curve(t1, t2, n);
            Some(Self::new(&curve, n, harmonic, mode))
        } else {
            None
        }
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
        let norm = NormFourBar::try_from(xs).unwrap();
        let norm = match self.mode {
            Mode::Close | Mode::Partial => norm.to_close_curve(),
            Mode::Open => norm.to_open_curve(),
        };
        if !norm.is_valid() {
            return INFEASIBLE;
        } else if matches!(self.mode, Mode::Close | Mode::Partial) {
            if norm.ty().is_open_curve() {
                return INFEASIBLE;
            }
        } else if norm.ty().is_close_curve() {
            // Mode::Open
            return INFEASIBLE;
        }
        let close_f = match self.mode {
            Mode::Close => curve::close_line,
            Mode::Partial | Mode::Open => curve::close_rev,
        };
        let f = |[t1, t2]: [f64; 2]| {
            let norm = &norm;
            #[cfg(feature = "rayon")]
            let iter = [false, true].into_par_iter();
            #[cfg(not(feature = "rayon"))]
            let iter = [false, true].into_iter();
            iter.map(move |inv| {
                let norm = norm.with_inv(inv);
                let curve = Mechanism::new(&norm).curve(t1, t2, self.n);
                (curve::get_valid_part(&curve), norm)
            })
            .filter(|(curve, _)| curve.len() > 2)
            .map(|(curve, norm)| {
                let efd = Efd2::from_curve(&close_f(curve), Some(self.efd.harmonic()));
                let four_bar = FourBar::from_transform(norm, efd.to(&self.efd));
                let fitness = efd.manhattan(&self.efd);
                (fitness, four_bar)
            })
        };
        match self.mode {
            Mode::Close => f([0., TAU])
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                .unwrap_or(INFEASIBLE),
            Mode::Partial => {
                #[cfg(feature = "rayon")]
                let iter = [[xs[5], xs[6]], [xs[6], xs[5]]].into_par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = [[xs[5], xs[6]], [xs[6], xs[5]]].into_iter();
                iter.map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                    .filter(|[t1, t2]| t2 - t1 > FRAC_PI_4)
                    .flat_map(f)
                    .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                    .unwrap_or(INFEASIBLE)
            }
            Mode::Open => norm
                .angle_bound()
                .and_then(|t| f(t).min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap()))
                .unwrap_or(INFEASIBLE),
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
    fn ub(&self) -> &[f64] {
        &self.ub
    }

    #[inline]
    fn lb(&self) -> &[f64] {
        &self.lb
    }
}
