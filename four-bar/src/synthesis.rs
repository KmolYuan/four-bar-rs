//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::{
//!     mh::{Rga, Solver},
//!     synthesis::Planar,
//! };
//!
//! # let curve = [[0., 0.], [1., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! let s = Solver::build(Rga::default())
//!     .task(|ctx| ctx.gen == gen)
//!     .pop_num(pop)
//!     .record(|ctx| ctx.best_f)
//!     .solve(Planar::new(&curve, 720, None, false));
//! let result = s.result();
//! ```
use crate::{curve, efd::Efd, mh::ObjFunc, repr, FourBar, Mechanism};
use std::f64::consts::{FRAC_PI_4, TAU};

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    /// Target curve
    pub curve: Vec<[f64; 2]>,
    /// Target coefficient
    pub efd: Efd,
    // How many points need to be generated / compared
    n: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
    open: bool,
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: Option<usize>, open: bool) -> Self {
        let curve = curve::close_loop(curve::get_valid_part(curve));
        assert!(curve.len() > 2, "target curve is not long enough");
        assert!(n > curve.len() - 1, "n must longer than target curve");
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        if open {
            ub.extend_from_slice(&[TAU; 2]);
            lb.extend_from_slice(&[0.; 2]);
        }
        let efd = Efd::from_curve(&curve, harmonic);
        Self {
            curve,
            efd,
            n,
            ub,
            lb,
            open,
        }
    }

    /// Check if the target is defined as  open curve.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// The harmonic used of target EFD.
    pub fn harmonic(&self) -> usize {
        self.efd.harmonic()
    }

    fn domain_search(&self, xs: &[f64]) -> (f64, FourBar) {
        use crate::mh::rayon::prelude::*;
        let v = repr::grashof_transform(xs);
        let f = |[t1, t2]: [f64; 2]| {
            [false, true]
                .into_par_iter()
                .map(move |inv| {
                    let m = Mechanism::new(&repr::four_bar_v(&v, inv));
                    (curve::close_loop(m.par_curve(t1, t2, self.n)), inv)
                })
                .filter(|(curve, _)| curve::is_valid(curve))
                .map(|(curve, inv)| {
                    let efd = Efd::from_curve(&curve, Some(self.efd.harmonic()));
                    let four_bar = repr::four_bar_transform(&v, inv, efd.to(&self.efd));
                    let fitness = efd.discrepancy(&self.efd);
                    (fitness, four_bar)
                })
        };
        let default = || (1e10, FourBar::default());
        if self.open {
            [[xs[5], xs[6]], [xs[6], xs[5]]]
                .into_par_iter()
                .map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                .filter(|[t1, t2]| t2 - t1 > FRAC_PI_4)
                .flat_map(f)
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                .unwrap_or_else(default)
        } else {
            f([0., TAU])
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                .unwrap_or_else(default)
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
