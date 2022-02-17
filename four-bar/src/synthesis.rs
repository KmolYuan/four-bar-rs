//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::synthesis::{
//!     mh::{Rga, Solver},
//!     Planar,
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
use self::{
    efd::Efd,
    mh::{utility::prelude::*, ObjFunc},
};
use crate::{curve, repr, FourBar, Mechanism};
use std::f64::consts::{FRAC_PI_4, TAU};

#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub use metaheuristics_nature as mh;

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

    fn domain_search(&self, v: &[f64]) -> (f64, FourBar) {
        let d = repr::grashof_transform(v);
        let f = |[t1, t2]: [f64; 2]| {
            [false, true]
                .into_par_iter()
                .map(move |inv| {
                    let m = Mechanism::new(&repr::four_bar_v(d, inv));
                    (curve::close_loop(m.par_four_bar_loop(t1, t2, self.n)), inv)
                })
                .filter(|(curve, _)| curve::is_valid(curve))
                .map(|(curve, inv)| {
                    let efd = Efd::from_curve(&curve, Some(self.efd.harmonic()));
                    let four_bar = repr::four_bar_transform(&d, inv, efd.to(&self.efd));
                    let fitness = efd.discrepancy(&self.efd);
                    (fitness, four_bar)
                })
        };
        if self.open {
            [[v[5], v[6]], [v[6], v[5]]]
                .into_par_iter()
                .map(|[t1, t2]| [t1, if t2 > t1 { t2 } else { t2 + TAU }])
                .filter(|[t1, t2]| t2 - t1 > FRAC_PI_4)
                .flat_map(f)
                .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
        } else {
            f([0., TAU]).min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
        }
        .unwrap_or_else(|| (1e10, FourBar::default()))
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;
    type Fitness = f64;

    fn fitness(&self, v: &[f64], _: f64) -> Self::Fitness {
        self.domain_search(v).0
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        self.domain_search(v).1
    }

    fn ub(&self) -> &[f64] {
        &self.ub
    }

    fn lb(&self) -> &[f64] {
        &self.lb
    }
}
