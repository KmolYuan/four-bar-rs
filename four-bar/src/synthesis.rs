//! The synthesis implementation of planar four-bar linkage mechanisms.
//!
//! ```
//! use four_bar::synthesis::synthesis;
//!
//! # let curve = [[0., 0.], [1., 0.]];
//! # let gen = 0;
//! # let pop = 2;
//! let s = synthesis(&curve, gen, pop, |_| true);
//! let result = s.result();
//! ```
use crate::{FourBar, Mechanism};
use efd::{Efd, GeoInfo};
use metaheuristics_nature::*;
use rayon::prelude::*;
use std::f64::consts::TAU;

fn guide(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let end = curve.len() - 1;
    let mut curve = Vec::from(curve);
    if (curve[0][0] - curve[end][0]).abs() > 1e-20 || (curve[0][1] - curve[end][1]).abs() > 1e-20 {
        curve.push(curve[0]);
    }
    curve
}

fn path_is_nan(path: &[[f64; 2]]) -> bool {
    for c in path {
        if c[0].is_nan() || c[0].is_nan() {
            return true;
        }
    }
    false
}

fn grashof_transform(v: &[f64]) -> Vec<f64> {
    let mut four = vec![v[0], 1., v[1], v[2]];
    four.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    if four[0] + four[3] - four[1] - four[2] >= 0. {
        let l1 = four[0];
        vec![four[1] / l1, four[3] / l1, four[2] / l1, v[3] / l1, v[4]]
    } else {
        v.to_vec()
    }
}

fn four_bar_v(v: &[f64], inv: bool) -> FourBar {
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

fn geo_err(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    assert!(curve.len() >= target.len());
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
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize) -> Self {
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        // Close loop
        let curve = guide(curve);
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
        }
    }

    fn four_bar_coeff(&self, v: &[f64], inv: bool, geo: GeoInfo) -> FourBar {
        let a = geo.semi_major_axis_angle - self.geo.semi_major_axis_angle;
        let scale = self.geo.scale / geo.scale;
        let locus_a = geo.locus.1.atan2(geo.locus.0) + a;
        let d = geo.locus.1.hypot(geo.locus.0) * scale;
        FourBar {
            p0: (
                self.geo.locus.0 - d * locus_a.cos(),
                self.geo.locus.1 - d * locus_a.sin(),
            ),
            a,
            l0: v[0] * scale,
            l1: scale,
            l2: v[1] * scale,
            l3: v[2] * scale,
            l4: v[3] * scale,
            g: v[4],
            inv,
        }
    }

    fn available_curve(&self, v: &[f64]) -> Vec<(bool, Vec<[f64; 2]>)> {
        [false, true]
            .into_par_iter()
            .map(|inv| {
                let fourbar = Mechanism::four_bar(four_bar_v(v, inv));
                let mut c = fourbar.par_four_bar_loop(0., self.n);
                c.push(c[0]);
                (inv, c)
            })
            .filter(|(_, curve)| !path_is_nan(curve))
            .collect()
    }

    fn efd_cal(&self, v: &[f64], inv: bool, curve: &[[f64; 2]]) -> (f64, FourBar) {
        let mut efd = Efd::from_curve(curve, Some(self.harmonic));
        let geo = efd.normalize();
        let four_bar = self.four_bar_coeff(v, inv, geo);
        let curve = Mechanism::four_bar(four_bar.clone()).par_four_bar_loop(0., self.n * 2);
        let geo_err = geo_err(&self.curve, &curve);
        let fitness = (efd.c - &self.efd.c).mapv(f64::abs).sum() + geo_err * 1e-5;
        (fitness, four_bar)
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;
    type Respond = f64;

    fn fitness(&self, v: &[f64], _r: &Report) -> Self::Respond {
        let v = grashof_transform(v);
        let curves = self.available_curve(&v);
        if curves.is_empty() {
            return 1e10;
        }
        curves
            .into_par_iter()
            .map(|(inv, curve)| self.efd_cal(&v, inv, &curve).0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        let v = grashof_transform(v);
        let curves = self.available_curve(&v);
        if curves.is_empty() {
            eprintln!("WARNING: synthesis failed");
            return four_bar_v(&v, false);
        }
        let (_, four_bar) = curves
            .into_par_iter()
            .map(|(inv, curve)| {
                let (fitness, four_bar) = self.efd_cal(&v, inv, &curve);
                (fitness, four_bar)
            })
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
            .unwrap();
        four_bar
    }

    fn ub(&self) -> &[f64] {
        &self.ub
    }

    fn lb(&self) -> &[f64] {
        &self.lb
    }
}

/// Dimensional synthesis with default options.
pub fn synthesis(
    curve: &[[f64; 2]],
    gen: u64,
    pop: usize,
    mut callback: impl FnMut(&Report) -> bool,
) -> Solver<Planar> {
    let planar = Planar::new(curve, 720, 360);
    Solver::build(De::default())
        .task(Task::MaxGen(gen))
        .pop_num(pop)
        .average(true)
        .callback(&mut callback)
        .solve(planar)
}
