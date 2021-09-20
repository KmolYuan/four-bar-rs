//! The synthesis implementation of planar four-bar linkage mechanisms.
use crate::{FourBar, Mechanism};
use efd::{calculate_efd, locus, normalize_efd};
pub use metaheuristics_nature::*;
use ndarray::{arr2, concatenate, Array2, AsArray, Axis, Ix2};
use std::f64::consts::TAU;

fn path_is_nan<'a, V>(path: V) -> bool
where
    V: AsArray<'a, f64, Ix2>,
{
    let path = path.into();
    for i in 0..path.nrows() {
        if path[[i, 0]].is_nan() || path[[i, 1]].is_nan() {
            return true;
        }
    }
    false
}

// Normalization information
#[derive(Default)]
struct Norm {
    target: Array2<f64>,
    rot: f64,
    scale: f64,
    locus: (f64, f64),
}

/// Synthesis task of planar four-bar linkage.
pub struct Planar {
    norm: Norm,
    n: usize,
    harmonic: usize,
    ub: Vec<f64>,
    lb: Vec<f64>,
}

impl Planar {
    /// Create a new task.
    pub fn new(curve: &[[f64; 2]], n: usize, harmonic: usize) -> Self {
        let mut curve = arr2(curve);
        let end = curve.nrows() - 1;
        let mut ub = vec![10.; 5];
        let mut lb = vec![1e-6; 5];
        // gamma
        ub[4] = TAU;
        lb[4] = 0.;
        let norm = {
            // Close loop
            if (curve[[0, 0]] - curve[[end, 0]]).abs() > 1e-20
                || (curve[[0, 1]] - curve[[end, 1]]).abs() > 1e-20
            {
                curve = concatenate!(Axis(0), curve, arr2(&[[curve[[0, 0]], curve[[0, 1]]]]));
            }
            let coeffs = calculate_efd(&curve, harmonic);
            let (target, rot, _, scale) = normalize_efd(&coeffs, true);
            Norm {
                target,
                rot,
                scale,
                locus: locus(&curve),
            }
        };
        Self {
            norm,
            n,
            harmonic,
            ub,
            lb,
        }
    }

    fn four_bar(v: &[f64]) -> FourBar {
        FourBar {
            p0: (0., 0.),
            a: 0.,
            l0: v[0],
            l1: 1.,
            l2: v[1],
            l3: v[2],
            l4: v[3],
            g: v[4],
        }
    }
}

impl ObjFunc for Planar {
    type Result = FourBar;

    fn fitness(&self, v: &[f64], _: &Report) -> f64 {
        let mut f = Mechanism::four_bar(Self::four_bar(v));
        let c = arr2(&f.four_bar_loop(0., self.n));
        if path_is_nan(&c) {
            return 1e20;
        }
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (coeffs, ..) = normalize_efd(&coeffs, true);
        (coeffs - &self.norm.target).mapv(f64::abs).sum()
    }

    fn result(&self, v: &[f64]) -> Self::Result {
        let c = arr2(&Mechanism::four_bar(Self::four_bar(v)).four_bar_loop(0., self.n));
        let curve = concatenate!(Axis(0), c, arr2(&[[c[[0, 0]], c[[0, 1]]]]));
        let coeffs = calculate_efd(&curve, self.harmonic);
        let (_, rot, _, scale) = normalize_efd(&coeffs, true);
        let locus = locus(&curve);
        let rot = rot - self.norm.rot;
        let scale = self.norm.scale / scale;
        let locus_rot = locus.1.atan2(locus.0) + rot;
        let d = locus.1.hypot(locus.0) * scale;
        FourBar {
            p0: (
                self.norm.locus.0 - d * locus_rot.cos(),
                self.norm.locus.1 - d * locus_rot.sin(),
            ),
            a: rot,
            l0: v[0] * scale,
            l1: scale,
            l2: v[1] * scale,
            l3: v[2] * scale,
            l4: v[3] * scale,
            g: v[4],
        }
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
    gen: u32,
    pop: usize,
    callback: impl FnMut(&Report) -> bool,
) -> (FourBar, Vec<Report>) {
    let planar = Planar::new(curve, 720, 360);
    let s = Solver::solve(
        planar,
        De::default().task(Task::MaxGen(gen)).rpt(1).pop_num(pop),
        callback,
    );
    (s.result(), s.reports())
}
