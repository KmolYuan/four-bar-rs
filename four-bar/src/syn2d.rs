//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::*;
//!
//! # let curve = vec![[0., 0.], [1., 0.], [2., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! # let res = 3;
//! let func = planar_syn::PlanarSyn::from_curve(curve, planar_syn::Mode::Closed)
//!     .expect("invalid curve")
//!     .res(res);
//! let s = mh::Solver::build(mh::Rga::default(), func)
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .solve()
//!     .unwrap();
//! ```
use crate::{curve, efd, mh, FourBar, NormFourBar};
use std::{
    borrow::Cow,
    f64::consts::{FRAC_PI_8, TAU},
};

type CowCurve<'a> = std::borrow::Cow<'a, [[f64; 2]]>;

/// The minimum input angle bound. (π/16)
pub const MIN_ANGLE: f64 = FRAC_PI_8 * 0.5;
const BOUND_F: f64 = 6.;
/// Boundary of the objective variables.
pub const BOUND: [[f64; 2]; 7] = [
    [1. / BOUND_F, BOUND_F],
    [1. / BOUND_F, BOUND_F],
    [1. / BOUND_F, BOUND_F],
    [1. / BOUND_F, BOUND_F],
    [0., TAU],
    [0., TAU],
    [0., TAU],
];

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
    pub fn regularize<'a, A, C>(&self, curve: C) -> Vec<A>
    where
        A: PartialEq + Clone + 'a,
        C: Into<Cow<'a, [A]>>,
    {
        let curve = curve.into();
        match self {
            _ if curve::is_closed(&curve) => curve.into_owned(),
            Self::Closed => curve::closed_lin(curve),
            Self::Partial | Self::Open => curve::closed_rev(curve),
        }
    }
}

/// Path generation task of planar four-bar linkage.
pub struct PlanarSyn {
    /// Target coefficient
    pub efd: efd::Efd2,
    mode: Mode,
    // How many points need to be generated or compared
    res: usize,
}

impl PlanarSyn {
    /// Create a new task from target curve. The harmonic number is selected
    /// automatically.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    pub fn from_curve<'a, C>(curve: C, mode: Mode) -> Option<Self>
    where
        C: Into<CowCurve<'a>>,
    {
        efd::Efd2::from_curve(mode.regularize(curve)).map(|efd| Self::from_efd(efd, mode))
    }

    /// Create a new task from target curve and harmonic number.
    ///
    /// Return none if harmonic is zero or the curve is less than 1.
    pub fn from_curve_harmonic<'a, C, H>(curve: C, harmonic: H, mode: Mode) -> Option<Self>
    where
        C: Into<CowCurve<'a>>,
        H: Into<Option<usize>>,
    {
        efd::Efd2::from_curve_harmonic(mode.regularize(curve), harmonic)
            .map(|efd| Self::from_efd(efd, mode))
    }

    /// Create a new task from target curve and Fourier power gate.
    ///
    /// Return none if the curve length is less than 1.
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
        Self { efd, mode, res: 720 }
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

impl mh::Bounded for PlanarSyn {
    #[inline]
    fn bound(&self) -> &[[f64; 2]] {
        if matches!(self.mode, Mode::Partial) {
            &BOUND
        } else {
            &BOUND[..5]
        }
    }
}

impl mh::ObjFactory for PlanarSyn {
    type Product = (f64, FourBar);
    type Eval = f64;

    fn produce(&self, xs: &[f64]) -> Self::Product {
        // Only parallelize here!!
        #[cfg(feature = "rayon")]
        use mh::rayon::prelude::*;
        const INFEASIBLE: (f64, FourBar) = (1e10, FourBar::ZERO);
        let fb = NormFourBar::try_from(&xs[..5]).unwrap();
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
                let fb = fb.clone().with_inv(inv);
                let curve = fb.curve_in(t1, t2, self.res);
                (curve, fb)
            })
            .filter(|(c, _)| c.len() > 1)
            .map(|(curve, fb)| {
                let curve = self.mode.regularize(curve);
                let efd = efd::Efd2::from_curve_harmonic(curve, self.efd.harmonic()).unwrap();
                let fb = FourBar::from_norm_trans(fb, &efd.as_trans().to(self.efd.as_trans()));
                (efd.l1_norm(&self.efd), fb)
            })
        };
        match self.mode {
            Mode::Closed | Mode::Open => fb
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
