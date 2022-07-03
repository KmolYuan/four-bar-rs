//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{
//!     mh::{Rga, Solver},
//!     syn::{Mode, Planar},
//! };
//!
//! # let curve = [[0., 0.], [1., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! let s = Solver::build(Rga::default())
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .record(|ctx| ctx.best_f)
//!     .solve(Planar::new(&curve, 720, None, Mode::Close));
//! let result = s.result();
//! ```
use crate::{curve, efd::Efd, mh::ObjFunc, FourBar, Mechanism, NormFourBar};
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

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    /// Target curve
    pub curve: Vec<[f64; 2]>,
    /// Target coefficient
    pub efd: Efd<f64>,
    // How many points need to be generated / compared
    n: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
    mode: Mode,
}

impl Planar {
    /// Create a new task.
    pub fn new<H>(curve: &[[f64; 2]], n: usize, harmonic: H, mode: Mode) -> Self
    where
        H: Into<Option<usize>>,
    {
        let curve = curve::close_line(curve::get_valid_part(curve));
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
        let efd = Efd::from_curve(&curve, harmonic);
        Self { curve, efd, n, ub, lb, mode }
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }

    fn domain_search(&self, xs: &[f64]) -> (f64, FourBar) {
        use crate::mh::rayon::prelude::*;
        let v = match self.mode {
            Mode::Close | Mode::Partial => NormFourBar::cr_transform(xs),
            Mode::Open => NormFourBar::dr_transform(xs),
        };
        let f = |[t1, t2]: [f64; 2]| {
            [false, true]
                .into_par_iter()
                .map(move |inv| {
                    let m = Mechanism::new(&NormFourBar::from_vec(v, inv));
                    (curve::close_line(m.par_curve(t1, t2, self.n)), inv)
                })
                .filter(|(curve, _)| curve::is_valid(curve))
                .map(|(curve, inv)| {
                    let efd = Efd::from_curve(&curve, Some(self.efd.harmonic()));
                    let four_bar = FourBar::from_transform(v, inv, efd.to(&self.efd));
                    let fitness = efd.manhattan(&self.efd);
                    (fitness, four_bar)
                })
        };
        match self.mode {
            Mode::Close => f([0., TAU])
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                .unwrap_or((1e10, FourBar::ZERO)),
            Mode::Partial => [[xs[5], xs[6]], [xs[6], xs[5]]]
                .into_par_iter()
                .map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                .filter(|[t1, t2]| t2 - t1 > FRAC_PI_4)
                .flat_map(f)
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                .unwrap_or((1e10, FourBar::ZERO)),
            Mode::Open => todo!(),
        }
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;
    type Fitness = f64;

    fn fitness(&self, xs: &[f64], _: f64) -> Self::Fitness {
        self.domain_search(xs).0
    }

    fn result(&self, xs: &[f64]) -> Self::Result {
        self.domain_search(xs).1
    }

    fn ub(&self) -> &[f64] {
        &self.ub
    }

    fn lb(&self) -> &[f64] {
        &self.lb
    }
}
