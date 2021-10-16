//! The synthesis implementation of planar four-bar linkage mechanisms.
use crate::{FourBar, Mechanism};
use efd::{calculate_efd, locus, normalize_efd};
pub use metaheuristics_nature::*;
use ndarray::{arr2, concatenate, Array2, Axis};
use std::f64::consts::TAU;

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
    if four[0] + four[3] > four[1] + four[2] {
        let l1 = four[0];
        vec![four[1] / l1, four[3] / l1, four[2] / l1, v[3] / l1, v[4]]
    } else {
        v.to_vec()
    }
}

fn four_bar_from_v(v: &[f64], inv: bool) -> FourBar {
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
    let mut iter: [Box<dyn Iterator<Item = &[f64; 2]>>; 2] = [Box::new(iter), Box::new(rev_iter)];
    let mut geo_min = f64::INFINITY;
    for iter in &mut iter {
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
        if geo_err < geo_min {
            geo_min = geo_err;
        }
    }
    geo_min
}

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    curve: Vec<[f64; 2]>,
    /// Target coefficient.
    pub target: Array2<f64>,
    rot: f64,
    scale: f64,
    locus: (f64, f64),
    n: usize,
    harmonic: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize) -> Self {
        let mut curve = Vec::from(curve);
        let end = curve.len() - 1;
        // linkages
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        // Close loop
        if (curve[0][0] - curve[end][0]).abs() > 1e-20
            || (curve[0][1] - curve[end][1]).abs() > 1e-20
        {
            curve.push(curve[0]);
        }
        let curve_arr = arr2(&curve);
        let coeffs = calculate_efd(&curve_arr, harmonic);
        let (target, rot, _, scale) = normalize_efd(&coeffs, true);
        let locus = locus(&curve_arr);
        Self {
            curve,
            target,
            rot,
            scale,
            locus,
            n,
            harmonic,
            ub,
            lb,
        }
    }

    fn four_bar_from_coeff(
        &self,
        v: &[f64],
        inv: bool,
        rot: f64,
        scale: f64,
        locus: (f64, f64),
    ) -> FourBar {
        let rot = rot - self.rot;
        let scale = self.scale / scale;
        let locus_rot = locus.1.atan2(locus.0) + rot;
        let d = locus.1.hypot(locus.0) * scale;
        FourBar {
            p0: (
                self.locus.0 - d * locus_rot.cos(),
                self.locus.1 - d * locus_rot.sin(),
            ),
            a: rot,
            l0: v[0] * scale,
            l1: scale,
            l2: v[1] * scale,
            l3: v[2] * scale,
            l4: v[3] * scale,
            g: v[4],
            inv,
        }
    }

    fn available_curve(&self, v: &[f64]) -> Vec<(bool, Array2<f64>)> {
        vec![false, true]
            .into_iter()
            .map(|inv| {
                let curve = Mechanism::four_bar(four_bar_from_v(v, inv)).four_bar_loop(0., self.n);
                (inv, curve)
            })
            .filter(|(_, curve)| !path_is_nan(curve))
            .map(|(inv, curve)| (inv, arr2(&curve)))
            .collect()
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;
    type Respond = f64;

    fn fitness(&self, v: &[f64], _r: &Report) -> f64 {
        let v = grashof_transform(v);
        let curves = self.available_curve(&v);
        if curves.is_empty() {
            return 1e10;
        }
        curves
            .into_iter()
            .map(|(inv, curve)| {
                let curve = concatenate![Axis(0), curve, arr2(&[[curve[[0, 0]], curve[[0, 1]]]])];
                let coeffs = calculate_efd(&curve, self.harmonic);
                let (coeffs, rot, _, scale) = normalize_efd(&coeffs, true);
                let four_bar = self.four_bar_from_coeff(&v, inv, rot, scale, locus(&curve));
                let curve = Mechanism::four_bar(four_bar).four_bar_loop(0., self.n * 2);
                let geo_err = geo_err(&self.curve, &curve);
                (coeffs - &self.target).mapv(f64::abs).sum() + geo_err * 1e-5
            })
            .fold(f64::INFINITY, |a, b| a.min(b))
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        let v = grashof_transform(v);
        let curves = self.available_curve(&v);
        if curves.is_empty() {
            println!("WARNING: synthesis failed");
            return four_bar_from_v(&v, false);
        }
        let coeffs = curves
            .iter()
            .map(|(_, c)| {
                let curve = concatenate!(Axis(0), *c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
                calculate_efd(&curve, self.harmonic)
            })
            .collect::<Vec<_>>();
        let mut index = 0;
        let mut min_err = f64::INFINITY;
        for (i, coeffs) in coeffs.iter().enumerate() {
            let (coeffs, ..) = normalize_efd(coeffs, true);
            let err = (coeffs - &self.target).mapv(f64::abs).sum();
            if err < min_err {
                index = i;
                min_err = err;
            }
        }
        let (inv, curve) = &curves[index];
        let (_, rot, _, scale) = normalize_efd(&coeffs[index], true);
        self.four_bar_from_coeff(&v, *inv, rot, scale, locus(curve))
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
    callback: impl FnMut(&Report) -> bool,
) -> (FourBar, Vec<Report>) {
    let planar = Planar::new(curve, 720, 360);
    let s = Solver::solve(
        planar,
        De::default()
            .task(Task::MaxGen(gen))
            .pop_num(pop)
            .average(true),
        callback,
    );
    (s.result(), s.reports())
}
