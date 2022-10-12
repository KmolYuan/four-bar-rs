//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{efd, mh, syn};
//!
//! # let curve = vec![[0., 0.], [1., 0.], [2., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! # let n = 3;
//! let func = syn::PathSyn::from_curve_gate(curve, None, syn::Mode::Close)
//!     .expect("invalid curve")
//!     .resolution(n);
//! let s = mh::Solver::build(mh::Rga::default())
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .record(|ctx| ctx.best_f)
//!     .solve(func)
//!     .unwrap();
//! ```
use crate::{curve, efd, mh, FourBar, NormFourBar};
use std::f64::consts::{FRAC_PI_8, TAU};

type CowCurve<'a> = std::borrow::Cow<'a, [[f64; 2]]>;

// π/16
const MIN_ANGLE: f64 = FRAC_PI_8 * 0.5;
/// Boundary of the objective variables.
pub const BOUND: [[f64; 2]; 7] = [
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
    pub const fn is_result_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Return true if the synthesis curve is close.
    pub const fn is_result_close(&self) -> bool {
        !self.is_result_open()
    }

    /// Regularize path with the mode.
    pub fn regularize<'a, C>(&self, curve: C) -> Vec<[f64; 2]>
    where
        C: Into<CowCurve<'a>>,
    {
        let curve = curve.into();
        match self {
            _ if curve::is_closed(&curve) => curve.into_owned(),
            Self::Close => curve::close_line(curve),
            Self::Partial | Self::Open => curve::close_rev(curve),
        }
    }
}

/// Path generation task of planar four-bar linkage.
pub struct PathSyn {
    /// Target coefficient
    pub efd: efd::Efd2,
    mode: Mode,
    // How many points need to be generated or compared
    n: usize,
}

impl PathSyn {
    /// Create a new task from target curve and harmonic number.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    ///
    /// Please do not change mode since the regularization is called in this
    /// constructor method.
    pub fn from_curve_harmonic<'a, C>(curve: C, harmonic: usize, mode: Mode) -> Option<Self>
    where
        C: Into<CowCurve<'a>>,
    {
        efd::Efd2::from_curve_harmonic(mode.regularize(curve), harmonic)
            .map(|efd| Self::from_efd(efd, mode))
    }

    /// Create a new task from target curve and Fourier power gate.
    ///
    /// Return none if the curve length is less than 1.
    ///
    /// Please do not change mode since the regularization is called in this
    /// constructor method.
    pub fn from_curve_gate<'a, C, T>(curve: C, threshold: T, mode: Mode) -> Option<Self>
    where
        C: Into<CowCurve<'a>>,
        T: Into<Option<f64>>,
    {
        efd::Efd2::from_curve_gate(mode.regularize(curve), threshold)
            .map(|efd| Self::from_efd(efd, mode))
    }

    /// Create a new task from target EFD coefficients.
    ///
    /// Please use threshold or harmonic to create the EFD object. The curve
    /// must preprocess with [`Mode::regularize()`] method before turned into
    /// EFD.
    pub fn from_efd(efd: efd::Efd2, mode: Mode) -> Self {
        Self { efd, mode, n: 720 }
    }

    /// Set the resolution during synthesis.
    pub fn resolution(self, n: usize) -> Self {
        assert!(n > 0);
        Self { n, ..self }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }
}

impl mh::Bounded for PathSyn {
    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if matches!(self.mode, Mode::Partial) {
            &BOUND
        } else {
            &BOUND[..5]
        }
    }
}

impl mh::ObjFactory for PathSyn {
    type Product = (f64, FourBar);
    type Eval = f64;

    fn produce(&self, xs: &[f64]) -> Self::Product {
        // Only parallelize here!!
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        const INFEASIBLE: (f64, FourBar) = (1e10, FourBar::ZERO);
        let fb = NormFourBar::try_from(&xs[..5]).unwrap();
        let fb = match self.mode {
            Mode::Close | Mode::Partial => fb.to_close_curve(),
            Mode::Open => fb.to_open_curve(),
        };
        if self.mode.is_result_open() != fb.ty().is_open_curve() {
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
                let curve = fb.curve_in(t1, t2, self.n);
                (curve, fb)
            })
            .filter(|(curve, _)| curve.len() > 1)
            .map(|(curve, fb)| {
                let curve = self.mode.regularize(curve);
                let efd = efd::Efd2::from_curve_harmonic(curve, self.efd.harmonic()).unwrap();
                let fb = FourBar::from_trans(fb, &efd.to(&self.efd));
                (efd.l1_norm(&self.efd), fb)
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

    fn evaluate(&self, (f, _): Self::Product) -> Self::Eval {
        f
    }
}
