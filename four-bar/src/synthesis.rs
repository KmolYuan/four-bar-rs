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
use std::{cmp::Ordering, f64::consts::TAU};

#[doc(no_inline)]
pub use metaheuristics_nature as mh;

/// Input a curve, split out none-NaN parts to a continuous curve.
///
/// The part is close to the first-found none-NaN item.
pub fn open_curve(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let is_nan = |c: &[f64; 2]| c[0].is_nan() || c[1].is_nan();
    let is_not_nan = |c: &[f64; 2]| !c[0].is_nan() && !c[1].is_nan();
    let mut iter = curve.iter();
    match iter.position(is_not_nan) {
        None => Vec::new(),
        Some(t1) => match iter.position(is_nan) {
            None => curve[t1..].to_vec(),
            Some(t2) => {
                let s1 = curve[t1..t1 + t2].to_vec();
                let mut iter = curve.iter().rev();
                match iter.position(is_not_nan) {
                    Some(t1) if t1 == 0 => {
                        let t1 = curve.len() - 1 - t1;
                        let t2 = t1 - iter.position(is_nan).unwrap();
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
    let [x0, y0] = [curve[0][0], curve[0][1]];
    let [xn, yn] = [curve[n][0], curve[n][1]];
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
        .map(|&[x, y]| [-x, -y])
        .rev()
        .collect();
    v1.append(&mut v2);
    v1
}

/// Return true if path contains any NaN coordinate.
pub fn path_is_nan(path: &[[f64; 2]]) -> bool {
    path.iter().any(|c| c[0].is_nan() || c[0].is_nan())
}

/// Geometry error between two closed curves.
pub fn geo_err_closed(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    assert!(
        curve.len() >= target.len(),
        "curve length {} must greater than target {}",
        curve.len(),
        target.len()
    );
    // Find the head (greedy)
    let (index, basic_err) = curve
        .par_iter()
        .enumerate()
        .map(|(i, c)| {
            let dx = target[0][0] - c[0];
            let dy = target[0][1] - c[1];
            (i, dx * dx + dy * dy)
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let iter = curve[index..].iter().chain(curve[0..index].iter().rev());
    basic_err + geo_err(target, iter)
}

fn geo_err_opened(target: &[[f64; 2]], curve: &[[f64; 2]]) -> (f64, GeoInfo) {
    let _ = target;
    let _ = curve;
    let fitness = 0.;
    let geo = GeoInfo {
        rot: 0.,
        scale: 0.,
        center: (0., 0.),
    };
    (fitness, geo)
}

fn geo_err<'a, I>(target: &[[f64; 2]], mut iter: I) -> f64
where
    I: DoubleEndedIterator<Item = &'a [f64; 2]> + Clone + Send + Sync,
{
    let start = iter.next().unwrap();
    let rev_iter = iter.clone().rev();
    let iter: [Box<dyn Iterator<Item = &[f64; 2]> + Send + Sync>; 2] =
        [Box::new(iter), Box::new(rev_iter)];
    iter.into_par_iter()
        .map(|mut iter| {
            let mut geo_err = 0.;
            let mut left = start;
            for tc in target {
                let mut last_d = (tc[0] - left[0]).powi(2) + (tc[1] - left[1]).powi(2);
                for c in &mut *iter {
                    let d = (tc[0] - c[0]).powi(2) + (tc[1] - c[1]).powi(2);
                    if d < last_d {
                        last_d = d;
                    } else {
                        left = c;
                        break;
                    }
                }
                geo_err += last_d;
            }
            geo_err
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
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
        p0: (0., 0.),
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
        assert!(curve.len() > 1, "target curve is not long enough");
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        let mut curve = if open {
            for _ in 0..2 {
                ub.push(1.);
                lb.push(0.);
            }
            anti_sym_ext(curve)
        } else {
            curve.to_vec()
        };
        curve.push(curve[0]);
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

    fn four_bar_coeff(&self, d: &[f64; 5], inv: bool, geo: GeoInfo) -> FourBar {
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

    fn domain_search(&self, v: &[f64]) -> (f64, FourBar) {
        let d = grashof_transform(v);
        [false, true]
            .into_par_iter()
            .map(|inv| {
                let fourbar = Mechanism::four_bar(&four_bar_v(&d, inv));
                let c = fourbar.par_four_bar_loop(0., self.n);
                (inv, c)
            })
            .filter(|(_, curve)| !path_is_nan(curve))
            .map(|(inv, mut curve)| {
                let (efd, geo_err, four_bar) = if self.open {
                    let [t1, t2] = [v[5], v[6]].map(|v| (v * self.n as f64) as usize);
                    let (geo_err, geo) = if t1 == t2 {
                        vec![[t1, t2]]
                    } else {
                        vec![[t1, t2], [t2, t1]]
                    }
                    .into_par_iter()
                    .map(|[t1, t2]| {
                        let curve = match t1.cmp(&t2) {
                            Ordering::Less => curve[t1..t2].to_vec(),
                            Ordering::Greater | Ordering::Equal => {
                                [&curve[t1..], &curve[..t2]].concat()
                            }
                        };
                        geo_err_opened(&self.curve, &curve)
                    })
                    .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
                    .unwrap();
                    let mut curve = anti_sym_ext(&curve);
                    curve.push(curve[0]);
                    let efd = Efd::from_curve(&curve, Some(self.harmonic));
                    (efd, geo_err, self.four_bar_coeff(&d, inv, geo))
                } else {
                    curve.push(curve[0]);
                    let efd = Efd::from_curve(&curve, Some(self.harmonic));
                    let four_bar = self.four_bar_coeff(&d, inv, efd.to(&self.efd));
                    let curve = Mechanism::four_bar(&four_bar).par_four_bar_loop(0., self.n);
                    (efd, geo_err_closed(&self.curve, &curve), four_bar)
                };
                (efd.discrepancy(&self.efd) + geo_err * 1e-5, four_bar)
            })
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
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
