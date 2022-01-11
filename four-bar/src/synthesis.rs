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
use std::f64::consts::TAU;

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
    let mut geo_err = f64::INFINITY;
    let mut index = 0;
    // Find the head
    for (i, c) in curve.iter().enumerate() {
        let d = (target[0][0] - c[0]).powi(2) + (target[0][1] - c[1]).powi(2);
        if d < geo_err {
            geo_err = d;
            index = i;
        }
    }
    let mut iter = curve[index..].iter().chain(curve[0..index].iter().rev());
    let start = iter.next().unwrap();
    let rev_iter = iter.clone().rev();
    let iter: [Box<dyn Iterator<Item = &[f64; 2]> + Send + Sync>; 2] =
        [Box::new(iter), Box::new(rev_iter)];
    iter.into_par_iter()
        .map(|mut iter| {
            let mut geo_err = geo_err;
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
    // Geometric information
    geo: GeoInfo,
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
        let mut curve = Vec::from(curve);
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        if open {
            for _ in 0..2 {
                ub.push(1.);
                lb.push(0.);
            }
            curve = anti_sym_ext(&curve);
        }
        curve.push(curve[0]);
        let mut efd = Efd::from_curve(&curve, Some(harmonic));
        let geo = efd.normalize();
        Self {
            curve,
            efd,
            geo,
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
            a: geo.semi_major_axis_angle,
            l0: d[0] * geo.scale,
            l1: geo.scale,
            l2: d[1] * geo.scale,
            l3: d[2] * geo.scale,
            l4: d[3] * geo.scale,
            g: d[4],
            inv,
        }
    }

    fn available_curve<'a>(
        &'a self,
        d: &'a [f64; 5],
    ) -> impl ParallelIterator<Item = (bool, Vec<[f64; 2]>)> + 'a {
        [false, true]
            .into_par_iter()
            .map(|inv| {
                let fourbar = Mechanism::four_bar(&four_bar_v(d, inv));
                let c = fourbar.par_four_bar_loop(0., self.n);
                (inv, c)
            })
            .filter(|(_, curve)| !path_is_nan(curve))
    }

    fn efd_cal(
        &self,
        v: &[f64],
        d: &[f64; 5],
        inv: bool,
        mut curve: Vec<[f64; 2]>,
    ) -> (f64, FourBar) {
        if self.open {
            let [_t1, _t2] = [v[5], v[6]].map(|v| (v * self.n as f64) as usize);
            todo!()
        } else {
            curve.push(curve[0]);
            let mut efd = Efd::from_curve(&curve, Some(self.harmonic));
            let four_bar = self.four_bar_coeff(d, inv, efd.normalize().to(&self.geo));
            let curve = Mechanism::four_bar(&four_bar).par_four_bar_loop(0., self.n);
            let geo_err = geo_err_closed(&self.curve, &curve);
            let fitness = efd.discrepancy(&self.efd) + geo_err * 1e-5;
            (fitness, four_bar)
        }
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;
    type Fitness = f64;

    fn fitness(&self, v: &[f64], _: f64) -> Self::Fitness {
        let d = grashof_transform(v);
        self.available_curve(&d)
            .map(|(inv, curve)| self.efd_cal(v, &d, inv, curve).0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1e10)
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        let d = grashof_transform(v);
        self.available_curve(&d)
            .map(|(inv, curve)| self.efd_cal(v, &d, inv, curve))
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
            .unwrap_or_else(|| {
                eprintln!("WARNING: synthesis failed");
                (0., four_bar_v(&d, false))
            })
            .1
    }

    fn ub(&self) -> &[f64] {
        &self.ub
    }

    fn lb(&self) -> &[f64] {
        &self.lb
    }
}
