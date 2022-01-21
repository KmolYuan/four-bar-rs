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
//!     .solve(Planar::new(&curve, 720, 90, false));
//! let result = s.result();
//! ```
use self::mh::{utility::prelude::*, ObjFunc};
use crate::{FourBar, Mechanism};
use efd::{Efd, GeoInfo};
use std::f64::consts::{FRAC_PI_4, TAU};

#[doc(no_inline)]
pub use metaheuristics_nature as mh;

type CurveIter<'a> = Box<dyn Iterator<Item = &'a [f64; 2]> + Send + Sync + 'a>;

/// Input a curve, split out finite parts to a continuous curve. (greedy method)
///
/// The result is close to the first-found finite item,
/// and the part of infinity and NaN will be dropped.
pub fn get_valid_part(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let is_invalid = |[x, y]: &[f64; 2]| !x.is_finite() || !y.is_finite();
    let is_valid = |[x, y]: &[f64; 2]| x.is_finite() && y.is_finite();
    let mut iter = curve.iter();
    match iter.position(is_valid) {
        None => Vec::new(),
        Some(t1) => match iter.position(is_invalid) {
            None => curve[t1..].to_vec(),
            Some(t2) => {
                let s1 = curve[t1..t1 + t2].to_vec();
                let mut iter = curve.iter().rev();
                match iter.position(is_valid) {
                    Some(t1) if t1 == 0 => {
                        let t1 = curve.len() - 1 - t1;
                        let t2 = t1 - iter.position(is_invalid).unwrap();
                        [&curve[t2..t1], &s1].concat()
                    }
                    _ => s1,
                }
            }
        },
    }
}

/// Anti-symmetric extension function.
pub fn anti_sym_ext(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = curve.len() - 1;
    let [x0, y0] = curve[0];
    let [xn, yn] = curve[n];
    let xd = xn - x0;
    let yd = yn - y0;
    let n = n as f64;
    let mut v1 = curve
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| {
            let i_n = i as f64 / n;
            [x - x0 - xd * i_n, y - y0 - yd * i_n]
        })
        .collect::<Vec<_>>();
    let mut v2 = v1
        .iter()
        .take(curve.len() - 1)
        .skip(1)
        .map(|[x, y]| [-x, -y])
        .rev()
        .collect();
    v1.append(&mut v2);
    v1
}

/// Close the open curve directly.
pub fn close_loop(mut curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    curve.push(curve[0]);
    curve
}

/// Return true if curve contains any NaN coordinate.
pub fn is_valid_curve(curve: &[[f64; 2]]) -> bool {
    curve.iter().any(|[x, y]| !x.is_finite() || !y.is_finite())
}

/// Geometry error between two open curves.
pub fn geo_err_opened(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    debug_assert!(target.len() < curve.len());
    let iters: [(_, CurveIter); 2] = [
        (curve[0], Box::new(curve.iter())),
        (curve[curve.len() - 1], Box::new(curve.iter().rev())),
    ];
    iters
        .into_par_iter()
        .map(|(start, iter)| geo_err(target, &start, iter))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

/// Geometry error between two closed curves.
pub fn geo_err_closed(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    let end = curve.len();
    debug_assert!(target.len() < end);
    // Find the head (correlation)
    let (index, basic_err) = curve
        .par_iter()
        .enumerate()
        .map(|(i, [x, y])| (i, (target[0][0] - x).hypot(target[0][1] - y)))
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let iter = curve.iter().cycle().skip(index).take(end);
    let start = &curve[index];
    let rev = curve.iter().rev().cycle().skip(end - index).take(end);
    let iters: [CurveIter; 2] = [Box::new(iter), Box::new(rev)];
    let err = iters
        .into_par_iter()
        .map(|iter| geo_err(&target[1..], start, iter))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    basic_err + err
}

fn geo_err<'a, I>(target: &[[f64; 2]], start: &[f64; 2], mut iter: I) -> f64
where
    I: Iterator<Item = &'a [f64; 2]>,
{
    let mut geo_err = 0.;
    let mut left = start;
    for tc in target {
        let mut last_d = (tc[0] - left[0]).hypot(tc[1] - left[1]);
        for c in &mut iter {
            let d = (tc[0] - c[0]).hypot(tc[1] - c[1]);
            if d < last_d {
                last_d = d;
            } else {
                left = c;
                break;
            }
        }
        geo_err += last_d;
    }
    geo_err / target.len() as f64
}

fn grashof_transform(v: &[f64]) -> [f64; 5] {
    let mut four = [v[0], 1., v[1], v[2]];
    four.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    if four[0] + four[3] - four[1] - four[2] < 0. && (four[0] == 1. || four[0] == v[0]) {
        [v[0], v[1], v[2], v[3], v[4]]
    } else {
        let l1 = four[0];
        [four[1] / l1, four[3] / l1, four[2] / l1, v[3] / l1, v[4]]
    }
}

fn four_bar_v(v: &[f64; 5], inv: bool) -> FourBar {
    FourBar {
        p0: [0., 0.],
        a: 0.,
        l0: v[0],
        l1: 1.,
        l2: v[1],
        l3: v[2],
        l4: v[3],
        g: v[4],
        inv,
    }
}

fn four_bar_coeff(d: &[f64; 5], inv: bool, geo: GeoInfo) -> FourBar {
    FourBar {
        p0: geo.center,
        a: geo.rot,
        l0: d[0] * geo.scale,
        l1: geo.scale,
        l2: d[1] * geo.scale,
        l3: d[2] * geo.scale,
        l4: d[3] * geo.scale,
        g: d[4],
        inv,
    }
}

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    /// Target curve
    pub curve: Vec<[f64; 2]>,
    /// Target coefficient
    pub efd: Efd,
    // How many points need to be generated / compared
    n: usize,
    harmonic: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
    open: bool,
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize, open: bool) -> Self {
        let curve = close_loop(get_valid_part(curve));
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
        let efd = Efd::from_curve(&curve, Some(harmonic));
        Self {
            curve,
            efd,
            n,
            harmonic,
            ub,
            lb,
            open,
        }
    }

    /// Check if the target is defined as  open curve.
    pub fn is_open(&self) -> bool {
        self.open
    }

    fn domain_search(&self, v: &[f64]) -> (f64, FourBar) {
        let d = grashof_transform(v);
        let f = |[t1, t2]: [f64; 2]| {
            [false, true]
                .into_par_iter()
                .map(move |inv| {
                    let m = Mechanism::four_bar(&four_bar_v(&d, inv));
                    (close_loop(m.par_four_bar_loop(t1, t2, self.n)), inv)
                })
                .filter(|(curve, _)| !is_valid_curve(curve))
                .map(|(curve, inv)| {
                    let efd = Efd::from_curve(&curve, Some(self.harmonic));
                    let four_bar = four_bar_coeff(&d, inv, efd.to(&self.efd));
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